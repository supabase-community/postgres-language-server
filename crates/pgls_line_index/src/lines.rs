use pgls_text_size::TextSize;

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
        let Some(end) = rest.find(['\n', '\r']) else {
            self.rest = None;
            return Some((start, rest));
        };
        let (line, after) = rest.split_at(end);
        let after = after
            .strip_prefix("\r\n")
            .or_else(|| after.get(1..))
            .unwrap_or_default();
        self.offset += TextSize::of(rest) - TextSize::of(after);
        self.rest = Some(after);
        Some((start, line))
    }
}
