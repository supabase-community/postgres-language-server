//! Converts flat `TextSize` offsets into `(Line, Column)` positions and back, with columns in
//! UTF-8, UTF-16 or UTF-32 code units.
//!
//! Lines end at `\n`: a lone `\r` doesn't end a line, and the `\r` of a `\r\n` is the last
//! column of its line.

mod line_index;
#[cfg(test)]
mod tests;

use pgls_text_size::TextSize;

pub use line_index::LineIndex;

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
