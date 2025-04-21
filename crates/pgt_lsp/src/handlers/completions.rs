use crate::{adapters::get_cursor_position, session::Session};
use anyhow::Result;
use pgt_workspace::{WorkspaceError, features::completions::GetCompletionsParams};
use tower_lsp::lsp_types::{self, CompletionItem, CompletionItemLabelDetails};

#[tracing::instrument(level = "debug", skip(session), err)]
pub fn get_completions(
    session: &Session,
    params: lsp_types::CompletionParams,
) -> Result<lsp_types::CompletionResponse> {
    let url = params.text_document_position.text_document.uri;
    let path = session.file_path(&url)?;

    let completion_result = match session.workspace.get_completions(GetCompletionsParams {
        path,
        position: get_cursor_position(session, &url, params.text_document_position.position)?,
    }) {
        Ok(result) => result,
        Err(e) => match e {
            WorkspaceError::DatabaseConnectionError(_) => {
                return Ok(lsp_types::CompletionResponse::Array(vec![]));
            }
            _ => {
                return Err(e.into());
            }
        },
    };

    let items: Vec<CompletionItem> = completion_result
        .into_iter()
        .map(|i| CompletionItem {
            label: i.label,
            label_details: Some(CompletionItemLabelDetails {
                description: Some(i.description),
                detail: Some(format!(" {}", i.kind.to_string())),
            }),
            preselect: Some(i.preselected),
            kind: Some(to_lsp_types_completion_item_kind(i.kind)),
            sort_text: Some(i.sort_text),
            ..CompletionItem::default()
        })
        .collect();

    for item in &items {
        tracing::info!("{}, {}", item.label, item.sort_text.as_ref().unwrap());
    }

    Ok(lsp_types::CompletionResponse::Array(items))
}

fn to_lsp_types_completion_item_kind(
    pg_comp_kind: pgt_completions::CompletionItemKind,
) -> lsp_types::CompletionItemKind {
    match pg_comp_kind {
        pgt_completions::CompletionItemKind::Function => lsp_types::CompletionItemKind::FUNCTION,
        pgt_completions::CompletionItemKind::Table => lsp_types::CompletionItemKind::CLASS,
        pgt_completions::CompletionItemKind::Column => lsp_types::CompletionItemKind::FIELD,
        pgt_completions::CompletionItemKind::Schema => lsp_types::CompletionItemKind::CLASS,
    }
}
