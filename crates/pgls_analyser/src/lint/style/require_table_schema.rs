use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::protobuf::{ObjectType, RangeVar};

/// The `relpersistence` of temporary tables, which always live in `pg_temp`.
const RELPERSISTENCE_TEMP: &str = "t";

declare_lint_rule! {
    /// Table names in DDL should be schema-qualified.
    ///
    /// Without a schema, Postgres resolves the table through the `search_path`, which depends on
    /// the session that runs the migration. This makes it ambiguous which table a statement acts
    /// on. Temporary tables are not reported.
    ///
    /// The rule checks `CREATE TABLE`, `CREATE TABLE AS`, `ALTER TABLE`, and `DROP TABLE`.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// create table posts (id bigint);
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// alter table posts add column title text;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// create table public.posts (id bigint);
    /// alter table public.posts add column title text;
    /// drop table public.posts;
    /// create temp table scratch (id bigint);
    /// ```
    ///
    pub RequireTableSchema {
        version: "next",
        name: "requireTableSchema",
        severity: Severity::Warning,
        recommended: false,
        sources: &[RuleSource::Squawk("require-table-schema")],
    }
}

impl LinterRule for RequireTableSchema {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let unqualified: Vec<&str> = match ctx.stmt() {
            pgls_query::NodeEnum::CreateStmt(stmt) => {
                stmt.relation.iter().filter_map(unqualified_table).collect()
            }
            pgls_query::NodeEnum::CreateTableAsStmt(stmt)
                if stmt.objtype() == ObjectType::ObjectTable =>
            {
                stmt.into
                    .iter()
                    .filter_map(|into| into.rel.as_ref())
                    .filter_map(unqualified_table)
                    .collect()
            }
            pgls_query::NodeEnum::AlterTableStmt(stmt)
                if stmt.objtype() == ObjectType::ObjectTable =>
            {
                stmt.relation.iter().filter_map(unqualified_table).collect()
            }
            pgls_query::NodeEnum::DropStmt(stmt)
                if stmt.remove_type() == ObjectType::ObjectTable =>
            {
                stmt.objects
                    .iter()
                    .filter_map(|object| match &object.node {
                        Some(pgls_query::NodeEnum::List(name)) => match name.items.as_slice() {
                            [name] => match &name.node {
                                Some(pgls_query::NodeEnum::String(name)) => {
                                    Some(name.sval.as_str())
                                }
                                _ => None,
                            },
                            _ => None,
                        },
                        _ => None,
                    })
                    .collect()
            }
            _ => vec![],
        };

        unqualified
            .into_iter()
            .map(|table| {
                LinterDiagnostic::new(
                    rule_category!(),
                    None,
                    markup! {
                        "Table "<Emphasis>{table}</Emphasis>" is not schema-qualified."
                    },
                )
                .detail(
                    None,
                    "Without a schema, the table is resolved through the search_path of the session that runs the statement.",
                )
                .note("Specify the schema, e.g. public.my_table.")
            })
            .collect()
    }
}

fn unqualified_table(relation: &RangeVar) -> Option<&str> {
    (relation.schemaname.is_empty() && relation.relpersistence != RELPERSISTENCE_TEMP)
        .then_some(relation.relname.as_str())
}
