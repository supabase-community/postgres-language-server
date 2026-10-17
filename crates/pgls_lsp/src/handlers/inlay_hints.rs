use pgls_workspace::{
    WorkspaceError,
    features::inlay_hints::{InlayHintKind, PullInlayHintsParams},
};
use tower_lsp::lsp_types;

use crate::{
    adapters::{from_lsp, to_lsp},
    diagnostics::LspError,
    session::Session,
};

#[tracing::instrument(level = "debug", skip(session), err)]
pub(crate) fn inlay_hints(
    session: &Session,
    params: lsp_types::InlayHintParams,
) -> Result<Option<Vec<lsp_types::InlayHint>>, LspError> {
    let url = params.text_document.uri;
    let path = session.file_path(&url)?;
    let doc = session.document(&url)?;
    let encoding = session.position_encoding();
    let range = from_lsp::text_range(&doc.line_index, params.range, encoding)?;

    let result = match session
        .workspace
        .pull_inlay_hints(PullInlayHintsParams { path, range })
    {
        Ok(result) => result,
        Err(e) => match e {
            WorkspaceError::DatabaseConnectionError(_) => return Ok(None),
            _ => return Err(e.into()),
        },
    };

    let hints = result
        .hints
        .into_iter()
        .map(|hint| {
            Ok(lsp_types::InlayHint {
                position: to_lsp::position(&doc.line_index, hint.offset, encoding)?,
                label: lsp_types::InlayHintLabel::String(hint.label),
                kind: Some(match hint.kind {
                    InlayHintKind::Parameter => lsp_types::InlayHintKind::PARAMETER,
                }),
                text_edits: None,
                tooltip: None,
                padding_left: None,
                padding_right: Some(true),
                data: None,
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(Some(hints))
}
