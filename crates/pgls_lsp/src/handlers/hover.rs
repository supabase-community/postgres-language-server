use pgls_workspace::{WorkspaceError, features::on_hover::OnHoverParams};
use tower_lsp::lsp_types::{self, MarkedString, MarkupContent};

use crate::{adapters::get_cursor_position, diagnostics::LspError, session::Session};

#[tracing::instrument(level = "debug", skip(session), err)]
pub(crate) fn on_hover(
    session: &Session,
    params: lsp_types::HoverParams,
) -> Result<lsp_types::HoverContents, LspError> {
    let url = params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;
    let path = session.file_path(&url)?;

    match session.workspace.on_hover(OnHoverParams {
        path,
        position: get_cursor_position(session, &url, position)?,
    }) {
        Ok(result) => {
            tracing::debug!("Found hover items: {:#?}", result);

            Ok(lsp_types::HoverContents::Array(
                result
                    .into_iter()
                    .map(MarkedString::from_markdown)
                    .collect(),
            ))
        }

        Err(e) => match e {
            WorkspaceError::DatabaseConnectionError(_) => {
                Ok(lsp_types::HoverContents::Markup(MarkupContent {
                    kind: lsp_types::MarkupKind::PlainText,
                    value: "Cannot connect to database.".into(),
                }))
            }
            _ => {
                tracing::error!("Received an error: {:#?}", e);
                Err(e.into())
            }
        },
    }
}
