use std::collections::HashMap;

use crate::{
    adapters::{get_cursor_position, to_lsp},
    session::Session,
};
use anyhow::{Result, anyhow};
use pgls_analyse::{Applicability, SUPPRESSION_ACTION_CATEGORY};
use tower_lsp_server::jsonrpc;
use tower_lsp_server::ls_types::{
    self, CodeAction, CodeActionDisabled, CodeActionOrCommand, Command, ExecuteCommandParams,
    MessageType, WorkspaceEdit,
};

use pgls_text_size::{TextRange, TextSize};
use pgls_workspace::features::code_actions::{
    CodeActionKind, CodeActionsParams, CommandActionCategory, ExecuteStatementParams, FixFileMode,
    FixFileParams,
};
use pgls_workspace::workspace::GetFileContentParams;

#[tracing::instrument(level = "debug", skip(session), err)]
pub fn get_actions(
    session: &Session,
    params: ls_types::CodeActionParams,
) -> Result<ls_types::CodeActionResponse> {
    let url = params.text_document.uri;
    let path = session.file_path(&url)?;
    let project_key = session.project_key(&path);

    // On save, editors only ask for the fix-all action, which doesn't need the other actions.
    let requested = params
        .context
        .only
        .as_deref()
        .filter(|kinds| !kinds.is_empty());
    let fix_all = requested
        .is_some_and(|kinds| kinds.iter().any(|kind| covers(kind, FIX_ALL_KIND)))
        .then(|| fix_all_action(session, &url, path.clone()))
        .transpose()?
        .flatten();
    if requested.is_some_and(|kinds| kinds.iter().all(|kind| covers(kind, FIX_ALL_KIND))) {
        return Ok(fix_all
            .into_iter()
            .map(CodeActionOrCommand::CodeAction)
            .collect());
    }

    let cursor_position = get_cursor_position(session, &url, params.range.start)?;

    let workspace_actions = session.workspace.pull_code_actions(CodeActionsParams {
        project_key,
        path,
        cursor_position,
        only: vec![],
        skip: vec![],
    })?;

    let actions: Vec<CodeAction> = workspace_actions
        .actions
        .into_iter()
        .filter_map(|action| match action.kind {
            CodeActionKind::Command(command) => {
                let command_id: String = command_id(&command.category);
                let title = action.title;

                match command.category {
                    CommandActionCategory::ExecuteStatement(stmt_id) => Some(CodeAction {
                        title: title.clone(),
                        kind: Some(ls_types::CodeActionKind::EMPTY),
                        command: Some({
                            Command {
                                title: title.clone(),
                                command: command_id,
                                arguments: Some(vec![
                                    serde_json::to_value(&stmt_id).unwrap(),
                                    serde_json::to_value(&url).unwrap(),
                                ]),
                            }
                        }),
                        disabled: action
                            .disabled_reason
                            .map(|reason| CodeActionDisabled { reason }),
                        ..Default::default()
                    }),
                    CommandActionCategory::InvalidateSchemaCache => Some(CodeAction {
                        title: title.clone(),
                        kind: Some(ls_types::CodeActionKind::EMPTY),
                        command: Some({
                            Command {
                                title: title.clone(),
                                command: command_id,
                                arguments: None,
                            }
                        }),
                        disabled: action
                            .disabled_reason
                            .map(|reason| CodeActionDisabled { reason }),
                        ..Default::default()
                    }),
                }
            }

            CodeActionKind::Edit(edit) => {
                let line_index = &session.document(&url).ok()?.line_index;
                let edits = edit
                    .edits
                    .iter()
                    .map(|text_edit| {
                        Some(ls_types::TextEdit {
                            range: to_lsp::range(
                                line_index,
                                text_edit.range,
                                session.position_encoding(),
                            )
                            .ok()?,
                            new_text: text_edit.new_text.clone(),
                        })
                    })
                    .collect::<Option<Vec<_>>>()?;
                let diagnostics: Vec<_> = params
                    .context
                    .diagnostics
                    .iter()
                    .filter(|diagnostic| {
                        edit.rule.as_ref().is_some_and(|rule| {
                            diagnostic.code == Some(ls_types::NumberOrString::String(rule.clone()))
                        })
                    })
                    .cloned()
                    .collect();
                let is_fix =
                    edit.rule.is_some() && !edit.category.matches(SUPPRESSION_ACTION_CATEGORY);

                Some(CodeAction {
                    title: action.title,
                    kind: Some(ls_types::CodeActionKind::from(
                        edit.category.to_str().into_owned(),
                    )),
                    diagnostics: (!diagnostics.is_empty()).then_some(diagnostics),
                    edit: Some(workspace_edit(session, &url, edits)),
                    is_preferred: (is_fix && edit.applicability == Applicability::Always)
                        .then_some(true),
                    disabled: action
                        .disabled_reason
                        .map(|reason| CodeActionDisabled { reason }),
                    ..Default::default()
                })
            }

            CodeActionKind::EditAndCommand(..) => None,
        })
        .collect();

    Ok(actions
        .into_iter()
        .chain(fix_all)
        .map(CodeActionOrCommand::CodeAction)
        .collect())
}

