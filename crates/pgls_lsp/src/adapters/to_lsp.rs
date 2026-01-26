use crate::adapters::PositionEncoding;
use crate::adapters::line_index::LineIndex;
use anyhow::{Context, Result};
use pgls_text_edit::{CompressedOp, DiffOp, TextEdit};
use pgls_text_size::{TextRange, TextSize};
use tower_lsp::lsp_types;

/// The function is used to convert TextSize to a LSP position.
pub fn position(
    line_index: &LineIndex,
    offset: TextSize,
    position_encoding: PositionEncoding,
) -> Result<lsp_types::Position> {
    let line_col = line_index
        .line_col(offset)
        .with_context(|| format!("could not convert offset {offset:?} into a line-column index"))?;

    let position = match position_encoding {
        PositionEncoding::Utf8 => lsp_types::Position::new(line_col.line, line_col.col),
        PositionEncoding::Wide(enc) => {
            let line_col = line_index
                .to_wide(enc, line_col)
                .with_context(|| format!("could not convert {line_col:?} into wide line column"))?;
            lsp_types::Position::new(line_col.line, line_col.col)
        }
    };

    Ok(position)
}

/// The function is used to convert TextRange to a LSP range.
pub fn range(
    line_index: &LineIndex,
    range: TextRange,
    position_encoding: PositionEncoding,
) -> Result<lsp_types::Range> {
    let start = position(line_index, range.start(), position_encoding)?;
    let end = position(line_index, range.end(), position_encoding)?;
    Ok(lsp_types::Range::new(start, end))
}

/// Convert a TextEdit diff to a list of LSP TextEdits.
pub fn text_edits(
    diff: &TextEdit,
    line_index: &LineIndex,
    encoding: PositionEncoding,
) -> Result<Vec<lsp_types::TextEdit>> {
    let mut result = Vec::new();
    let mut offset = TextSize::from(0u32);

    for op in diff.iter() {
        match op {
            CompressedOp::DiffOp(DiffOp::Equal { range }) => {
                offset += range.len();
            }
            CompressedOp::DiffOp(DiffOp::Insert { range }) => {
                let start = position(line_index, offset, encoding)?;
                let last = result.last_mut().filter(|e: &&mut lsp_types::TextEdit| {
                    e.range.end == start && e.new_text.is_empty()
                });

                if let Some(last_edit) = last {
                    last_edit.new_text = diff.get_text(*range).to_string();
                } else {
                    result.push(lsp_types::TextEdit {
                        range: lsp_types::Range::new(start, start),
                        new_text: diff.get_text(*range).to_string(),
                    });
                }
            }
            CompressedOp::DiffOp(DiffOp::Delete { range }) => {
                let start = position(line_index, offset, encoding)?;
                offset += range.len();
                let end = position(line_index, offset, encoding)?;

                result.push(lsp_types::TextEdit {
                    range: lsp_types::Range::new(start, end),
                    new_text: String::new(),
                });
            }
            CompressedOp::EqualLines { line_count } => {
                let mut line_col = line_index
                    .line_col(offset)
                    .context("offset out of range while processing diff")?;
                line_col.line += line_count.get() + 1;
                line_col.col = 0;
                offset = line_index
                    .offset(line_col)
                    .context("line_col out of range while processing diff")?;
            }
        }
    }

    Ok(result)
}
