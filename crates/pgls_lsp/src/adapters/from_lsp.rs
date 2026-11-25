use anyhow::{Result, ensure};
use pgls_line_index::{LineIndex, PositionEncoding, WideLineCol};
use pgls_text_size::{TextRange, TextSize};
use tower_lsp_server::ls_types;

/// The function is used to convert a LSP position to TextSize.
pub(crate) fn offset(
    line_index: &LineIndex,
    position: ls_types::Position,
    position_encoding: PositionEncoding,
) -> Result<TextSize> {
    let position = WideLineCol {
        line: position.line,
        col: position.character,
    };
    Ok(line_index.offset_at(position, position_encoding))
}

/// The function is used to convert a LSP range to TextRange.
pub(crate) fn text_range(
    line_index: &LineIndex,
    range: ls_types::Range,
    position_encoding: PositionEncoding,
) -> Result<TextRange> {
    let start = offset(line_index, range.start, position_encoding)?;
    let end = offset(line_index, range.end, position_encoding)?;
    // `TextRange::new` panics on a range that ends before it starts.
    ensure!(start <= end, "range {range:?} ends before it starts");
    Ok(TextRange::new(start, end))
}
