use crate::{adapters::get_cursor_position, session::Session};
use anyhow::{Result, anyhow};
use tower_lsp::lsp_types::{
    self, CodeAction, CodeActionDisabled, CodeActionOrCommand, Command, ExecuteCommandParams,
    MessageType,
};

use pgls_workspace::features::code_actions::{
    CodeActionKind, CodeActionsParams, CommandActionCategory, ExecuteStatementParams,
};

#[tracing::instrument(level = "debug", skip(session), err)]
pub fn get_actions(
    session: &Session,
    params: lsp_types::CodeActionParams,
) -> Result<lsp_types::CodeActionResponse> {
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
                        kind: Some(lsp_types::CodeActionKind::EMPTY),
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
                }
            }

            _ => todo!(),
        })
        .collect();

    Ok(actions
        .into_iter()
        .map(CodeActionOrCommand::CodeAction)
        .collect())
}

pub fn command_id(command: &CommandActionCategory) -> String {
    match command {
        CommandActionCategory::ExecuteStatement(_) => "pgt.executeStatement".into(),
    }
}

#[tracing::instrument(level = "debug", skip(session), err)]
pub async fn execute_command(
    session: &Session,
    params: ExecuteCommandParams,
) -> anyhow::Result<Option<serde_json::Value>> {
    let command = params.command;

    match command.as_str() {
        "pgt.executeStatement" => {
            let statement_id = serde_json::from_value::<pgls_workspace::workspace::StatementId>(
                params.arguments[0].clone(),
            )?;
            let doc_url: lsp_types::Url = serde_json::from_value(params.arguments[1].clone())?;

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

        any => Err(anyhow!(format!("Unknown command: {}", any))),
    }
}
