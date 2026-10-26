use pgls_workspace::{
    WorkspaceError,
    features::inlay_hints::{InlayHintKind, PullInlayHintsParams},
};
use tower_lsp_server::ls_types;

use crate::{
    adapters::{from_lsp, to_lsp},
    diagnostics::LspError,
    session::Session,
};

#[tracing::instrument(level = "debug", skip(session), err)]
pub(crate) fn inlay_hints(
    session: &Session,
    params: ls_types::InlayHintParams,
) -> Result<Option<Vec<ls_types::InlayHint>>, LspError> {
    let url = params.text_document.uri;
    let path = session.file_path(&url)?;
    let project_key = session.project_key(&path);
    let doc = session.document(&url)?;
    let encoding = session.position_encoding();
    let range = from_lsp::text_range(&doc.line_index, params.range, encoding)?;

    let result = match session.workspace.pull_inlay_hints(PullInlayHintsParams {
        project_key,
        path,
        range,
    }) {
        Ok(result) => result,
        Err(e) => match e {
            WorkspaceError::DatabaseConnectionError(_) => return Ok(None),
            _ => return Err(e.into()),
        },
    };

    // A hint that can't be placed is left out rather than failing the whole response.
    let hints = result
        .hints
        .into_iter()
        .filter_map(|hint| {
            let position = to_lsp::position(&doc.line_index, hint.offset, encoding)
                .inspect_err(|error| tracing::warn!("Skipping an inlay hint: {error}"))
                .ok()?;
            Some(ls_types::InlayHint {
                position,
                label: ls_types::InlayHintLabel::String(hint.label),
                kind: Some(match hint.kind {
                    InlayHintKind::Parameter => ls_types::InlayHintKind::PARAMETER,
                }),
                text_edits: None,
                tooltip: None,
                padding_left: None,
                padding_right: Some(true),
                data: None,
            })
        })
        .collect();

    Ok(Some(hints))
}