/// The kind of the action that applies all safe fixes, e.g. on save with
/// `"editor.codeActionsOnSave": { "source.fixAll.pgls": "explicit" }`.
const FIX_ALL_KIND: &str = "source.fixAll.pgls";

/// Whether the requested code action kind `requested` includes `kind`, e.g. `source` includes
/// `source.fixAll.pgls`.
fn covers(requested: &ls_types::CodeActionKind, kind: &str) -> bool {
    let requested = requested.as_str();
    kind == requested
        || kind
            .strip_prefix(requested)
            .is_some_and(|rest| rest.starts_with('.'))
}

/// The action that applies all safe fixes of the document, if there are any.
fn fix_all_action(
    session: &Session,
    url: &ls_types::Uri,
    path: pgls_fs::PgLSPath,
) -> Result<Option<CodeAction>> {
    let Ok(doc) = session.document(url) else {
        return Ok(None);
    };
    let project_key = session.project_key(&path);
    let content = session.workspace.get_file_content(GetFileContentParams {
        project_key,
        path: path.clone(),
    })?;
    let fixed = session.workspace.fix_file(FixFileParams {
        project_key,
        path,
        fix_file_mode: FixFileMode::SafeFixes,
        only: vec![],
        skip: vec![],
    })?;
    if fixed.fixes_applied == 0 || fixed.code == content {
        return Ok(None);
    }

    // Replaces only what changed, so that the editor keeps the cursor and the folds.
    let (range, new_text) = changed_range(&content, &fixed.code);
    let edit = ls_types::TextEdit {
        range: to_lsp::range(&doc.line_index, range, session.position_encoding())?,
        new_text: new_text.to_string(),
    };
    Ok(Some(CodeAction {
        title: "Apply all safe fixes".into(),
        kind: Some(ls_types::CodeActionKind::from(FIX_ALL_KIND)),
        edit: Some(workspace_edit(session, url, vec![edit])),
        ..Default::default()
    }))
}

/// The range of `old` that differs from `new`, and the text that replaces it.
fn changed_range<'a>(old: &str, new: &'a str) -> (TextRange, &'a str) {
    let prefix = old
        .char_indices()
        .zip(new.chars())
        .find(|((_, a), b)| a != b)
        .map_or(old.len().min(new.len()), |((index, _), _)| index);
    let suffix = old[prefix..]
        .chars()
        .rev()
        .zip(new[prefix..].chars().rev())
        .take_while(|(a, b)| a == b)
        .map(|(a, _)| a.len_utf8())
        .sum::<usize>();
    let size = |offset: usize| TextSize::try_from(offset).unwrap();
    (
        TextRange::new(size(prefix), size(old.len() - suffix)),
        &new[prefix..new.len() - suffix],
    )
}

/// An edit of the document at `url`. If the client supports it, the edit names the version of
/// the document it applies to, so that the client rejects it if the document changed since.
fn workspace_edit(
    session: &Session,
    url: &ls_types::Uri,
    edits: Vec<ls_types::TextEdit>,
) -> WorkspaceEdit {
    let versioned = session
        .client_capabilities()
        .and_then(|capabilities| capabilities.workspace.as_ref())
        .and_then(|workspace| workspace.workspace_edit.as_ref())
        .and_then(|workspace_edit| workspace_edit.document_changes)
        == Some(true);
    match session.document(url) {
        Ok(doc) if versioned => WorkspaceEdit {
            document_changes: Some(ls_types::DocumentChanges::Edits(vec![
                ls_types::TextDocumentEdit {
                    text_document: ls_types::OptionalVersionedTextDocumentIdentifier {
                        uri: url.clone(),
                        version: Some(doc.version),
                    },
                    edits: edits.into_iter().map(ls_types::OneOf::Left).collect(),
                },
            ])),
            ..Default::default()
        },
        _ => WorkspaceEdit {
            changes: Some(HashMap::from([(url.clone(), edits)])),
            ..Default::default()
        },
    }
}

