use crate::{LinterDiagnostic, LinterRule, LinterRuleContext, RuleFix};
use pgls_analyse::{FixKind, RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::protobuf::Token;

declare_lint_rule! {
    /// Prefer statements with guards for robustness in migrations.
    ///
    /// When running migrations outside of transactions (e.g., CREATE INDEX CONCURRENTLY),
    /// statements should be made robust by using guards like IF NOT EXISTS or IF EXISTS.
    /// This allows migrations to be safely re-run if they fail partway through.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// CREATE INDEX CONCURRENTLY users_email_idx ON users (email);
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// DROP INDEX CONCURRENTLY users_email_idx;
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// CREATE TABLE users (id int);
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// DROP TABLE users;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// CREATE INDEX CONCURRENTLY IF NOT EXISTS users_email_idx ON users (email);
    /// ```
    ///
    /// ```sql
    /// DROP INDEX CONCURRENTLY IF EXISTS users_email_idx;
    /// ```
    ///
    /// ```sql
    /// CREATE TABLE IF NOT EXISTS users (id int);
    /// ```
    ///
    /// ```sql
    /// DROP TABLE IF EXISTS users;
    /// ```
    ///
    pub PreferRobustStmts {
        version: "0.15.0",
        name: "preferRobustStmts",
        severity: Severity::Warning,
        fix_kind: FixKind::Unsafe,
        recommended: false,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Squawk("prefer-robust-stmts")],
    }
}

impl LinterRule for PreferRobustStmts {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let mut diagnostics = Vec::new();

        // Since we assume we're always in a transaction, we only check for
        // statements that explicitly run outside transactions
        match &ctx.stmt() {
            pgls_query::NodeEnum::IndexStmt(stmt) => {
                // Concurrent index creation runs outside transaction
                if stmt.concurrent {
                    // Check for unnamed index
                    if stmt.idxname.is_empty() {
                        diagnostics.push(LinterDiagnostic::new(
                            rule_category!(),
                            None,
                            markup! {
                                "Concurrent index should have an explicit name."
                            },
                        ).detail(None, "Use an explicit name for a concurrently created index to make migrations more robust."));
                    }
                    // Check for IF NOT EXISTS
                    if !stmt.if_not_exists {
                        diagnostics.push(
                            LinterDiagnostic::new(
                                rule_category!(),
                                None,
                                markup! {
                                    "Concurrent index creation should use IF NOT EXISTS."
                                },
                            )
                            .detail(
                                None,
                                "Add IF NOT EXISTS to make the migration re-runnable if it fails.",
                            )
                            // `IF NOT EXISTS` requires an index name.
                            .maybe_fix(if stmt.idxname.is_empty() {
                                None
                            } else {
                                insert_after(ctx, Token::Concurrently, "IF NOT EXISTS")
                            }),
                        );
                    }
                }
            }
            pgls_query::NodeEnum::DropStmt(stmt) => {
                if stmt.concurrent && !stmt.missing_ok {
                    diagnostics.push(
                        LinterDiagnostic::new(
                            rule_category!(),
                            None,
                            markup! {
                                "Concurrent drop should use IF EXISTS."
                            },
                        )
                        .detail(
                            None,
                            "Add IF EXISTS to make the migration re-runnable if it fails.",
                        )
                        .maybe_fix(insert_after(
                            ctx,
                            Token::Concurrently,
                            "IF EXISTS",
                        )),
                    );
                }
                if stmt.remove_type() == pgls_query::protobuf::ObjectType::ObjectTable
                    && !stmt.missing_ok
                {
                    diagnostics.push(
                        LinterDiagnostic::new(
                            rule_category!(),
                            None,
                            markup! {
                                <Emphasis>"DROP TABLE"</Emphasis>" should use "<Emphasis>"IF EXISTS"</Emphasis>"."
                            },
                        )
                        .detail(
                            None,
                            "Add IF EXISTS to make the migration re-runnable if it fails.",
                        )
                        .maybe_fix(insert_after(ctx, Token::Table, "IF EXISTS")),
                    );
                }
            }
            pgls_query::NodeEnum::CreateStmt(stmt) if !stmt.if_not_exists => {
                diagnostics.push(
                        LinterDiagnostic::new(
                            rule_category!(),
                            None,
                            markup! {
                                <Emphasis>"CREATE TABLE"</Emphasis>" should use "<Emphasis>"IF NOT EXISTS"</Emphasis>"."
                            },
                        )
                        .detail(
                            None,
                            "Add IF NOT EXISTS to make the migration re-runnable if it fails.",
                        )
                        .maybe_fix(insert_after(ctx, Token::Table, "IF NOT EXISTS")),
                    );
            }
            _ => {}
        }

        diagnostics
    }
}

/// Inserts `IF [NOT] EXISTS` after the first `keyword` of the statement. It is unsafe because the
/// statement then silently does nothing if the object already exists, or doesn't exist.
fn insert_after(
    ctx: &LinterRuleContext<PreferRobustStmts>,
    keyword: Token,
    text: &str,
) -> Option<RuleFix> {
    let tokens = ctx.tokens()?;
    let index = tokens.find(keyword)?;
    Some(tokens.insert_after(index, &format!("Add {text}"), text))
}
