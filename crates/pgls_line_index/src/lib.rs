//! Converts flat `TextSize` offsets into `(Line, Column)` positions and back, with columns in
//! UTF-8, UTF-16 or UTF-32 code units: [`LineIndex::position`] and [`LineIndex::offset_at`].
//!
//! Lines end at `\n`, `\r\n` or a lone `\r`, like in the LSP. The `\r` of a `\r\n` belongs to the
//! line break: a position can't point between the `\r` and the `\n`. [`changed_range`] follows
//! that rule for the edit that turns one text into another, and [`Lines`] splits a text into the
//! same lines.
//!
//! [`offset_of_char_position`] converts the character positions that Postgres reports.

mod changed_range;
mod char_position;
mod line_break;
mod line_index;
mod lines;
#[cfg(test)]
mod tests;

use pgls_text_size::TextSize;

pub use changed_range::changed_range;
pub use char_position::offset_of_char_position;
pub use line_break::LineBreak;
pub use line_index::LineIndex;
pub use lines::Lines;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WideEncoding {
    Utf16,
    Utf32,
}

/// The code units that the column of a position counts, e.g. the encoding that an LSP client and
/// server agreed on.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PositionEncoding {
    Utf8,
    Wide(WideEncoding),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LineCol {
    /// Zero-based
    pub line: u32,
    /// Zero-based utf8 offset
    pub col: u32,
}

/// A line and a column in the code units of a [`PositionEncoding`]. Deliberately not a generic
/// type and different from `LineCol`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WideLineCol {
    /// Zero-based
    pub line: u32,
    /// Zero-based
    pub col: u32,
}

/// A non-ASCII character of the text.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct WideChar {
    /// Start offset of the character in the text
    start: TextSize,
    /// End offset of the character in the text
    end: TextSize,
}

impl WideChar {
    /// Pushes the non-ASCII characters of `line`, which starts at offset `start`, onto
    /// `wide_chars`. An ASCII line is checked a word at a time, and only a line that isn't ASCII
    /// is decoded.
    fn push_line(wide_chars: &mut Vec<WideChar>, line: &str, start: TextSize) {
        if line.is_ascii() {
            return;
        }
        let mut offset = start;
        for char in line.chars() {
            let end = offset + TextSize::of(char);
            if !char.is_ascii() {
                wide_chars.push(WideChar { start: offset, end });
            }
            offset = end;
        }
    }

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
