use crate::utils::{alter_table_cmds, cmd_def, column_constraints};
use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_catalog::{CatalogView, Decision, contains_volatile_functions};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Adding a column with a DEFAULT value may lead to a table rewrite while holding an ACCESS EXCLUSIVE lock.
    ///
    /// In Postgres versions before 11, adding a column with a DEFAULT value causes a full table rewrite,
    /// which holds an ACCESS EXCLUSIVE lock on the table and blocks all reads and writes.
    ///
    /// In Postgres 11+, this behavior was optimized for non-volatile defaults. However:
    /// - Volatile default values (like random() or custom functions) still cause table rewrites
    /// - Stored generated columns (GENERATED ALWAYS AS ... STORED) always require table rewrites
    /// - Non-volatile defaults are safe in Postgres 11+
    ///
    /// The server version and the volatility of the functions, operators and casts of a default
    /// come from the database. Without a connection, or when the volatility of a default isn't
    /// known, the rule assumes that it rewrites the table.
    ///
    /// Virtual generated columns (Postgres 18) are computed when rows are read, so adding one
    /// doesn't rewrite the table.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// ALTER TABLE "core_recipe" ADD COLUMN "foo" integer DEFAULT 10;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// ALTER TABLE "core_recipe" ADD COLUMN "foo" integer;
    /// ALTER TABLE "core_recipe" ALTER COLUMN "foo" SET DEFAULT 10;
    /// -- Then backfill and add NOT NULL constraint if needed
    /// ```
    ///
    pub AddingFieldWithDefault {
        version: "0.15.0",
        name: "addingFieldWithDefault",
        severity: Severity::Warning,
        recommended: true,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Squawk("adding-field-with-default")],
    }
}

impl LinterRule for AddingFieldWithDefault {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let mut diagnostics = Vec::new();

        // Since Postgres 11, a default without volatile functions needs no rewrite.
        let has_fast_defaults = ctx.catalog().server_version_at_least(11) == Some(true);

        for cmd in alter_table_cmds(ctx.stmt()) {
            if cmd.subtype() == pgls_query::protobuf::AlterTableType::AtAddColumn
                && let Some(pgls_query::NodeEnum::ColumnDef(col_def)) = cmd_def(cmd)
            {
                let has_default = column_constraints(col_def)
                    .any(|c| c.contype() == pgls_query::protobuf::ConstrType::ConstrDefault);

                // A virtual generated column ('v', Postgres 18) is computed on read and
                // needs no rewrite.
                let has_stored_generated = column_constraints(col_def).any(|c| {
                    c.contype() == pgls_query::protobuf::ConstrType::ConstrGenerated
                        && c.generated_kind == "s"
                });

                if has_stored_generated {
                    diagnostics.push(
                                LinterDiagnostic::new(
                                    rule_category!(),
                                    None,
                                    markup! {
                                        "Adding a stored generated column requires a table rewrite."
                                    },
                                )
                                .detail(None, "This operation requires an ACCESS EXCLUSIVE lock and rewrites the entire table.")
                                .note("Add the column as nullable, backfill existing rows, and add a trigger to update the column on write instead."),
                            );
                } else if has_default {
                    // For PG 11+, check if the default is volatile
                    if has_fast_defaults {
                        let is_safe_default = column_constraints(col_def).any(|c| {
                            c.contype() == pgls_query::protobuf::ConstrType::ConstrDefault
                                && c.raw_expr
                                    .as_ref()
                                    .and_then(|raw_expr| raw_expr.node.as_ref())
                                    .is_some_and(|raw_expr| {
                                        contains_volatile_functions(
                                            raw_expr,
                                            ctx.catalog(),
                                            ctx.session().search_path(),
                                        ) == Decision::Known(false)
                                    })
                        });

                        if !is_safe_default {
                            diagnostics.push(
                                        LinterDiagnostic::new(
                                            rule_category!(),
                                            None,
                                            markup! {
                                                "Adding a column with a volatile default value causes a table rewrite."
                                            },
                                        )
                                        .detail(None, "Even in Postgres 11+, volatile default values require a full table rewrite.")
                                        .note("Add the column without a default, then set the default in a separate statement."),
                                    );
                        }
                    } else {
                        // Pre PG 11, all defaults cause rewrites
                        diagnostics.push(
                                    LinterDiagnostic::new(
                                        rule_category!(),
                                        None,
                                        markup! {
                                            "Adding a column with a DEFAULT value causes a table rewrite."
                                        },
                                    )
                                    .detail(None, "This operation requires an ACCESS EXCLUSIVE lock and rewrites the entire table.")
                                    .note("Add the column without a default, then set the default in a separate statement."),
                                );
                    }
                }
            }
        }

        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use pgls_analyse::{AnalysisFilter, RuleFilter};
    use pgls_catalog::{Behavior, CatalogBase, Function, Schema, Snapshot, Version};
    use pgls_text_size::TextRange;

