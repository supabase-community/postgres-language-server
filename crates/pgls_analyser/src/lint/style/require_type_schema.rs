use std::collections::HashSet;

use crate::utils::statement_nodes;
use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::declare_lint_rule;
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::protobuf::{AlterTableType, Node, TypeName};
use pgls_query::{NodeEnum, NodeRef};
use pgls_query_ext::utils::string_value;

/// The serial pseudo-types. Postgres recognizes them by their unqualified name in the column
/// definitions of `CREATE TABLE` and `ALTER TABLE ... ADD COLUMN`, and doesn't look them up
/// there, so `pg_catalog.serial` is an error. Elsewhere, like in a cast, they are looked up like
/// any other type name.
///
/// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_utilcmd.c#L604
const SERIAL_TYPES: &[&str] = &[
    "smallserial",
    "serial2",
    "serial",
    "serial4",
    "bigserial",
    "serial8",
];

declare_lint_rule! {
    /// Type names should be schema-qualified.
    ///
    /// Without a schema, Postgres looks the type up in the schemas on the `search_path`. A type
    /// with the same name in a schema earlier on the search path is used instead of the intended
    /// one, which changes what a cast or a function call does, and which function overload is
    /// called.
    ///
    /// The rule is meant for code where this matters, like functions with `SECURITY DEFINER` and
    /// extension scripts, as defence in depth next to setting the `search_path` explicitly.
    ///
    /// The rule checks every type name, like the ones of casts, column definitions, and function
    /// arguments and results. The types whose names are SQL keywords, like `integer`, `varchar`,
    /// `timestamp` or `boolean`, always refer to `pg_catalog` and are not reported. Types like
    /// `text`, `uuid` or `jsonb` are not keywords and are reported. The serial pseudo-types of
    /// the columns of `CREATE TABLE` and `ALTER TABLE ... ADD COLUMN`, which Postgres doesn't
    /// look up, and `%TYPE` references are not reported either.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// select 'hello'::text;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// select 'hello'::pg_catalog.text;
    /// select 1::integer;
    /// create table public.users (id serial, created_at timestamp with time zone);
    /// ```
    ///
    pub RequireTypeSchema {
        version: "next",
        name: "requireTypeSchema",
        severity: Severity::Warning,
        recommended: false,
    }
}

impl LinterRule for RequireTypeSchema {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let tokens = ctx.tokens();
        let nodes = statement_nodes(ctx.stmt());
        // The types of the columns that can be serial, by address.
        let column_types: HashSet<*const TypeName> = nodes
            .iter()
            .flat_map(|node| {
                let column_defs: Vec<&Node> = match node {
                    NodeRef::CreateStmt(create) => create.table_elts.iter().collect(),
                    NodeRef::AlterTableCmd(cmd) if cmd.subtype() == AlterTableType::AtAddColumn => {
                        cmd.def.as_deref().into_iter().collect()
                    }
                    _ => vec![],
                };
                column_defs
                    .into_iter()
                    .filter_map(|column_def| match &column_def.node {
                        Some(NodeEnum::ColumnDef(column_def)) => column_def.type_name.as_ref(),
                        _ => None,
                    })
                    .map(std::ptr::from_ref)
            })
            .collect();
        let mut names: Vec<(&str, i32)> = nodes
            .into_iter()
            .filter_map(|node| match node {
                NodeRef::TypeName(type_name) if !type_name.pct_type => {
                    match type_name.names.as_slice() {
                        [name] => {
                            let name = string_value(name)?;
                            let is_serial = SERIAL_TYPES.contains(&name)
                                && column_types.contains(&std::ptr::from_ref(type_name));
                            (!is_serial).then_some((name, type_name.location))
                        }
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
                        "Type "<Emphasis>{name}</Emphasis>" is not schema-qualified."
                    },
                )
                .detail(
                    None,
                    "Without a schema, the type is resolved through the search_path: a type with the same name in a schema earlier on the search_path is used instead.",
                )
                .note(markup! {
                    "Specify the schema, e.g. "<Emphasis>"pg_catalog."{name}</Emphasis>" for a built-in type."
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    fn spans(sql: &str) -> Vec<String> {
        crate::spanned_texts("style", "requireTypeSchema", sql)
    }

    #[test]
    fn points_at_the_type_name() {
        assert_eq!(
            spans("select x::\"Mood\"[], cast(y as uuid), z::pg_catalog.text from t"),
            ["\"Mood\"", "uuid"]
        );
    }

    #[test]
    fn skips_keyword_types() {
        assert!(
            spans(
                "select 1::integer, 1::int, 'a'::varchar(3), 'a'::character varying, \
                 now()::timestamp with time zone, true::boolean, 1::double precision, \
                 interval '1 day'"
            )
            .is_empty()
        );
    }

    #[test]
    fn reports_serial_outside_column_definitions() {
        assert_eq!(
            spans(
                "create function public.f(a serial) returns serial language sql \
                 as 'select 1::\"serial\"'"
            ),
            ["serial", "serial"]
        );
        assert_eq!(
            spans("select 1::serial, 2::\"bigserial\""),
            ["serial", "\"bigserial\""]
        );
        assert_eq!(
            spans("alter table public.t add column b serial, alter column a type serial"),
            ["serial"]
        );
    }

    #[test]
    fn skips_serial_and_pct_type() {
        assert_eq!(
            spans("create table public.t (a serial, b bigserial, c text)"),
            ["text"]
        );
        assert!(
            spans(
                "create function public.f(x public.t.a%type) returns int language sql as 'select 1'"
            )
            .is_empty()
        );
    }
}
