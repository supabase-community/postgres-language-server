//! The code actions offered at the cursor, and the commands and fixes they run: lint fixes and
//! suppressions (`lint`), expanding a `*` into its columns (`expand_star`), executing the
//! statement (`execute_statement`), and invalidating the schema cache (`schema_cache`).

use crate::workspace::StatementId;
use pgls_analyse::Applicability;
use pgls_configuration::RuleSelector;
use pgls_fs::PgLSPath;
use pgls_text_size::{TextRange, TextSize};
use std::borrow::Cow;

pub(crate) mod execute_statement;
pub(crate) mod expand_star;
pub(crate) mod lint;
pub(crate) mod schema_cache;

pub use execute_statement::{
    EXECUTE_STATEMENT_MAX_RESULT_SIZE, EXECUTE_STATEMENT_MAX_ROWS,
    EXECUTE_STATEMENT_MAX_VALUE_SIZE, ExecuteStatementError, ExecuteStatementParams,
    ExecuteStatementResult,
};
pub use lint::{FixFileMode, FixFileParams, FixFileResult};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CodeActionsParams {
    pub project_key: crate::workspace::ProjectKey,
    pub path: PgLSPath,
    pub cursor_position: TextSize,
    pub only: Vec<RuleSelector>,
    pub skip: Vec<RuleSelector>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CodeActionsResult {
    pub actions: Vec<CodeAction>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CodeAction {
    pub title: String,
    pub kind: CodeActionKind,
    pub disabled_reason: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum CodeActionKind {
    Edit(EditAction),
    Command(CommandAction),
    EditAndCommand(EditAction, CommandAction),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EditAction {
    /// The category of the diagnostic the action fixes or suppresses, e.g. `lint/banDropTable`.
    /// `None` for refactors.
    pub rule: Option<String>,
    pub category: ActionCategory,
    pub applicability: Applicability,
    /// Non-overlapping edits of the file, ordered by their position.
    pub edits: Vec<TextEdit>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TextEdit {
    pub range: TextRange,
    pub new_text: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CommandAction {
    pub category: CommandActionCategory,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, strum::EnumIter)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum CommandActionCategory {
    ExecuteStatement(StatementId),
    InvalidateSchemaCache,
}

/// Actions that suppress rules should start with this string
pub const SUPPRESSION_ACTION_CATEGORY: &str = "quickfix.suppressRule";

/// The category of a code action, this type maps directly to the
/// [CodeActionKind] type in the Language Server Protocol specification
///
/// [CodeActionKind]: https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#codeActionKind
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ActionCategory {
    /// Base kind for quickfix actions: 'quickfix'.
    ///
    /// This action provides a fix to the diagnostic emitted by the same signal
    QuickFix(Cow<'static, str>),
    /// Base kind for refactoring actions: 'refactor'.
    ///
    /// This action provides an optional refactor opportunity
    Refactor(RefactorKind),
    /// Base kind for source actions: `source`.
    ///
    /// Source code actions apply to the entire file.
    Source(SourceActionKind),
    /// This action is using a base kind not covered by any of the previous
    /// variants
    Other(Cow<'static, str>),
}

impl ActionCategory {
    /// Returns true if this category matches the provided filter
    pub fn matches(&self, filter: &str) -> bool {
        self.to_str().starts_with(filter)
    }

    /// Returns the representation of this [ActionCategory] as a `CodeActionKind` string
    pub fn to_str(&self) -> Cow<'static, str> {
        match self {
            ActionCategory::QuickFix(tag) => {
                if tag.is_empty() {
                    Cow::Borrowed("quickfix.pgls")
                } else {
                    Cow::Owned(format!("quickfix.pgls.{tag}"))
                }
            }

            ActionCategory::Refactor(RefactorKind::None) => Cow::Borrowed("refactor.pgls"),
            ActionCategory::Refactor(RefactorKind::Extract) => {
                Cow::Borrowed("refactor.extract.pgls")
            }
            ActionCategory::Refactor(RefactorKind::Inline) => Cow::Borrowed("refactor.inline.pgls"),
            ActionCategory::Refactor(RefactorKind::Rewrite) => {
                Cow::Borrowed("refactor.rewrite.pgls")
            }
            ActionCategory::Refactor(RefactorKind::Other(tag)) => {
                Cow::Owned(format!("refactor.{tag}.pgls"))
            }

            ActionCategory::Source(SourceActionKind::None) => Cow::Borrowed("source.pgls"),
            ActionCategory::Source(SourceActionKind::FixAll) => Cow::Borrowed("source.fixAll.pgls"),
            ActionCategory::Source(SourceActionKind::OrganizeImports) => {
                Cow::Borrowed("source.organizeImports.pgls")
            }
            ActionCategory::Source(SourceActionKind::Other(tag)) => {
                Cow::Owned(format!("source.{tag}.pgls"))
            }

            ActionCategory::Other(tag) => Cow::Owned(format!("{tag}.pgls")),
        }
    }
}

/// The sub-category of a refactor code action.
///
/// [Check the LSP spec](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#codeActionKind) for more information:
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum RefactorKind {
    /// This action describes a refactor with no particular sub-category
    None,
    /// Base kind for refactoring extraction actions: 'refactor.extract'.
    ///
    /// Example extract actions:
    /// - Extract method
    /// - Extract function
    /// - Extract variable
    /// - Extract interface from class
    Extract,
    /// Base kind for refactoring inline actions: 'refactor.inline'.
    ///
    /// Example inline actions:
    /// - Inline function
    /// - Inline variable
    /// - Inline constant
    /// - ...
    Inline,
    /// Base kind for refactoring rewrite actions: 'refactor.rewrite'.
    ///
    /// Example rewrite actions:
    /// - Convert JavaScript function to class
    /// - Add or remove parameter
    /// - Encapsulate field
    /// - Make method static
    /// - Move method to base class
    /// - ...
    Rewrite,
    /// This action is using a refactor kind not covered by any of the previous
    /// variants
    Other(Cow<'static, str>),
}

/// The sub-category of a source code action
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum SourceActionKind {
    /// This action describes a source action with no particular sub-category
    None,
    // Base kind for a 'fix all' source action: `source.fixAll`.
    //
    // 'Fix all' actions automatically fix errors that have a clear fix that
    // do not require user input. They should not suppress errors or perform
    // unsafe fixes such as generating new types or classes.
    FixAll,
    /// Base kind for an organize imports source action: `source.organizeImports`.
    OrganizeImports,
    /// This action is using a source action kind not covered by any of the
    /// previous variants
    Other(Cow<'static, str>),
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::{ActionCategory, RefactorKind};

    #[test]
    fn action_categories_match_their_parent_kinds() {
        assert!(ActionCategory::QuickFix(Cow::from("quickfix")).matches("quickfix"));

        assert!(ActionCategory::Refactor(RefactorKind::None).matches("refactor"));
        assert!(!ActionCategory::Refactor(RefactorKind::None).matches("refactor.extract"));

        assert!(ActionCategory::Refactor(RefactorKind::Extract).matches("refactor"));
        assert!(ActionCategory::Refactor(RefactorKind::Extract).matches("refactor.extract"));
    }
}
