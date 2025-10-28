use pgls_text_size::TextSize;

#[derive(Debug, Default, Clone)]
pub(crate) struct LineIndex {
    line_offset: Vec<pgls_text_size::TextSize>,
}

impl LineIndex {
    pub fn new(doc: &str) -> Self {
        let line_offset = std::iter::once(0)
            .chain(doc.match_indices(&['\n', '\r']).filter_map(|(i, _)| {
                let bytes = doc.as_bytes();

                match bytes[i] {
                    // Filter out the `\r` in `\r\n` to avoid counting the line break twice
                    b'\r' if i + 1 < bytes.len() && bytes[i + 1] == b'\n' => None,
                    _ => Some(i + 1),
                }
            }))
            .map(|i| TextSize::try_from(i).expect("integer overflow"))
            .collect();

        Self { line_offset }
    }

    pub fn offset_for_line(&self, idx: usize) -> Option<&pgls_text_size::TextSize> {
        self.line_offset.get(idx)
    }

    pub fn line_for_offset(&self, offset: TextSize) -> Option<usize> {
        self.line_offset
            .iter()
            .enumerate()
            .filter_map(|(i, line_offset)| {
                if offset >= *line_offset {
                    Some(i)
                } else {
                    None
                }
            })
            .next_back()
    }
}
