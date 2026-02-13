use pgls_text_edit::TextEdit as PglsTextEdit;
use pgls_workspace::features::format::{PullFileFormattingParams, PullFormattingResult};
use tower_lsp::lsp_types::{DocumentFormattingParams, DocumentRangeFormattingParams, TextEdit};

use crate::{
    adapters::{PositionEncoding, from_lsp, line_index::LineIndex, to_lsp},
    diagnostics::LspError,
    session::Session,
};

pub fn formatting(
    session: &Session,
    params: DocumentFormattingParams,
) -> Result<Option<Vec<TextEdit>>, LspError> {
    let url = params.text_document.uri;
    let path = session.file_path(&url)?;

    let result = session
        .workspace
        .pull_file_formatting(PullFileFormattingParams { path, range: None })?;

    result_to_edits(result, session.position_encoding())
}

pub fn range_formatting(
    session: &Session,
    params: DocumentRangeFormattingParams,
) -> Result<Option<Vec<TextEdit>>, LspError> {
    let url = params.text_document.uri;
    let path = session.file_path(&url)?;
    let doc = session.document(&url)?;
    let range = from_lsp::text_range(&doc.line_index, params.range, session.position_encoding())?;

    let result = session
        .workspace
        .pull_file_formatting(PullFileFormattingParams {
            path,
            range: Some(range),
        })?;

    result_to_edits(result, session.position_encoding())
}

fn result_to_edits(
    result: PullFormattingResult,
    encoding: PositionEncoding,
) -> Result<Option<Vec<TextEdit>>, LspError> {
    if result.original == result.formatted {
        return Ok(None);
    }

    let diff = PglsTextEdit::from_unicode_words(&result.original, &result.formatted);
    let line_index = LineIndex::new(&result.original);
    let edits = to_lsp::text_edits(&diff, &line_index, encoding)?;

    Ok(Some(edits))
}
