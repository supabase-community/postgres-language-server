//! `LineIndex` maps flat `TextSize` offsets into `(Line, Column)`
//! representation.

use std::mem;

use pgls_text_size::TextSize;
use rustc_hash::FxHashMap;

use crate::{LineCol, WideChar, WideEncoding, WideLineCol};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LineIndex {
    /// Offset the beginning of each line, zero-based.
    newlines: Vec<TextSize>,
    /// The offsets of the `\n`s that follow a `\r`, in order.
    crlf_newlines: Vec<TextSize>,
    /// List of non-ASCII characters on each line.
    line_wide_chars: FxHashMap<u32, Vec<WideChar>>,
    /// The length of the text.
    len: TextSize,
}

impl LineIndex {
    pub fn new(text: &str) -> LineIndex {
        let mut line_wide_chars = FxHashMap::default();
        let mut wide_chars = Vec::new();

        let mut newlines = vec![TextSize::from(0)];
        let mut crlf_newlines = Vec::new();

        let mut current_col = TextSize::from(0);

        let mut line = 0;
        for (offset, char) in text.char_indices() {
            let char_size = TextSize::of(char);

            if char == '\n' {
                #[expect(
                    clippy::expect_used,
                    reason = "offsets are `TextSize` (u32) everywhere, documents over 4 GiB aren't supported"
                )]
                let char_offset = TextSize::try_from(offset).expect("TextSize overflow");
                newlines.push(char_offset + char_size);
                if text[..offset].ends_with('\r') {
                    crlf_newlines.push(char_offset);
                }

                // Save any utf-16 characters seen in the previous line
                if !wide_chars.is_empty() {
                    line_wide_chars.insert(line, mem::take(&mut wide_chars));
                }

                // Prepare for processing the next line
                current_col = TextSize::from(0);
                line += 1;
                continue;
            }

            if !char.is_ascii() {
                wide_chars.push(WideChar {
                    start: current_col,
                    end: current_col + char_size,
                });
            }

            current_col += char_size;
        }

        // Save any utf-16 characters seen in the last line
        if !wide_chars.is_empty() {
            line_wide_chars.insert(line, wide_chars);
        }

        LineIndex {
            newlines,
            crlf_newlines,
            line_wide_chars,
            len: TextSize::of(text),
        }
    }

    /// The line and UTF-8 column of `offset`. An offset between the `\r` and the `\n` of a line
    /// break is the end of its line, and an offset past the end of the text is the end of the text.
    pub fn line_col(&self, offset: TextSize) -> Option<LineCol> {
        let offset = offset.min(self.len);
        let offset = if self.crlf_newlines.binary_search(&offset).is_ok() {
            offset - TextSize::from(1)
        } else {
            offset
        };
        let line = self.newlines.partition_point(|&it| it <= offset) - 1;
        let line_start_offset = self.newlines.get(line)?;
        let col = offset - line_start_offset;

        Some(LineCol {
            line: u32::try_from(line).ok()?,
            col: col.into(),
        })
    }

    pub fn offset(&self, line_col: LineCol) -> Option<TextSize> {
        self.newlines
            .get(line_col.line as usize)
            .map(|offset| offset + TextSize::from(line_col.col))
    }

    /// Like [`LineIndex::offset`], but a column past the end of its line is the end of the line,
    /// as the LSP specifies, and a line past the end of the text is the end of the text. A column
    /// inside a character is the start of that character.
    pub fn offset_clamped(&self, line_col: LineCol) -> TextSize {
        let line = line_col.line as usize;
        let Some(&start) = self.newlines.get(line) else {
            return self.len;
        };
        let col = TextSize::from(line_col.col).min(self.line_end(line) - start);
        start + self.floor_char_boundary(line_col.line, col)
    }

    /// The end of `line`: the start of its line break, or the end of the text.
    fn line_end(&self, line: usize) -> TextSize {
        let Some(&next) = self.newlines.get(line + 1) else {
            return self.len;
        };
        let newline = next - TextSize::from(1);
        if self.crlf_newlines.binary_search(&newline).is_ok() {
            newline - TextSize::from(1)
        } else {
            newline
        }
    }

    /// The start of the character that contains column `col` of `line`.
    fn floor_char_boundary(&self, line: u32, col: TextSize) -> TextSize {
        let Some(wide_chars) = self.line_wide_chars.get(&line) else {
            return col;
        };
        // The characters of a line are sorted, so the first one that ends after `col` is the only
        // one that can contain it.
        let index = wide_chars.partition_point(|c| c.end <= col);
        match wide_chars.get(index) {
            Some(c) if c.start < col => c.start,
            _ => col,
        }
    }

    pub fn to_wide(&self, enc: WideEncoding, line_col: LineCol) -> Option<WideLineCol> {
        let col = self.utf8_to_wide_col(enc, line_col.line, line_col.col.into());
        Some(WideLineCol {
            line: line_col.line,
            col: u32::try_from(col).ok()?,
        })
    }

    pub fn to_utf8(&self, enc: WideEncoding, line_col: WideLineCol) -> LineCol {
        let col = self.wide_to_utf8_col(enc, line_col.line, line_col.col);
        LineCol {
            line: line_col.line,
            col: col.into(),
        }
    }

    fn utf8_to_wide_col(&self, enc: WideEncoding, line: u32, col: TextSize) -> usize {
        let mut res: usize = col.into();
        if let Some(wide_chars) = self.line_wide_chars.get(&line) {
            for c in wide_chars {
                if c.end <= col {
                    res -= usize::from(c.len()) - c.wide_len(enc);
                } else {
                    // From here on, all utf16 characters come *after* the character we are mapping,
                    // so we don't need to take them into account
                    break;
                }
            }
        }
        res
    }

    fn wide_to_utf8_col(&self, enc: WideEncoding, line: u32, mut col: u32) -> TextSize {
        if let Some(wide_chars) = self.line_wide_chars.get(&line) {
            for c in wide_chars {
                if col > u32::from(c.start) {
                    col = col.saturating_add(u32::from(c.len()) - c.wide_len(enc) as u32);
                } else {
                    // From here on, all utf16 characters come *after* the character we are mapping,
                    // so we don't need to take them into account
                    break;
                }
            }
        }

        col.into()
    }
}
