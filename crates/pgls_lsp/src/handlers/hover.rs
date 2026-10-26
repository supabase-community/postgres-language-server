use pgls_workspace::{WorkspaceError, features::on_hover::OnHoverParams};
use tower_lsp_server::ls_types::{self, MarkedString, MarkupContent};

use crate::{adapters::get_cursor_position, diagnostics::LspError, session::Session};

#[tracing::instrument(level = "debug", skip(session), err)]
pub(crate) fn on_hover(
    session: &Session,
    params: ls_types::HoverParams,
) -> Result<Option<ls_types::HoverContents>, LspError> {
    let url = params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;
    let path = session.file_path(&url)?;
    let project_key = session.project_key(&path);

    match session.workspace.on_hover(OnHoverParams {
        project_key,
        path,
        position: get_cursor_position(session, &url, position)?,
    }) {
        Ok(result) => {
            tracing::debug!("Found hover items: {:#?}", result);

            let hover_items: Vec<MarkedString> = result
                .into_iter()
                .map(MarkedString::from_markdown)
                .collect();

            if hover_items.is_empty() {
                Ok(None)
            } else {
                Ok(Some(ls_types::HoverContents::Array(hover_items)))
            }
        }

        Err(e) => match e {
            // Hover loads the schema of the database, which fails either because the database
            // can't be reached or because one of its queries fails, e.g. for a missing permission.
            WorkspaceError::DatabaseConnectionError(error) => {
                let value = if error.is_connection_failure() {
                    format!("Cannot connect to the database: {}", error.message())
                } else {
                    format!("Cannot load the database schema: {}", error.message())
                };
                Ok(Some(ls_types::HoverContents::Markup(MarkupContent {
                    kind: ls_types::MarkupKind::PlainText,
                    value,
                })))
            }
            _ => {
                tracing::error!("Received an error: {:#?}", e);
                Err(e.into())
            }
        },
    }
}
