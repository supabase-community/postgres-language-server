use pgt_analyse::{Rule, RuleDiagnostic, RuleSource, context::RuleContext, declare_lint_rule};
use pgt_console::markup;
use pgt_diagnostics::Severity;
use std::collections::HashSet;

declare_lint_rule! {
    /// Adding a column with a DEFAULT value may lead to a table rewrite while holding an ACCESS EXCLUSIVE lock.
    ///
    /// In PostgreSQL versions before 11, adding a column with a DEFAULT value causes a full table rewrite,
    /// which holds an ACCESS EXCLUSIVE lock on the table and blocks all reads and writes.
    ///
    /// In PostgreSQL 11+, this behavior was optimized for non-volatile defaults. However:
    /// - Volatile default values (like random() or custom functions) still cause table rewrites
    /// - Generated columns (GENERATED ALWAYS AS) always require table rewrites
    /// - Non-volatile defaults are safe in PostgreSQL 11+
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
        version: "next",
        name: "addingFieldWithDefault",
        severity: Severity::Warning,
        recommended: true,
        sources: &[RuleSource::Squawk("adding-field-with-default")],
    }
}

// Generated via the following Postgres query:
//      select proname from pg_proc where provolatile <> 'v';
const NON_VOLATILE_BUILT_IN_FUNCTIONS: &str =
    include_str!("../../../resources/non_volatile_built_in_functions.txt");

impl Rule for AddingFieldWithDefault {
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Vec<RuleDiagnostic> {
        let mut diagnostics = Vec::new();

        // Check PostgreSQL version - in 11+, non-volatile defaults are safe
        let pg_version = ctx.schema_cache().and_then(|sc| sc.version.major_version);

        if let pgt_query::NodeEnum::AlterTableStmt(stmt) = &ctx.stmt() {
            for cmd in &stmt.cmds {
                if let Some(pgt_query::NodeEnum::AlterTableCmd(cmd)) = &cmd.node {
                    if cmd.subtype() == pgt_query::protobuf::AlterTableType::AtAddColumn {
                        if let Some(pgt_query::NodeEnum::ColumnDef(col_def)) =
                            &cmd.def.as_ref().and_then(|d| d.node.as_ref())
                        {
                            let has_default = col_def.constraints.iter().any(|constraint| {
                                if let Some(pgt_query::NodeEnum::Constraint(c)) = &constraint.node {
                                    c.contype() == pgt_query::protobuf::ConstrType::ConstrDefault
                                } else {
                                    false
                                }
                            });

                            let has_generated = col_def.constraints.iter().any(|constraint| {
                                if let Some(pgt_query::NodeEnum::Constraint(c)) = &constraint.node {
                                    c.contype() == pgt_query::protobuf::ConstrType::ConstrGenerated
                                } else {
                                    false
                                }
                            });

                            if has_generated {
                                diagnostics.push(
                                    RuleDiagnostic::new(
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
                                if pg_version.is_some_and(|v| v >= 11) {
                                    let non_volatile_funcs: HashSet<_> =
                                        NON_VOLATILE_BUILT_IN_FUNCTIONS
                                            .lines()
                                            .map(|x| x.trim().to_lowercase())
                                            .filter(|x| !x.is_empty())
                                            .collect();

                                    // Check if default is non-volatile
                                    let is_safe_default = col_def.constraints.iter().any(|constraint| {
                                        if let Some(pgt_query::NodeEnum::Constraint(c)) = &constraint.node {
                                            if c.contype() == pgt_query::protobuf::ConstrType::ConstrDefault {
                                                if let Some(raw_expr) = &c.raw_expr {
                                                    return is_safe_default_expr(&raw_expr.node.as_ref().map(|n| Box::new(n.clone())), &non_volatile_funcs);
                                                }
                                            }
                                        }
                                        false
                                    });

                                    if !is_safe_default {
                                        diagnostics.push(
                                            RuleDiagnostic::new(
                                                rule_category!(),
                                                None,
                                                markup! {
                                                    "Adding a column with a volatile default value causes a table rewrite."
                                                },
                                            )
                                            .detail(None, "Even in PostgreSQL 11+, volatile default values require a full table rewrite.")
                                            .note("Add the column without a default, then set the default in a separate statement."),
                                        );
                                    }
                                } else {
                                    // Pre PG 11, all defaults cause rewrites
                                    diagnostics.push(
                                        RuleDiagnostic::new(
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
            }
        }

        diagnostics
    }
}

fn is_safe_default_expr(
    expr: &Option<Box<pgt_query::NodeEnum>>,
    non_volatile_funcs: &HashSet<String>,
) -> bool {
    match expr {
        Some(node) => match node.as_ref() {
            // Constants are always safe
            pgt_query::NodeEnum::AConst(_) => true,
            // Type casts of constants are safe
            pgt_query::NodeEnum::TypeCast(tc) => is_safe_default_expr(
                &tc.arg.as_ref().and_then(|a| a.node.clone()).map(Box::new),
                non_volatile_funcs,
            ),
            // Function calls might be safe if they're non-volatile and have no args
            pgt_query::NodeEnum::FuncCall(fc) => {
                // Must have no args
                if !fc.args.is_empty() {
                    return false;
                }
                // Check if function is in non-volatile list
                if let Some(first_name) = fc.funcname.first() {
                    if let Some(pgt_query::NodeEnum::String(s)) = &first_name.node {
                        return non_volatile_funcs.contains(&s.sval.to_lowercase());
                    }
                }
                false
            }
            // Everything else is potentially unsafe
            _ => false,
        },
        None => false,
    }
}
