//! `LineIndex` maps flat `TextSize` offsets into `(Line, Column)`
//! representation, with columns in UTF-8, UTF-16 or UTF-32 code units.
//!
//! Lines end at `\n`, so a lone `\r` doesn't end a line. The `\r` of a `\r\n` belongs to the line
//! break, like in the LSP: a position can't point between the `\r` and the `\n`.

use std::mem;

use pgls_text_size::TextSize;
use rustc_hash::FxHashMap;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WideEncoding {
    Utf16,
    Utf32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LineCol {
    /// Zero-based
    pub line: u32,
    /// Zero-based utf8 offset
    pub col: u32,
}

/// Deliberately not a generic type and different from `LineCol`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WideLineCol {
    /// Zero-based
    pub line: u32,
    /// Zero-based
    pub col: u32,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct WideChar {
    /// Start offset of a character inside a line, zero-based
    start: TextSize,
    /// End offset of a character inside a line, zero-based
    end: TextSize,
}

impl WideChar {
    /// Returns the length in 8-bit UTF-8 code units.
    fn len(&self) -> TextSize {
        self.end - self.start
    }

    /// Returns the length in UTF-16 or UTF-32 code units.
    fn wide_len(&self, enc: WideEncoding) -> usize {
        match enc {
            WideEncoding::Utf16 => {
                if self.len() == TextSize::from(4) {
                    2
                } else {
                    1
                }
            }

            WideEncoding::Utf32 => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use pgls_text_size::TextSize;

    use crate::WideEncoding::{Utf16, Utf32};
    use crate::{LineCol, LineIndex, WideLineCol};

    #[test]
    fn line_breaks_and_wide_characters() {
        // Line 0 ends with `\r\n`, line 1 contains a lone `\r`, which doesn't end it, and line 2
        // has a 2-byte `é` (1 UTF-16 unit) and a 4-byte `😀` (a UTF-16 surrogate pair).
        let text = "ab\r\nc\rd\né😀x";
        let line_index = LineIndex::new(text);
        let line_col = |offset: u32| line_index.line_col(TextSize::from(offset)).unwrap();
        let at = |line, col| LineCol { line, col };

        assert_eq!(line_col(2), at(0, 2));
        // Between the `\r` and the `\n`: the end of the line.
        assert_eq!(line_col(3), at(0, 2));
        assert_eq!(line_col(4), at(1, 0));
        assert_eq!(line_col(6), at(1, 2));
        assert_eq!(line_col(8), at(2, 0));
        assert_eq!(line_col(10), at(2, 2));
        assert_eq!(line_col(14), at(2, 6));
        assert_eq!(line_col(15), at(2, 7));
        assert_eq!(line_col(100), at(2, 7));

        // Past the end of a line: the start of its line break. Past the last line: the end of the
        // text.
        let clamped = |line, col| u32::from(line_index.offset_clamped(at(line, col)));
        assert_eq!(clamped(0, 100), 2);
        assert_eq!(clamped(1, 100), 7);
        assert_eq!(clamped(2, 100), 15);
        assert_eq!(clamped(3, 0), 15);
        // Inside a character: the start of the character.
        assert_eq!(clamped(2, 1), 8);
        assert_eq!(clamped(2, 4), 10);
        let mid_surrogate_pair = line_index.to_utf8(Utf16, WideLineCol { line: 2, col: 2 });
        assert_eq!(u32::from(line_index.offset_clamped(mid_surrogate_pair)), 10);

        for (offset, utf16, utf32) in [(8, 0, 0), (10, 1, 1), (14, 3, 2), (15, 4, 3)] {
            let utf8 = line_col(offset);
            for (enc, col) in [(Utf16, utf16), (Utf32, utf32)] {
                let wide = line_index.to_wide(enc, utf8).unwrap();
                assert_eq!(wide, WideLineCol { line: 2, col }, "{offset} {enc:?}");
                assert_eq!(line_index.to_utf8(enc, wide), utf8, "{offset} {enc:?}");
            }
        }
    }

    #[test]
    fn out_of_bounds_line() {
        let line_index = LineIndex::new("abcde\nfghij\n");

        let offset = line_index.offset(LineCol { line: 5, col: 0 });
        assert!(offset.is_none());
    }

    #[ignore]
    #[test]
    fn test_every_chars() {
        let text: String = {
            let mut chars: Vec<char> = ((0 as char)..char::MAX).collect();
            chars.extend("\n".repeat(chars.len() / 16).chars());
            chars.into_iter().collect()
        };

        let line_index = LineIndex::new(&text);

        let mut lin_col = LineCol { line: 0, col: 0 };
        let mut col_utf16 = 0;
        let mut col_utf32 = 0;
        for (offset, char) in text.char_indices() {
            let got_offset = line_index.offset(lin_col).unwrap();
            assert_eq!(usize::from(got_offset), offset);

            let got_lin_col = line_index.line_col(got_offset).unwrap();
            assert_eq!(got_lin_col, lin_col);

            for enc in [Utf16, Utf32] {
                let wide_lin_col = line_index.to_wide(enc, lin_col).unwrap();
                let got_lin_col = line_index.to_utf8(enc, wide_lin_col);
                assert_eq!(got_lin_col, lin_col);

                let want_col = match enc {
                    Utf16 => col_utf16,
                    Utf32 => col_utf32,
                };
                assert_eq!(wide_lin_col.col, want_col)
            }

            if char == '\n' {
                lin_col.line += 1;
                lin_col.col = 0;
                col_utf16 = 0;
                col_utf32 = 0;
            } else {
                lin_col.col += char.len_utf8() as u32;
                col_utf16 += char.len_utf16() as u32;
                col_utf32 += 1;
            }
        }
    }
}
