use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_catalog::{Behavior, CatalogView, Lookup};
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
    /// - Generated columns (GENERATED ALWAYS AS) always require table rewrites
    /// - Non-volatile defaults are safe in Postgres 11+
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

        // Check Postgres version - in 11+, non-volatile defaults are safe
        let pg_version = ctx.catalog().server_version_num();

        if let pgls_query::NodeEnum::AlterTableStmt(stmt) = &ctx.stmt() {
            for cmd in &stmt.cmds {
                if let Some(pgls_query::NodeEnum::AlterTableCmd(cmd)) = &cmd.node
                    && cmd.subtype() == pgls_query::protobuf::AlterTableType::AtAddColumn
                    && let Some(pgls_query::NodeEnum::ColumnDef(col_def)) =
                        &cmd.def.as_ref().and_then(|d| d.node.as_ref())
                {
                    let has_default = col_def.constraints.iter().any(|constraint| {
                        if let Some(pgls_query::NodeEnum::Constraint(c)) = &constraint.node {
                            c.contype() == pgls_query::protobuf::ConstrType::ConstrDefault
                        } else {
                            false
                        }
                    });

                    let has_generated = col_def.constraints.iter().any(|constraint| {
                        if let Some(pgls_query::NodeEnum::Constraint(c)) = &constraint.node {
                            c.contype() == pgls_query::protobuf::ConstrType::ConstrGenerated
                        } else {
                            false
                        }
                    });

                    if has_generated {
                        diagnostics.push(
                                    LinterDiagnostic::new(
                                        rule_category!(),
                                        None,
                                        markup! {
                                            "Adding a generated column requires a table rewrite."
                                        },
                                    )
                                    .detail(None, "This operation requires an ACCESS EXCLUSIVE lock and rewrites the entire table.")
                                    .note("Add the column as nullable, backfill existing rows, and add a trigger to update the column on write instead."),
                                );
                    } else if has_default {
                        // For PG 11+, check if the default is volatile
                        if pg_version.is_some_and(|v| v >= 110000) {
                            // Check if default is non-volatile
                            let is_safe_default = col_def.constraints.iter().any(|constraint| {
                                if let Some(pgls_query::NodeEnum::Constraint(c)) = &constraint.node
                                    && c.contype()
                                        == pgls_query::protobuf::ConstrType::ConstrDefault
                                    && let Some(raw_expr) = &c.raw_expr
                                {
                                    return is_safe_default_expr(
                                        &raw_expr.node.as_ref().map(|n| Box::new(n.clone())),
                                        ctx,
                                    );
                                }
                                false
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
        }

        diagnostics
    }
}

fn is_safe_default_expr(
    expr: &Option<Box<pgls_query::NodeEnum>>,
    ctx: &LinterRuleContext<AddingFieldWithDefault>,
) -> bool {
    match expr {
        Some(node) => match node.as_ref() {
            // Constants are always safe
            pgls_query::NodeEnum::AConst(_) => true,
            // Type casts of constants are safe
            pgls_query::NodeEnum::TypeCast(tc) => is_safe_default_expr(
                &tc.arg.as_ref().and_then(|a| a.node.clone()).map(Box::new),
                ctx,
            ),
            // function calls might be safe if they are non-volatile and have no args
            pgls_query::NodeEnum::FuncCall(fc) => {
                // must have no args
                if !fc.args.is_empty() {
                    return false;
                }

                let Some((schema, name)) = pgls_query_ext::utils::parse_name(&fc.funcname) else {
                    return false;
                };

                let Lookup::Found(functions) =
                    ctx.catalog()
                        .functions(schema.as_deref(), &name, ctx.session().search_path())
                else {
                    return false;
                };

                // The functions come in the order of the search path, and the call resolves to
                // one in the first schema that has a function callable without arguments.
                let candidates: Vec<_> = functions
                    .iter()
                    .filter(|function| function.min_args == 0)
                    .collect();
                let Some(first) = candidates.first() else {
                    return false;
                };
                // Postgres passes the defaults of the parameters the call leaves out, which may
                // be volatile even if the function isn't. We don't know them, so only functions
                // without parameters are safe.
                candidates
                    .iter()
                    .filter(|function| function.schema == first.schema)
                    .all(|function| {
                        function.max_args == Some(0)
                            && function
                                .volatility
                                .is_some_and(|volatility| volatility != Behavior::Volatile)
                    })
            }
            // everything else is potentially unsafe
            _ => false,
        },
        None => false,
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
