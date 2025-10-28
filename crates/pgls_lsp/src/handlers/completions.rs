use crate::{
    adapters::{self, get_cursor_position},
    diagnostics::LspError,
    session::Session,
};
use anyhow::Result;
use pgls_workspace::{WorkspaceError, features::completions::GetCompletionsParams};
use tower_lsp::lsp_types::{
    self, CompletionItem, CompletionItemLabelDetails, InsertTextFormat, TextEdit,
};

#[tracing::instrument(level = "debug", skip(session), err)]
pub fn get_completions(
    session: &Session,
    params: lsp_types::CompletionParams,
) -> Result<lsp_types::CompletionResponse, LspError> {
    let url = params.text_document_position.text_document.uri;
    let path = session.file_path(&url)?;

    let doc = session.document(&url)?;
    let encoding = adapters::negotiated_encoding(session.client_capabilities().unwrap());

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
                detail: i
                    .detail
                    .map(|s| format!(" {s}"))
                    .or(Some(format!(" {}", i.kind))),
            }),
            preselect: Some(i.preselected),
            sort_text: Some(i.sort_text),
            insert_text_format: if i.completion_text.as_ref().is_some_and(|c| c.is_snippet) {
                Some(InsertTextFormat::SNIPPET)
            } else {
                Some(InsertTextFormat::PLAIN_TEXT)
            },
            text_edit: i.completion_text.map(|c| {
                lsp_types::CompletionTextEdit::Edit(TextEdit {
                    new_text: c.text,
                    range: adapters::to_lsp::range(&doc.line_index, c.range, encoding).unwrap(),
                })
            }),
            kind: Some(to_lsp_types_completion_item_kind(i.kind)),
            ..CompletionItem::default()
        })
        .collect();

    Ok(lsp_types::CompletionResponse::Array(items))
}

fn to_lsp_types_completion_item_kind(
    pg_comp_kind: pgls_completions::CompletionItemKind,
) -> lsp_types::CompletionItemKind {
    match pg_comp_kind {
        pgls_completions::CompletionItemKind::Function => lsp_types::CompletionItemKind::FUNCTION,
        pgls_completions::CompletionItemKind::Table => lsp_types::CompletionItemKind::CLASS,
        pgls_completions::CompletionItemKind::Column => lsp_types::CompletionItemKind::FIELD,
        pgls_completions::CompletionItemKind::Schema => lsp_types::CompletionItemKind::CLASS,
        pgls_completions::CompletionItemKind::Policy => lsp_types::CompletionItemKind::CONSTANT,
        pgls_completions::CompletionItemKind::Role => lsp_types::CompletionItemKind::CONSTANT,
    }
}
