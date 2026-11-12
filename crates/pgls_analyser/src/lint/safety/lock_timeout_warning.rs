use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Taking a dangerous lock without setting a lock timeout can cause indefinite blocking.
    ///
    /// When a statement acquires a lock that would block common operations (like SELECT, INSERT, UPDATE, DELETE),
    /// it can cause the database to become unresponsive if another transaction is holding a conflicting lock
    /// while idle in transaction or active. This is particularly dangerous for:
    ///
    /// - ALTER TABLE statements (acquire ACCESS EXCLUSIVE lock for most subcommands, and SHARE ROW
    ///   EXCLUSIVE to enable or disable triggers and to add foreign keys)
    /// - CREATE INDEX without CONCURRENTLY (acquires SHARE lock)
    ///
    /// Subcommands that only take a SHARE UPDATE EXCLUSIVE lock, like `VALIDATE CONSTRAINT`, don't
    /// block reads or writes and aren't reported.
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
        version: "0.17.0",
        name: "lockTimeoutWarning",
        severity: Severity::Warning,
        recommended: true,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Eugene("E9")],
    }
}

impl LinterRule for LockTimeoutWarning {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let session = ctx.session();
        if session.has_lock_timeout() {
            return vec![];
        }

        // The strongest lock that blocks writes, on a table the transaction didn't create.
        let Some(lock) = session
            .table_locks(ctx.stmt(), ctx.catalog())
            .into_iter()
            .filter(|lock| lock.mode.blocks_writes())
            .reduce(|strongest, lock| {
                if lock.mode > strongest.mode {
                    lock
                } else {
                    strongest
                }
            })
        else {
            return vec![];
        };
        let full_name = format!("{}.{}", lock.schema, lock.name);
        let mode = lock.mode.name();
        let detail = if lock.mode.blocks_reads() {
            "This can block all operations on the table indefinitely if another transaction holds a conflicting lock."
        } else {
            "This blocks writes to the table indefinitely if another transaction holds a conflicting lock."
        };

        let diagnostic = match ctx.stmt() {
            pgls_query::NodeEnum::IndexStmt(stmt) => {
                let index_name = &stmt.idxname;
                LinterDiagnostic::new(
                    rule_category!(),
                    None,
                    markup! {
                        "Statement takes "{mode}" lock on "<Emphasis>{full_name}</Emphasis>" while creating index "<Emphasis>{index_name}</Emphasis>" without lock timeout set."
                    },
                )
                .detail(None, detail)
                .note("Run 'SET LOCAL lock_timeout = '2s';' before this statement, or use CREATE INDEX CONCURRENTLY to avoid blocking writes.")
            }
            _ => LinterDiagnostic::new(
                rule_category!(),
                None,
                markup! {
                    "Statement takes "{mode}" lock on "<Emphasis>{full_name}</Emphasis>" without lock timeout set."
                },
            )
            .detail(None, detail)
            .note("Run 'SET LOCAL lock_timeout = '2s';' before this statement and retry the migration if it times out."),
        };
        vec![diagnostic]
    }
}
