//! The command that drops the cached database schemas, so that they are loaded again.

use super::{CodeAction, CodeActionKind, CommandAction, CommandActionCategory};

/// The action that invalidates the schema cache. `disabled_reason` says why it isn't available,
/// if it isn't.
pub(crate) fn invalidate_schema_cache_action(disabled_reason: Option<String>) -> CodeAction {
    CodeAction {
        title: "Invalidate Schema Cache".into(),
        kind: CodeActionKind::Command(CommandAction {
            category: CommandActionCategory::InvalidateSchemaCache,
        }),
        disabled_reason,
    }
}
