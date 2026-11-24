//! `LineIndex` maps flat `TextSize` offsets into `(Line, Column)`
//! representation.

use pgls_text_size::TextSize;

use crate::{LineCol, PositionEncoding, WideChar, WideEncoding, WideLineCol};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LineIndex {
    /// Offset the beginning of each line, zero-based.
    line_starts: Vec<TextSize>,
    /// The offsets of the `\n`s that follow a `\r`, in order.
    crlf_newlines: Vec<TextSize>,
    /// The non-ASCII characters of the text, in order. Empty for an ASCII text.
    wide_chars: Vec<WideChar>,
    /// The length of the text.
    len: TextSize,
}

/// The index of an empty text.
impl Default for LineIndex {
    fn default() -> Self {
        Self::new("")
    }
}

impl LineIndex {
    /// The index of `text`. The line breaks are found with `memchr`, and only the lines that
    /// aren't ASCII are decoded, so an ASCII text with `\n` line breaks takes one allocation.
    pub fn new(text: &str) -> LineIndex {
        #[expect(
            clippy::expect_used,
            reason = "offsets are `TextSize` (u32) everywhere, documents over 4 GiB aren't supported"
        )]
        let len = TextSize::try_from(text.len()).expect("TextSize overflow");
        // Every offset in the text fits, since its length does.
        let size = |offset: usize| TextSize::try_from(offset).unwrap_or(len);

        let bytes = text.as_bytes();
        let line_breaks = || memchr::memchr2_iter(b'\n', b'\r', bytes);
        // Each `\n` and `\r` starts at most one line.
        let mut line_starts = Vec::with_capacity(line_breaks().count() + 1);
        line_starts.push(TextSize::from(0));
        let mut crlf_newlines = Vec::new();
        let mut wide_chars = Vec::new();

        let is_crlf_at = |start: usize| {
            bytes
                .get(start..)
                .is_some_and(|rest| rest.starts_with(b"\r\n"))
        };
        let mut line_start = 0;
        for at in line_breaks() {
            // The `\r` of a `\r\n` is part of its line until the `\n` ends it.
            if is_crlf_at(at) {
                continue;
            }
            if at.checked_sub(1).is_some_and(is_crlf_at) {
                crlf_newlines.push(size(at));
            }
            let line = text.get(line_start..at).unwrap_or_default();
            WideChar::push_line(&mut wide_chars, line, size(line_start));
            line_start = at + 1;
            line_starts.push(size(line_start));
        }
        let line = text.get(line_start..).unwrap_or_default();
        WideChar::push_line(&mut wide_chars, line, size(line_start));

        LineIndex {
            line_starts,
            crlf_newlines,
            wide_chars,
            len,
        }
    }

    /// The length of the text.
    pub fn len(&self) -> TextSize {
        self.len
    }

    /// The offset of the start of each line, starting with `0`. A line break at the end of the
    /// text starts an empty last line.
    pub fn line_starts(&self) -> &[TextSize] {
        &self.line_starts
    }

    /// The position of `offset`, with the column in the code units of `encoding`. An offset
    /// between the `\r` and the `\n` of a line break is the end of its line. Debug builds assert
    /// that `offset` is inside the text, release builds turn an offset past the end of the text
    /// into the end of the text. `None` if the column doesn't fit in a `u32`.
    pub fn position(&self, offset: TextSize, encoding: PositionEncoding) -> Option<WideLineCol> {
        let line_col = self.line_col(offset)?;
        let position = match encoding {
            PositionEncoding::Utf8 => WideLineCol {
                line: line_col.line,
                col: line_col.col,
            },
            PositionEncoding::Wide(enc) => self.to_wide(enc, line_col)?,
        };
        // An offset between the `\r` and the `\n` of a line break is the end of its line, so the
        // position maps back to the offset of `line_col`, not always to `offset`.
        debug_assert_eq!(
            Some(self.offset_at(position, encoding)),
            self.offset(line_col),
            "{position:?} doesn't map back to offset {offset:?}"
        );
        Some(position)
    }

    /// The offset of `position`, with the column in the code units of `encoding`. As the LSP
    /// specifies, a column past the end of its line is the end of the line, and a line past the
    /// end of the text is the end of the text. A column inside a character is the start of that
    /// character.
    pub fn offset_at(&self, position: WideLineCol, encoding: PositionEncoding) -> TextSize {
        let line_col = match encoding {
            PositionEncoding::Utf8 => LineCol {
                line: position.line,
                col: position.col,
            },
            PositionEncoding::Wide(enc) => self.to_utf8(enc, position),
        };
        self.offset_clamped(line_col)
    }

    /// The line and UTF-8 column of `offset`. An offset between the `\r` and the `\n` of a line
    /// break is the end of its line. Debug builds assert that `offset` is inside the text, release
    /// builds turn an offset past the end of the text into the end of the text.
    pub fn line_col(&self, offset: TextSize) -> Option<LineCol> {
        debug_assert!(
            self.is_char_boundary(offset),
            "offset {offset:?} is not on a char boundary of a text of length {:?}",
            self.len
        );
        let offset = offset.min(self.len);
        let offset = if self.crlf_newlines.binary_search(&offset).is_ok() {
            offset - TextSize::from(1)
        } else {
            offset
        };
        let line = self.line_starts.partition_point(|&it| it <= offset) - 1;
        let line_start_offset = self.line_starts.get(line)?;
        let col = offset - line_start_offset;

        Some(LineCol {
            line: u32::try_from(line).ok()?,
            col: col.into(),
        })
    }

    pub fn offset(&self, line_col: LineCol) -> Option<TextSize> {
        self.line_starts
            .get(line_col.line as usize)
            .map(|offset| offset + TextSize::from(line_col.col))
    }

    /// Like [`LineIndex::offset`], but a column past the end of its line is the end of the line,
    /// and a line past the end of the text is the end of the text. A column inside a character is
    /// the start of that character.
    fn offset_clamped(&self, line_col: LineCol) -> TextSize {
        let line = line_col.line as usize;
        let Some(&start) = self.line_starts.get(line) else {
            return self.len;
        };
        let col = TextSize::from(line_col.col).min(self.line_end(line) - start);
        let offset = self.floor_char_boundary(start + col);
        debug_assert!(self.is_char_boundary(offset));
        offset
    }

    /// The end of `line`: the start of its line break, or the end of the text.
    fn line_end(&self, line: usize) -> TextSize {
        let Some(&next) = self.line_starts.get(line + 1) else {
            return self.len;
        };
        let newline = next - TextSize::from(1);
        if self.crlf_newlines.binary_search(&newline).is_ok() {
            newline - TextSize::from(1)
        } else {
            newline
        }
    }

    /// Whether `offset` lies inside the text and not inside a multi-byte character.
    pub fn is_char_boundary(&self, offset: TextSize) -> bool {
        offset <= self.len && self.floor_char_boundary(offset) == offset
    }

    /// The start of the character that contains `offset`.
    fn floor_char_boundary(&self, offset: TextSize) -> TextSize {
        // The characters are sorted, so the first one that ends after `offset` is the only one
        // that can contain it.
        let index = self.wide_chars.partition_point(|c| c.end <= offset);
        match self.wide_chars.get(index) {
            Some(c) if c.start < offset => c.start,
            _ => offset,
        }
    }

    /// The non-ASCII characters of `line`, in order.
    fn wide_chars_of_line(&self, line: u32) -> &[WideChar] {
        if self.wide_chars.is_empty() {
            return &[];
        }
        let line = line as usize;
        let Some(&start) = self.line_starts.get(line) else {
            return &[];
        };
        let end = self.line_end(line);
        let first = self.wide_chars.partition_point(|c| c.start < start);
        let rest = self.wide_chars.get(first..).unwrap_or_default();
        rest.get(..rest.partition_point(|c| c.start < end))
            .unwrap_or_default()
    }

    fn to_wide(&self, enc: WideEncoding, line_col: LineCol) -> Option<WideLineCol> {
        let col = self.utf8_to_wide_col(enc, line_col.line, line_col.col.into());
        Some(WideLineCol {
            line: line_col.line,
            col: u32::try_from(col).ok()?,
        })
    }

    fn to_utf8(&self, enc: WideEncoding, line_col: WideLineCol) -> LineCol {
        let col = self.wide_to_utf8_col(enc, line_col.line, line_col.col);
        LineCol {
            line: line_col.line,
            col: col.into(),
        }
    }

    fn utf8_to_wide_col(&self, enc: WideEncoding, line: u32, col: TextSize) -> usize {
        let mut res: usize = col.into();
        let Some(&start) = self.line_starts.get(line as usize) else {
            return res;
        };
        for c in self.wide_chars_of_line(line) {
            if c.end - start <= col {
                res -= usize::from(c.len()) - c.wide_len(enc);
            } else {
                // From here on, all utf16 characters come *after* the character we are mapping,
                // so we don't need to take them into account
                break;
            }
        }
        res
    }

    fn wide_to_utf8_col(&self, enc: WideEncoding, line: u32, mut col: u32) -> TextSize {
        let Some(&start) = self.line_starts.get(line as usize) else {
            return col.into();
        };
        for c in self.wide_chars_of_line(line) {
            if col > u32::from(c.start - start) {
                col = col.saturating_add(u32::from(c.len()) - c.wide_len(enc) as u32);
            } else {
                // From here on, all utf16 characters come *after* the character we are mapping,
                // so we don't need to take them into account
                break;
            }
        }

        col.into()
    }
}
