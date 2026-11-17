use pgls_text_size::{TextRange, TextSize};

use crate::LineBreak;

/// The lines of a text, each with the offset of its start and without its line break. A line ends
/// at `\n`, `\r\n` or a lone `\r`, like in [`LineIndex`](crate::LineIndex). Unlike [`str::lines`],
/// a line break at the end of the text is followed by an empty line, so the `n`th item is line `n`
/// of the line index.
#[derive(Clone, Debug)]
pub struct Lines<'a> {
    /// The text after the lines returned so far, `None` after the last line.
    rest: Option<&'a str>,
    /// The offset of `rest` in the text.
    offset: TextSize,
}

impl<'a> Lines<'a> {
    /// The lines of `text`.
    pub fn new(text: &'a str) -> Self {
        Self {
            rest: Some(text),
            offset: TextSize::from(0),
        }
    }

    /// The range of the line that contains `offset` of the text, without its line break. Only
    /// looks at that line, not at the lines before it. An offset in a line break is on the line it
    /// ends, an offset past the end of the text is on the last line, and an offset inside a
    /// character is at its start.
    pub fn line_at(self, offset: TextSize) -> TextRange {
        let text = self.rest.unwrap_or_default();
        let mut at =
            usize::from(offset.checked_sub(self.offset).unwrap_or_default()).min(text.len());
        while !text.is_char_boundary(at) {
            at -= 1;
        }
        if Self::splits_crlf(text, at) {
            at -= 1;
        }
        let (before, after) = text.split_at(at);
        let start = before
            .rfind(LineBreak::START_CHARS)
            .map_or(0, |line_break| line_break + 1);
        let end = after
            .find(LineBreak::START_CHARS)
            .map_or(text.len(), |line_break| at + line_break);
        let to_size = |offset: usize| self.offset + TextSize::try_from(offset).unwrap_or_default();
        TextRange::new(to_size(start), to_size(end))
    }

    /// Whether `offset` is between the `\r` and the `\n` of a line break, where no position can
    /// point.
    pub(crate) fn splits_crlf(text: &str, offset: usize) -> bool {
        text[..offset].ends_with('\r') && text[offset..].starts_with('\n')
    }
}

impl<'a> Iterator for Lines<'a> {
    type Item = (TextSize, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        let rest = self.rest?;
        let start = self.offset;
        let Some(end) = rest.find(LineBreak::START_CHARS) else {
            self.rest = None;
            return Some((start, rest));
        };
        let (line, after) = rest.split_at(end);
        let line_break =
            LineBreak::at_start_of(after).map_or(1, |line_break| line_break.as_str().len());
        let after = after.get(line_break..).unwrap_or_default();
        self.offset += TextSize::of(rest) - TextSize::of(after);
        self.rest = Some(after);
        Some((start, line))
    }
}
