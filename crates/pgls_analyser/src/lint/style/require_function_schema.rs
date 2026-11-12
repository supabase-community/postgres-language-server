use crate::utils::statement_nodes;
use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::declare_lint_rule;
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::NodeRef;
use pgls_query::protobuf::CoercionForm;
use pgls_query_ext::utils::string_value;

declare_lint_rule! {
    /// Function calls should be schema-qualified.
    ///
    /// Without a schema, Postgres looks the function up in every schema on the `search_path` and
    /// calls the best match. A function with the same name in a schema earlier on the search
    /// path, or an overload in any of them whose argument types match the call better, is called
    /// instead of the intended one. Anyone who can create functions in one of these schemas can
    /// run code with the privileges of the caller.
    ///
    /// The rule is meant for code where this matters, like functions with `SECURITY DEFINER` and
    /// extension scripts, as defence in depth next to setting the `search_path` explicitly.
    ///
    /// The rule checks every function call, including aggregates, window functions, `CALL` and
    /// calls in DDL, like column defaults and index expressions. It doesn't report the SQL syntax
    /// that Postgres turns into a call of a `pg_catalog` function itself, like `extract(...)`,
    /// `position(... in ...)`, `substring(... from ...)`, `overlay`, `trim`, `current_date` or
    /// `coalesce`.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// select abs(1);
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// select pg_catalog.abs(1);
    /// select extract(year from pg_catalog.now());
    /// ```
    ///
    pub RequireFunctionSchema {
        version: "next",
        name: "requireFunctionSchema",
        severity: Severity::Warning,
        recommended: false,
    }
}

impl LinterRule for RequireFunctionSchema {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let tokens = ctx.tokens();
        let mut names: Vec<(&str, i32)> = statement_nodes(ctx.stmt())
            .into_iter()
            .filter_map(|node| match node {
                NodeRef::FuncCall(call) if call.funcformat() != CoercionForm::CoerceSqlSyntax => {
                    match call.funcname.as_slice() {
                        [name] => Some((string_value(name)?, call.location)),
                        _ => None,
                    }
                }
                _ => None,
            })
            .collect();
        names.sort_by_key(|(_, location)| *location);
        names
            .into_iter()
            .map(|(name, location)| {
                LinterDiagnostic::new(
                    rule_category!(),
                    tokens.and_then(|tokens| tokens.range_at(location)),
                    markup! {
                        "Function "<Emphasis>{name}</Emphasis>" is not schema-qualified."
                    },
                )
                .detail(
                    None,
                    "Without a schema, the function is resolved through the search_path: a function with the same name earlier on the search_path, or an overload whose argument types match better, is called instead.",
                )
                .note(markup! {
                    "Specify the schema, e.g. "<Emphasis>"pg_catalog."{name}</Emphasis>" for a built-in function."
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    fn spans(sql: &str) -> Vec<String> {
        crate::spanned_texts("style", "requireFunctionSchema", sql)
    }

    #[test]
    fn points_at_the_function_name() {
        assert_eq!(spans("select \"Abs\"(1), pg_catalog.abs(2)"), ["\"Abs\""]);
        assert_eq!(spans("select count(*) over () from t"), ["count"]);
    }

    #[test]
    fn skips_sql_syntax() {
        assert!(
            spans(
                "select extract(year from x), position('a' in x), substring(x from 1 for 2), \
                 overlay(x placing 'a' from 1), trim(both from x), current_date, \
                 coalesce(x, 1), nullif(x, 1), greatest(x, 1), x at time zone 'utc' from t"
            )
            .is_empty()
        );
    }

    #[test]
    fn skips_the_body_of_a_sql_function() {
        // The statements of the body are analysed on their own.
        assert_eq!(
            spans("create function public.f(x int default g()) returns int return abs(x)"),
            ["g"]
        );
    }
}
