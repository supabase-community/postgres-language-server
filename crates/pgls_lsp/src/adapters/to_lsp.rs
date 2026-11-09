use crate::adapters::{PositionEncoding, from_lsp};
use anyhow::{Context, Result};
use pgls_line_index::LineIndex;
use pgls_text_edit::{CompressedOp, DiffOp, TextEdit};
use pgls_text_size::{TextRange, TextSize};
use tower_lsp_server::ls_types;

/// The function is used to convert TextSize to a LSP position.
pub fn position(
    line_index: &LineIndex,
    offset: TextSize,
    position_encoding: PositionEncoding,
) -> Result<ls_types::Position> {
    let line_col = line_index
        .line_col(offset)
        .with_context(|| format!("could not convert offset {offset:?} into a line-column index"))?;

    let position = match position_encoding {
        PositionEncoding::Utf8 => ls_types::Position::new(line_col.line, line_col.col),
        PositionEncoding::Wide(enc) => {
            let line_col = line_index
                .to_wide(enc, line_col)
                .with_context(|| format!("could not convert {line_col:?} into wide line column"))?;
            ls_types::Position::new(line_col.line, line_col.col)
        }
    };
    // An offset between the `\r` and the `\n` of a line break is the end of its line, so the
    // position maps back to the offset of `line_col`, not always to `offset`.
    debug_assert_eq!(
        from_lsp::offset(line_index, position, position_encoding).ok(),
        line_index.offset(line_col),
        "{position:?} doesn't map back to offset {offset:?}"
    );

    Ok(position)
}

/// The function is used to convert TextRange to a LSP range.
pub fn range(
    line_index: &LineIndex,
    range: TextRange,
    position_encoding: PositionEncoding,
) -> Result<ls_types::Range> {
    let start = position(line_index, range.start(), position_encoding)?;
    let end = position(line_index, range.end(), position_encoding)?;
    Ok(ls_types::Range::new(start, end))
}

/// Convert a TextEdit diff to a list of LSP TextEdits.
pub fn text_edits(
    diff: &TextEdit,
    line_index: &LineIndex,
    encoding: PositionEncoding,
) -> Result<Vec<ls_types::TextEdit>> {
    let mut result = Vec::new();
    let mut offset = TextSize::from(0u32);

    for op in diff.iter() {
        match op {
            CompressedOp::DiffOp(DiffOp::Equal { range }) => {
                offset += range.len();
            }
            CompressedOp::DiffOp(DiffOp::Insert { range }) => {
                let start = position(line_index, offset, encoding)?;
                let last = result.last_mut().filter(|e: &&mut ls_types::TextEdit| {
                    e.range.end == start && e.new_text.is_empty()
                });

                if let Some(last_edit) = last {
                    last_edit.new_text = diff.get_text(*range).to_string();
                } else {
                    result.push(ls_types::TextEdit {
                        range: ls_types::Range::new(start, start),
                        new_text: diff.get_text(*range).to_string(),
                    });
                }
            }
            CompressedOp::DiffOp(DiffOp::Delete { range }) => {
                let start = position(line_index, offset, encoding)?;
                offset += range.len();
                let end = position(line_index, offset, encoding)?;

                result.push(ls_types::TextEdit {
                    range: ls_types::Range::new(start, end),
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
    debug_assert_eq!(
        offset,
        line_index.len(),
        "the diff doesn't cover the whole text"
    );

    Ok(result)
}

#[cfg(test)]
mod tests {
    use pgls_line_index::LineIndex;
    use pgls_text_edit::TextEdit;
    use tower_lsp_server::ls_types as lsp;

    use super::text_edits;
    use crate::adapters::PositionEncoding;

    #[test]
    fn test_diff_1() {
        const OLD: &str = "line 1 old
line 2
line 3
line 4
line 5
line 6
line 7 old";

        const NEW: &str = "line 1 new
line 2
line 3
line 4
line 5
line 6
line 7 new";

        let line_index = LineIndex::new(OLD);
        let diff = TextEdit::from_unicode_words(OLD, NEW);

        let text_edit = text_edits(&diff, &line_index, PositionEncoding::Utf8).unwrap();

        assert_eq!(
            text_edit.as_slice(),
            &[
                lsp::TextEdit {
                    range: lsp::Range {
                        start: lsp::Position {
                            line: 0,
                            character: 7,
                        },
                        end: lsp::Position {
                            line: 0,
                            character: 10,
                        },
                    },
                    new_text: String::from("new"),
                },
                lsp::TextEdit {
                    range: lsp::Range {
                        start: lsp::Position {
                            line: 6,
                            character: 7
                        },
                        end: lsp::Position {
                            line: 6,
                            character: 10
                        }
                    },
                    new_text: String::from("new"),
                },
            ]
        );
    }

    #[test]
    fn test_diff_2() {
        const OLD: &str = "console.log(\"Variable: \" + variable);";
        const NEW: &str = "console.log(`Variable: ${variable}`);";

        let line_index = LineIndex::new(OLD);
        let diff = TextEdit::from_unicode_words(OLD, NEW);

        let text_edit = text_edits(&diff, &line_index, PositionEncoding::Utf8).unwrap();

        assert_eq!(
            text_edit.as_slice(),
            &[
                lsp::TextEdit {
                    range: lsp::Range {
                        start: lsp::Position {
                            line: 0,
                            character: 12,
                        },
                        end: lsp::Position {
                            line: 0,
                            character: 13,
                        },
                    },
                    new_text: String::from("`"),
                },
                lsp::TextEdit {
                    range: lsp::Range {
                        start: lsp::Position {
                            line: 0,
                            character: 23
                        },
                        end: lsp::Position {
                            line: 0,
                            character: 27
                        }
                    },
                    new_text: String::from("${"),
                },
                lsp::TextEdit {
                    range: lsp::Range {
                        start: lsp::Position {
                            line: 0,
                            character: 35
                        },
                        end: lsp::Position {
                            line: 0,
                            character: 35
                        }
                    },
                    new_text: String::from("}`"),
                },
            ]
        );
    }
}
