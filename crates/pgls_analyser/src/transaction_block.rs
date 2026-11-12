//! The diagnostic for statements that Postgres can't run in a transaction block, shared by
//! `banConcurrentIndexCreationInTransaction` and `banNonTransactionalStatementInTransaction`.

use pgls_catalog::{NonTransactional, prevents_transaction_block};
use pgls_console::markup;
use pgls_diagnostics::Category;

use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};

/// The statement, if Postgres can't run it in a transaction block, see
/// [`prevents_transaction_block`].
pub(crate) fn non_transactional_statement<R: LinterRule + 'static>(
    ctx: &LinterRuleContext<R>,
) -> Option<NonTransactional> {
    prevents_transaction_block(ctx.stmt(), ctx.catalog(), ctx.session().search_path())
}

/// The diagnostic for `statement`, if it runs in a transaction block or in a function body.
///
/// A statement is reported where it surely runs in a transaction block: between `BEGIN` and
/// `COMMIT`, in a function body, which runs in the transaction of its caller, and in a migration
/// section that runs in a transaction or as one query with several statements.
///
/// A statement with `CONCURRENTLY` is also reported where we only assume a transaction block: in
/// a file with several statements and without migration sections, which migration tools usually
/// run in a transaction. [`crate::fix::add_concurrently`] makes the same assumption, so the fixes
/// that add `CONCURRENTLY` never write a statement that is reported here. Other statements, like
/// `VACUUM`, are common in maintenance scripts that psql runs one statement at a time, so they
/// are only reported where the transaction block is known.
pub(crate) fn transaction_block_diagnostic<R: LinterRule + 'static>(
    ctx: &LinterRuleContext<R>,
    category: &'static Category,
    statement: NonTransactional,
) -> Option<LinterDiagnostic> {
    let name = statement.statement;
    if ctx.function().is_some() {
        return Some(
            LinterDiagnostic::new(
                category,
                None,
                markup! {
                    {name}" cannot be executed from a function."
                },
            )
            .detail(
                None,
                "The statements of a function run in the transaction of the statement that calls it. Run the statement outside of the function.",
            ),
        );
    }

    let assumed = !ctx.knows_transaction();
    if !ctx.in_transaction() || (assumed && !statement.concurrently) {
        return None;
    }
    let diagnostic = LinterDiagnostic::new(
        category,
        None,
        markup! {
            {name}" cannot run inside a transaction block."
        },
    )
    .detail(
        None,
        format!("Run {name} outside of a transaction block, e.g. in a migration of its own that runs without a transaction."),
    );
    let session = ctx.session();
    Some(if assumed {
        diagnostic.note("Migration tools usually run each migration in a transaction, so we assume that the file runs in one.")
    } else if session.in_implicit_block() && !session.in_explicit_transaction() {
        diagnostic.note("The migration tool sends the statements to Postgres as one query, and Postgres runs a query with more than one statement in a transaction block. Put the statement in a section of its own.")
    } else {
        diagnostic
    })
}
