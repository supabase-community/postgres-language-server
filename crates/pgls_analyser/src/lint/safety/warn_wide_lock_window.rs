use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Acquiring ACCESS EXCLUSIVE locks on multiple tables widens the lock window.
    ///
    /// When a transaction holds an ACCESS EXCLUSIVE lock on one table and acquires
    /// another on a different table, both locks are held until the transaction commits.
    /// This increases the chance of blocking concurrent operations and causing downtime.
    ///
    /// Split the operations into separate transactions to minimize the lock window.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// Acquiring locks on multiple tables in the same transaction:
    /// ```sql
    /// ALTER TABLE users ADD COLUMN email TEXT;
    /// ALTER TABLE orders ADD COLUMN total NUMERIC;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// select 1;
    /// ```
    ///
    pub WarnWideLockWindow {
        version: "next",
        name: "warnWideLockWindow",
        severity: Severity::Warning,
        recommended: true,
        sources: &[RuleSource::Pgfence("wide-lock-window")],
    }
}

impl LinterRule for WarnWideLockWindow {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let mut diagnostics = vec![];

        let tx_state = ctx.file_context().transaction_state();
        let existing_tables = tx_state.access_exclusive_tables();

        if existing_tables.is_empty() {
            return diagnostics;
        }

        let current_table = match ctx.stmt() {
            pgls_query::NodeEnum::AlterTableStmt(stmt) => stmt.relation.as_ref().map(|r| {
                let schema = if r.schemaname.is_empty() {
                    "public".to_string()
                } else {
                    r.schemaname.clone()
                };
                (schema, r.relname.clone())
            }),
            pgls_query::NodeEnum::IndexStmt(stmt) if !stmt.concurrent => {
                stmt.relation.as_ref().map(|r| {
                    let schema = if r.schemaname.is_empty() {
                        "public".to_string()
                    } else {
                        r.schemaname.clone()
                    };
                    (schema, r.relname.clone())
                })
            }
            _ => None,
        };

        if let Some((schema, name)) = current_table {
            if tx_state.has_created_object(&schema, &name) {
                return diagnostics;
            }

            let is_new_table = !existing_tables
                .iter()
                .any(|(s, n)| s == &schema && n == &name);

            if is_new_table {
                let full_name = format!("{schema}.{name}");
                diagnostics.push(
                    LinterDiagnostic::new(
                        rule_category!(),
                        None,
                        markup! {
                            "Acquiring a lock on "<Emphasis>{full_name}</Emphasis>" while already holding ACCESS EXCLUSIVE locks on other tables."
                        },
                    )
                    .detail(
                        None,
                        "This widens the lock window. Split into separate transactions to minimize lock duration.",
                    ),
                );
            }
        }

        diagnostics
    }
}