pub fn command_id(command: &CommandActionCategory) -> String {
    match command {
        CommandActionCategory::ExecuteStatement(_) => "pgls.executeStatement".into(),
        CommandActionCategory::InvalidateSchemaCache => "pgls.invalidateSchemaCache".into(),
    }
}

/// The statement and the document that `pgls.executeStatement` gets from [`get_actions`].
fn execute_statement_arguments(
    arguments: &[serde_json::Value],
) -> Result<(pgls_workspace::workspace::StatementId, ls_types::Uri), jsonrpc::Error> {
    let invalid = |message: String| {
        jsonrpc::Error::invalid_params(format!(
            "Invalid arguments for pgls.executeStatement: {message}"
        ))
    };
    let [statement_id, uri] = arguments else {
        return Err(invalid(format!(
            "expected a statement and a document URI, got {} arguments",
            arguments.len()
        )));
    };
    let statement_id = serde_json::from_value(statement_id.clone())
        .map_err(|error| invalid(format!("the statement: {error}")))?;
    let uri = serde_json::from_value(uri.clone())
        .map_err(|error| invalid(format!("the document URI: {error}")))?;
    Ok((statement_id, uri))
}

#[tracing::instrument(level = "debug", skip(session), err)]
pub async fn execute_command(
    session: &Session,
    params: ExecuteCommandParams,
) -> anyhow::Result<Option<serde_json::Value>> {
    let command = params.command;

    match command.as_str() {
        "pgls.executeStatement" => {
            let (statement_id, doc_url) = execute_statement_arguments(&params.arguments)?;

            let path = session.file_path(&doc_url)?;
            let project_key = session.project_key(&path);

            let result = session
                .workspace
                .execute_statement(ExecuteStatementParams {
                    project_key,
                    statement_id,
                    path,
                })?;

            /*
             * Updating all diagnostics: the changes caused by the statement execution
             * might affect many files.
             */
            session.update_all_diagnostics().await;

            session
                .client
                .show_message(MessageType::INFO, &result.message)
                .await;

            // The rows go back to the client, so editor extensions can render them as a table.
            Ok(Some(serde_json::to_value(result)?))
        }
        "pgls.invalidateSchemaCache" => {
            session.workspace.invalidate_schema_cache(true)?;
            // Everything that depends on the schema is out of date.
            session.update_all_diagnostics().await;

            session
                .client
                .show_message(MessageType::INFO, "Schema cache invalidated")
                .await;

            Ok(None)
        }
        any => Err(anyhow!(format!("Unknown command: {any}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::{FIX_ALL_KIND, changed_range, covers};
    use pgls_text_size::TextRange;
    use tower_lsp_server::ls_types::CodeActionKind;

    #[test]
    fn requested_kinds_cover_their_children() {
        for kind in ["source", "source.fixAll", "source.fixAll.pgls"] {
            assert!(covers(&CodeActionKind::from(kind), FIX_ALL_KIND), "{kind}");
        }
        for kind in ["quickfix", "source.fix", "source.fixAll.other"] {
            assert!(!covers(&CodeActionKind::from(kind), FIX_ALL_KIND), "{kind}");
        }
    }

    #[test]
    fn changed_range_keeps_the_common_start_and_end() {
        assert_eq!(
            changed_range("drop index ä_idx;", "drop index concurrently ä_idx;"),
            (TextRange::new(11.into(), 11.into()), "concurrently ")
        );
        assert_eq!(
            changed_range("a", "a"),
            (TextRange::new(1.into(), 1.into()), "")
        );
        assert_eq!(
            changed_range("ab", "b"),
            (TextRange::new(0.into(), 1.into()), "")
        );
    }
}
