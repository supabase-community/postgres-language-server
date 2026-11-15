use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::declare_lint_rule;
use pgls_catalog::resolve::{LookupKind, OperatorConstruct, SearchPathLookup};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Operators should be schema-qualified.
    ///
    /// Without a schema, Postgres looks the operator up in every schema on the `search_path`
    /// and uses the best match. An operator with the same name in a schema earlier on the
    /// search path, or one in any of them whose argument types match better, is used instead of
    /// the intended one. Anyone who can create operators in one of these schemas can run code
    /// with the privileges of the caller. A schema-qualified operator is written as
    /// `OPERATOR(schema.op)`, e.g. `a OPERATOR(pg_catalog.=) b`.
    ///
    /// The rule is meant for code where this matters, like functions with `SECURITY DEFINER` and
    /// extension scripts, as defence in depth next to setting the `search_path` explicitly.
    ///
    /// The rule checks binary and unary operators, `op ANY (...)` and `op ALL (...)` with arrays
    /// and subqueries, and `ORDER BY ... USING op`. It also reports the SQL constructs that
    /// Postgres turns into operators that it looks up through the search path: `IN` and
    /// `NOT IN` (`=` and `<>`), `LIKE` and `ILIKE` (`~~` and `~~*`), `SIMILAR TO` (`~`),
    /// `IS [NOT] DISTINCT FROM` and `NULLIF` (`=`), and `BETWEEN` (`>=` and `<=`). They have no
    /// schema-qualified form, so the comparison has to be written with `OPERATOR(...)` instead.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// select * from pg_catalog.pg_stat_io where backend_type = 'client backend';
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// select * from pg_catalog.pg_class where relkind in ('r', 'p');
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// select * from pg_catalog.pg_stat_io where backend_type operator(pg_catalog.=) 'client backend';
    /// select * from pg_catalog.pg_class where relkind operator(pg_catalog.=) any (array['r', 'p']);
    /// ```
    ///
    pub RequireOperatorSchema {
        version: "next",
        name: "requireOperatorSchema",
        severity: Severity::Warning,
        recommended: false,
    }
}

impl LinterRule for RequireOperatorSchema {
    type Options = ();
    const READS_LOOKUPS: bool = true;

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let mut lookups: Vec<(&SearchPathLookup, Option<OperatorConstruct>)> = ctx
            .lookups()
            .iter()
            .filter_map(|lookup| match lookup.kind {
                LookupKind::Operator(construct) => Some((lookup, construct)),
                _ => None,
            })
            .collect();
        lookups.sort_by_key(|(lookup, _)| lookup.location);
        lookups
            .into_iter()
            .map(|(lookup, construct)| {
                let operator = &lookup.name;
                let diagnostic = match construct {
                    None => LinterDiagnostic::new(
                        rule_category!(),
                        lookup.span,
                        markup! {
                            "Operator "<Emphasis>{operator}</Emphasis>" is not schema-qualified."
                        },
                    ),
                    Some(construct) => {
                        let keywords = construct.keywords();
                        let (noun, operators) = match construct.operators() {
                            [operator] => ("operator", (*operator).to_owned()),
                            operators => ("operators", operators.join(" and ")),
                        };
                        LinterDiagnostic::new(
                            rule_category!(),
                            lookup.span,
                            markup! {
                                <Emphasis>{keywords}</Emphasis>" uses the "{noun}" "<Emphasis>{operators}</Emphasis>" without a schema."
                            },
                        )
                    }
                };
                let diagnostic = diagnostic.detail(
                    None,
                    "Without a schema, the operator is resolved through the search_path: an operator with the same name earlier on the search_path, or one whose argument types match better, is used instead.",
                );
                match construct {
                    None => diagnostic.note(markup! {
                        "Specify the schema, e.g. "<Emphasis>"OPERATOR(pg_catalog."{operator}")"</Emphasis>"."
                    }),
                    Some(construct) => {
                        let keywords = construct.keywords();
                        diagnostic.note(markup! {
                            <Emphasis>{keywords}</Emphasis>" has no schema-qualified form. Write the comparison with "<Emphasis>"OPERATOR(pg_catalog."{operator}")"</Emphasis>" instead."
                        })
                    }
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    fn spans(sql: &str) -> Vec<String> {
        crate::spanned_texts("style", "requireOperatorSchema", sql)
    }

    #[test]
    fn points_at_the_operator() {
        assert_eq!(
            spans("select a = b, -a, a operator(=) b, a operator(pg_catalog.=) b, a != b from t"),
            ["=", "-", "operator", "!="]
        );
        assert_eq!(
            spans("select a = any (array[1]), a < all (select b from u) from t"),
            ["=", "<"]
        );
        assert_eq!(spans("select a from t order by a using <"), ["<"]);
    }

    #[test]
    fn points_at_the_construct() {
        assert_eq!(
            spans(
                "select a in (1), a not in (1), a in (select b from u), a like 'x', \
                 a not ilike 'x', a similar to 'x', a is not distinct from b, nullif(a, b), \
                 a not between symmetric 1 and 2 from t"
            ),
            [
                "in",
                "not in",
                "in",
                "like",
                "not ilike",
                "similar to",
                "is not distinct from",
                "nullif",
                "not between symmetric"
            ]
        );
    }
}
