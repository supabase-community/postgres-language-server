//! The lint code actions: the fixes of the lint diagnostics, the actions that suppress them, and
//! applying all fixes of a file.

use std::borrow::Cow;

use pgls_analyse::{ActionCategory, Applicability, SUPPRESSION_ACTION_CATEGORY};
use pgls_analyser::{LinterDiagnostic, RuleFix};
use pgls_configuration::RuleSelector;
use pgls_diagnostics::Diagnostic;
use pgls_fs::PgLSPath;
use pgls_text_size::{TextRange, TextSize};
use tracing::warn;

use super::{CodeAction, CodeActionKind, EditAction, TextEdit};
use crate::workspace::Document;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct FixFileParams {
    pub project_key: crate::workspace::ProjectKey,
    pub path: PgLSPath,
    pub fix_file_mode: FixFileMode,
    pub only: Vec<RuleSelector>,
    pub skip: Vec<RuleSelector>,
}

/// Which fixes `fix_file` applies.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum FixFileMode {
    /// Only the fixes that are safe to apply.
    SafeFixes,
    /// The safe and the unsafe fixes.
    SafeAndUnsafeFixes,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Default)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct FixFileResult {
    /// The content of the file with the fixes applied.
    pub code: String,
    /// The number of fixes that were applied.
    pub fixes_applied: u32,
    /// The number of unsafe fixes that were not applied with [FixFileMode::SafeFixes].
    pub skipped_suggested_fixes: u32,
}

/// The fixes of the lint diagnostics at `cursor`, and an action to suppress each of them.
/// `diagnostics` are the lint diagnostics of `doc` that are not suppressed.
pub(crate) fn lint_actions(
    diagnostics: &[LinterDiagnostic],
    doc: &Document,
    cursor: TextSize,
) -> Vec<CodeAction> {
    let mut actions = Vec::new();
    for diagnostic in diagnostics {
        let Some(span) = diagnostic.location().span else {
            continue;
        };
        if !span.contains_inclusive(cursor) {
            continue;
        }
        let category = diagnostic.get_category_name();
        let rule = category.strip_prefix("lint/").unwrap_or(category);

        if let Some(fix) = diagnostic.get_fix() {
            actions.push(CodeAction {
                title: fix.message.clone(),
                kind: CodeActionKind::Edit(EditAction {
                    rule: Some(category.to_string()),
                    category: ActionCategory::QuickFix(Cow::Owned(rule.to_string())),
                    applicability: fix.applicability,
                    edits: fix
                        .edits
                        .iter()
                        .map(|edit| TextEdit {
                            range: edit.range,
                            new_text: edit.replacement.clone(),
                        })
                        .collect(),
                }),
                disabled_reason: None,
            });
        }

        actions.push(CodeAction {
            title: format!("Suppress rule {rule} for this statement"),
            kind: CodeActionKind::Edit(EditAction {
                rule: Some(category.to_string()),
                category: ActionCategory::Other(Cow::Borrowed(SUPPRESSION_ACTION_CATEGORY)),
                applicability: Applicability::MaybeIncorrect,
                edits: vec![suppression_edit(doc, span.start(), rule)],
            }),
            disabled_reason: None,
        });
    }
    actions
}

/// Applies the fixes of the lint diagnostics to `code`, the content of the file at `path`, until
/// no fix is left. `lint` returns the lint diagnostics of a document that are not suppressed.
pub(crate) fn fix_file(
    path: &PgLSPath,
    mut code: String,
    mode: FixFileMode,
    mut lint: impl FnMut(&Document) -> Vec<LinterDiagnostic>,
) -> FixFileResult {
    /// Fixes can enable other fixes, e.g. `CONCURRENTLY` enables `IF NOT EXISTS`, and fixes
    /// that overlap are applied one after the other.
    const MAX_PASSES: usize = 10;

    let mut fixes_applied = 0;
    let mut skipped_suggested_fixes = 0;
    let mut converged = false;
    for _ in 0..MAX_PASSES {
        let doc = Document::new(code.clone(), 0);
        let diagnostics = lint(&doc);

        skipped_suggested_fixes = 0;
        let mut fixes: Vec<&RuleFix> = Vec::new();
        for fix in diagnostics.iter().filter_map(LinterDiagnostic::get_fix) {
            if fix.applicability == Applicability::Always || mode == FixFileMode::SafeAndUnsafeFixes
            {
                fixes.push(fix);
            } else {
                skipped_suggested_fixes += 1;
            }
        }
        fixes.sort_by_key(|fix| fix.range().map(TextRange::start));

        // Applies the fixes that don't touch the text of an earlier fix in this pass.
        let mut combined = RuleFix::new("");
        let mut applied = 0;
        for fix in fixes {
            let Some(range) = fix.range() else {
                continue;
            };
            if combined
                .range()
                .is_some_and(|previous| range.start() <= previous.end())
            {
                continue;
            }
            combined.edits.extend(fix.edits.iter().cloned());
            applied += 1;
        }
        if applied == 0 {
            converged = true;
            break;
        }
        code = combined.apply(&code);
        fixes_applied += applied;
    }
    if !converged {
        warn!(
            ?path,
            "Fixes still changed the file after {MAX_PASSES} passes, some may be left"
        );
    }

    FixFileResult {
        code,
        fixes_applied,
        skipped_suggested_fixes,
    }
}

/// The edit that adds `-- pgls-ignore <rule>` on its own line above the statement at `offset`,
/// with the statement's indentation and the file's line ending. Above the statement, the comment
/// suppresses the rule for all of the statement, including the bodies of its functions.
fn suppression_edit(doc: &Document, offset: TextSize, rule: &str) -> TextEdit {
    let content = doc.get_document_content();
    let start = doc
        .statement_ranges()
        .iter()
        .find(|statement| statement.contains_inclusive(offset))
        .map_or(offset, |statement| statement.start());
    let line_ending = match content.find('\n') {
        Some(newline) if content[..newline].ends_with('\r') => "\r\n",
        _ => "\n",
    };

    let start = usize::from(start);
    let line_start = content[..start]
        .rfind('\n')
        .map_or(0, |newline| newline + 1);
    let before = &content[line_start..start];
    let indentation: String = before
        .chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .collect();
    let offset = |offset: usize| TextSize::try_from(offset).unwrap();
    if before.trim().is_empty() {
        TextEdit {
            range: TextRange::empty(offset(line_start)),
            new_text: format!("{indentation}-- pgls-ignore {rule}{line_ending}"),
        }
    } else {
        // The statement starts after another one on the same line, so it moves to a line of its
        // own.
        let previous_end = line_start + before.trim_end().len();
        TextEdit {
            range: TextRange::new(offset(previous_end), offset(start)),
            new_text: format!(
                "{line_ending}{indentation}-- pgls-ignore {rule}{line_ending}{indentation}"
            ),
        }
    }
}
