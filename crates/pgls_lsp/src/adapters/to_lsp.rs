use anyhow::{Context, Result};
use pgls_line_index::{LineIndex, PositionEncoding};
use pgls_text_edit::{CompressedOp, DiffOp, TextEdit};
use pgls_text_size::{TextRange, TextSize};
use tower_lsp_server::ls_types;

/// The function is used to convert TextSize to a LSP position.
pub(crate) fn position(
    line_index: &LineIndex,
    offset: TextSize,
    position_encoding: PositionEncoding,
) -> Result<ls_types::Position> {
    let position = line_index
        .position(offset, position_encoding)
        .with_context(|| format!("could not convert offset {offset:?} into a position"))?;
    Ok(ls_types::Position::new(position.line, position.col))
}

/// The function is used to convert TextRange to a LSP range.
pub(crate) fn range(
    line_index: &LineIndex,
    range: TextRange,
    position_encoding: PositionEncoding,
) -> Result<ls_types::Range> {
    let start = position(line_index, range.start(), position_encoding)?;
    let end = position(line_index, range.end(), position_encoding)?;
    Ok(ls_types::Range::new(start, end))
}

/// Convert a TextEdit diff to a list of LSP TextEdits.
///
/// A position can't point between the `\r` and the `\n` of a line break, so an edit that starts
/// or ends there takes in the half of the line break next to it, and keeps it, like
/// [`pgls_line_index::changed_range`].
pub(crate) fn text_edits(
    diff: &TextEdit,
    line_index: &LineIndex,
    encoding: PositionEncoding,
) -> Result<Vec<ls_types::TextEdit>> {
    // The ranges of the old text that change and their new text, adjacent ones merged.
    let mut edits: Vec<(TextRange, String)> = Vec::new();
    let mut offset = TextSize::from(0u32);

    for op in diff.iter() {
        let (changed, new_text) = match op {
            CompressedOp::DiffOp(DiffOp::Equal { range }) => {
                offset += range.len();
                continue;
            }
            CompressedOp::DiffOp(DiffOp::Insert { range }) => {
                (TextRange::empty(offset), diff.get_text(*range))
            }
            CompressedOp::DiffOp(DiffOp::Delete { range }) => {
                let start = offset;
                offset += range.len();
                (TextRange::new(start, offset), "")
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
                continue;
            }
        };
        match edits.last_mut() {
            Some((last, text)) if last.end() == changed.start() => {
                *last = last.cover(changed);
                text.push_str(new_text);
            }
            _ => edits.push((changed, new_text.to_string())),
        }
    }
    debug_assert_eq!(
        offset,
        line_index.len(),
        "the diff doesn't cover the whole text"
    );

    edits
        .into_iter()
        .map(|(changed, mut new_text)| {
            let mut start = changed.start();
            if line_index.splits_crlf(start) {
                start -= TextSize::from(1);
                new_text.insert(0, '\r');
            }
            let mut end = changed.end();
            if line_index.splits_crlf(end) {
                end += TextSize::from(1);
                new_text.push('\n');
            }
            Ok(ls_types::TextEdit {
                range: range(line_index, TextRange::new(start, end), encoding)?,
                new_text,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use pgls_line_index::LineIndex;
    use pgls_text_edit::TextEdit;
    use tower_lsp_server::ls_types as lsp;

    use super::text_edits;
    use crate::adapters::from_lsp;
    use pgls_line_index::{PositionEncoding, WideEncoding};
    use pgls_text_edit::RangeEdit;

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

    /// An edit that changes one half of a `\r\n` takes in the whole line break, as no position
    /// points between its halves.
    #[test]
    fn edits_keep_crlf_whole() {
        let encoding = PositionEncoding::Wide(WideEncoding::Utf16);
        for (old, new) in [
            ("select 1;\r\nselect 2;", "select 1;\nselect 2;"),
            ("select 1;\nselect 2;", "select 1;\r\nselect 2;"),
            ("select 1;\r\n", "select 1;\r\r\n"),
            ("select 1;\r\nselect 2;", "select 1;\r\n\r\nselect 2;"),
            ("select\r\n  1;\r\n", "select\n  1;\n"),
            ("select 'ä'\r\n  from t;\r\n", "select 'ä' from t;\r\n"),
        ] {
            let index = LineIndex::new(old);
            let diff = TextEdit::from_unicode_words(old, new);
            let edits: Vec<RangeEdit> = text_edits(&diff, &index, encoding)
                .unwrap()
                .into_iter()
                .map(|edit| {
                    let range = from_lsp::text_range(&index, edit.range, encoding).unwrap();
                    RangeEdit::new(range, edit.new_text)
                })
                .collect();
            let applied = RangeEdit::apply_all(old, 0u32.into(), &edits);
            assert_eq!(applied.as_deref(), Some(new), "{edits:?}");
        }
    }
}
