use std::collections::HashMap;

use crate::{
    adapters::{get_cursor_position, to_lsp},
    session::Session,
};
use anyhow::{Result, anyhow};
use pgls_analyse::{Applicability, SUPPRESSION_ACTION_CATEGORY};
use tower_lsp_server::ls_types::{
    self, CodeAction, CodeActionDisabled, CodeActionOrCommand, Command, ExecuteCommandParams,
    MessageType, WorkspaceEdit,
};

use pgls_workspace::features::code_actions::{
    CodeActionKind, CodeActionsParams, CommandActionCategory, ExecuteStatementParams,
};

#[tracing::instrument(level = "debug", skip(session), err)]
pub fn get_actions(
    session: &Session,
    params: ls_types::CodeActionParams,
) -> Result<ls_types::CodeActionResponse> {
    let url = params.text_document.uri;
    let path = session.file_path(&url)?;

    let cursor_position = get_cursor_position(session, &url, params.range.start)?;

    let workspace_actions = session.workspace.pull_code_actions(CodeActionsParams {
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
                        diagnostic.code == Some(ls_types::NumberOrString::String(edit.rule.clone()))
                    })
                    .cloned()
                    .collect();
                let is_suppression = edit.category.matches(SUPPRESSION_ACTION_CATEGORY);

                Some(CodeAction {
                    title: action.title,
                    kind: Some(ls_types::CodeActionKind::from(
                        edit.category.to_str().into_owned(),
                    )),
                    diagnostics: (!diagnostics.is_empty()).then_some(diagnostics),
                    edit: Some(WorkspaceEdit {
                        changes: Some(HashMap::from([(url.clone(), edits)])),
                        ..Default::default()
                    }),
                    is_preferred: (!is_suppression && edit.applicability == Applicability::Always)
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
        .map(CodeActionOrCommand::CodeAction)
        .collect())
}

pub fn command_id(command: &CommandActionCategory) -> String {
    match command {
        CommandActionCategory::ExecuteStatement(_) => "pgls.executeStatement".into(),
        CommandActionCategory::InvalidateSchemaCache => "pgls.invalidateSchemaCache".into(),
    }
}

#[tracing::instrument(level = "debug", skip(session), err)]
pub async fn execute_command(
    session: &Session,
    params: ExecuteCommandParams,
) -> anyhow::Result<Option<serde_json::Value>> {
    let command = params.command;

    match command.as_str() {
        "pgls.executeStatement" => {
            let statement_id = serde_json::from_value::<pgls_workspace::workspace::StatementId>(
                params.arguments[0].clone(),
            )?;
            let doc_url: ls_types::Uri = serde_json::from_value(params.arguments[1].clone())?;

            let path = session.file_path(&doc_url)?;

            let result = session
                .workspace
                .execute_statement(ExecuteStatementParams { statement_id, path })?;

            /*
             * Updating all diagnostics: the changes caused by the statement execution
             * might affect many files.
             */
            session.update_all_diagnostics().await;

            session
                .client
                .show_message(MessageType::INFO, result.message)
                .await;

            Ok(None)
        }
        "pgls.invalidateSchemaCache" => {
            session.workspace.invalidate_schema_cache(true)?;

            session
                .client
                .show_message(MessageType::INFO, "Schema cache invalidated")
                .await;

            Ok(None)
        }
        any => Err(anyhow!(format!("Unknown command: {any}"))),
    }
}