    use crate::{AnalysableStatement, Analyser, AnalyserConfig, AnalyserParams, LinterOptions};

    /// A Postgres 17 database with the functions `pg_catalog.now()` (stable),
    /// `pg_catalog.random()` (volatile), and `app.now()` (volatile).
    fn database() -> Arc<CatalogBase> {
        let function = |schema: &str, name: &str, behavior| Function {
            schema: schema.into(),
            name: name.into(),
            behavior,
            ..Default::default()
        };
        let schema = |name: &str| Schema {
            name: name.into(),
            ..Default::default()
        };
        Arc::new(CatalogBase::new(Arc::new(Snapshot {
            schemas: vec![schema("pg_catalog"), schema("public"), schema("app")],
            functions: vec![
                function("pg_catalog", "now", Behavior::Stable),
                function("pg_catalog", "random", Behavior::Volatile),
                function("app", "now", Behavior::Volatile),
            ],
            version: Version {
                version_num: Some(170000),
                major_version: Some(17),
                ..Default::default()
            },
            ..Default::default()
        })))
    }

    /// The number of diagnostics for the statements, separated by `;`.
    fn diagnostics(sql: &str) -> usize {
        let rule_filter = RuleFilter::Rule("safety", "addingFieldWithDefault");
        let options = LinterOptions::default();
        let analyser = Analyser::new(AnalyserConfig {
            options: &options,
            filter: AnalysisFilter {
                enabled_rules: Some(std::slice::from_ref(&rule_filter)),
                ..Default::default()
            },
        });
        let stmts = pgls_query::parse(sql)
            .unwrap()
            .stmts()
            .into_iter()
            .map(|stmt| AnalysableStatement::new(stmt.clone(), TextRange::default()))
            .collect();
        analyser
            .run(AnalyserParams {
                stmts,
                catalog_base: Some(database()),
                search_path: vec!["public".into()],
                ..Default::default()
            })
            .len()
    }

    #[test]
    fn non_volatile_functions_are_safe_defaults() {
        assert_eq!(
            diagnostics("alter table t add column c timestamptz default now()"),
            0
        );
        assert_eq!(
            diagnostics("alter table t add column c float8 default random()"),
            1
        );
        assert_eq!(
            diagnostics("alter table t add column c timestamptz default app.now()"),
            1
        );
        assert_eq!(
            diagnostics("alter table t add column c int default missing()"),
            1
        );
    }

    #[test]
    fn functions_follow_the_search_path() {
        // `pg_catalog` comes first, unless the search path lists it later.
        assert_eq!(
            diagnostics(
                "set search_path = app; alter table t add column c timestamptz default now()"
            ),
            0
        );
        assert_eq!(
            diagnostics(
                "set search_path = app, pg_catalog; alter table t add column c timestamptz default now()"
            ),
            1
        );
    }

    #[test]
    fn functions_of_the_file_are_known() {
        assert_eq!(
            diagnostics(
                "create function public.one() returns int language sql immutable as 'select 1';
                 alter table t add column c int default one()"
            ),
            0
        );
        assert_eq!(
            diagnostics(
                "create function public.one() returns int language sql as 'select 1';
                 alter table t add column c int default one()"
            ),
            1
        );
        assert_eq!(
            diagnostics(
                "create function public.one() returns int language sql immutable as 'select 1';
                 alter function public.one() volatile;
                 alter table t add column c int default one()"
            ),
            1
        );
    }

    #[test]
    fn functions_with_defaulted_parameters_are_unsafe() {
        // Postgres passes `random()` for `x`, so the default is volatile.
        assert_eq!(
            diagnostics(
                "create function public.f(x float8 default random()) returns float8 language sql immutable as 'select x';
                 alter table t add column c float8 default f()"
            ),
            1
        );
        assert_eq!(
            diagnostics(
                "create function public.f(variadic xs int[] default '{}') returns int language sql immutable as 'select 1';
                 alter table t add column c int default f()"
            ),
            1
        );
    }
}
