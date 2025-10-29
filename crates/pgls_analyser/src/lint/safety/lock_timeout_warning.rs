use pgls_analyse::{Rule, RuleDiagnostic, RuleSource, context::RuleContext, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Taking a dangerous lock without setting a lock timeout can cause indefinite blocking.
    ///
    /// When a statement acquires a lock that would block common operations (like SELECT, INSERT, UPDATE, DELETE),
    /// it can cause the database to become unresponsive if another transaction is holding a conflicting lock
    /// while idle in transaction or active. This is particularly dangerous for:
    ///
    /// - ALTER TABLE statements (acquire ACCESS EXCLUSIVE lock)
    /// - CREATE INDEX without CONCURRENTLY (acquires SHARE lock)
    ///
    /// Setting a lock timeout ensures that if the lock cannot be acquired within a reasonable time,
    /// the statement will fail rather than blocking indefinitely.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// ALTER TABLE users ADD COLUMN email TEXT;
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// CREATE INDEX users_email_idx ON users(email);
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// -- Use CONCURRENTLY to avoid taking dangerous locks
    /// CREATE INDEX CONCURRENTLY users_email_idx ON users(email);
    /// ```
    ///
    pub LockTimeoutWarning {
        version: "next",
        name: "lockTimeoutWarning",
        severity: Severity::Warning,
        recommended: true,
        sources: &[RuleSource::Eugene("E9")],
    }
}

impl Rule for LockTimeoutWarning {
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Vec<RuleDiagnostic> {
        let mut diagnostics = Vec::new();

        // Check if lock timeout has been set in the transaction
        let tx_state = ctx.file_context().transaction_state();
        if tx_state.has_lock_timeout() {
            return diagnostics;
        }

        // Check if this statement takes a dangerous lock on an existing object
        match ctx.stmt() {
            // ALTER TABLE takes ACCESS EXCLUSIVE lock
            pgls_query::NodeEnum::AlterTableStmt(stmt) => {
                if let Some(relation) = &stmt.relation {
                    let schema = if relation.schemaname.is_empty() {
                        "public"
                    } else {
                        &relation.schemaname
                    };
                    let table = &relation.relname;

                    // Only warn if the table wasn't created in this transaction
                    if !tx_state.has_created_object(schema, table) {
                        let full_name = format!("{schema}.{table}");
                        diagnostics.push(
                            RuleDiagnostic::new(
                                rule_category!(),
                                None,
                                markup! {
                                    "Statement takes ACCESS EXCLUSIVE lock on "<Emphasis>{full_name}</Emphasis>" without lock timeout set."
                                },
                            )
                            .detail(None, "This can block all operations on the table indefinitely if another transaction holds a conflicting lock.")
                            .note("Run 'SET LOCAL lock_timeout = '2s';' before this statement and retry the migration if it times out."),
                        );
                    }
                }
            }

            // CREATE INDEX without CONCURRENTLY takes SHARE lock
            pgls_query::NodeEnum::IndexStmt(stmt) => {
                if !stmt.concurrent {
                    if let Some(relation) = &stmt.relation {
                        let schema = if relation.schemaname.is_empty() {
                            "public"
                        } else {
                            &relation.schemaname
                        };
                        let table = &relation.relname;

                        // Only warn if the table wasn't created in this transaction
                        if !tx_state.has_created_object(schema, table) {
                            let full_name = format!("{schema}.{table}");
                            let index_name = &stmt.idxname;
                            diagnostics.push(
                                RuleDiagnostic::new(
                                    rule_category!(),
                                    None,
                                    markup! {
                                        "Statement takes SHARE lock on "<Emphasis>{full_name}</Emphasis>" while creating index "<Emphasis>{index_name}</Emphasis>" without lock timeout set."
                                    },
                                )
                                .detail(None, "This blocks writes to the table indefinitely if another transaction holds a conflicting lock.")
                                .note("Run 'SET LOCAL lock_timeout = '2s';' before this statement, or use CREATE INDEX CONCURRENTLY to avoid blocking writes."),
                            );
                        }
                    }
                }
            }

            _ => {}
        }

        diagnostics
    }
}
