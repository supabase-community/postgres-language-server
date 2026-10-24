use crate::adapters::line_index::LineIndex;
use crate::adapters::{LineCol, PositionEncoding, WideLineCol};
use anyhow::{Result, ensure};
use pgls_text_size::{TextRange, TextSize};
use tower_lsp_server::ls_types;

/// The function is used to convert a LSP position to TextSize.
pub fn offset(
    line_index: &LineIndex,
    position: ls_types::Position,
    position_encoding: PositionEncoding,
) -> Result<TextSize> {
    let line_col = match position_encoding {
        PositionEncoding::Utf8 => LineCol {
            line: position.line,
            col: position.character,
        },
        PositionEncoding::Wide(enc) => {
            let line_col = WideLineCol {
                line: position.line,
                col: position.character,
            };
            line_index.to_utf8(enc, line_col)
        }
    };

    Ok(line_index.offset_clamped(line_col))
}

/// The function is used to convert a LSP range to TextRange.
pub fn text_range(
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
