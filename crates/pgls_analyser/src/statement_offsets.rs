use pgls_text_size::{TextRange, TextSize};

/// Where the text of a statement is in the document, when it is not a slice of the document: the
/// body of a SQL function quoted with `'`, where each `'` of the body is written `''`. Empty for
/// every other statement, whose offsets are the same in the statement and the document.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StatementOffsets {
    /// The offsets in the statement of the characters that take one more byte in the document,
    /// in ascending order.
    escaped: Vec<TextSize>,
}

impl StatementOffsets {
    pub fn new(escaped: Vec<TextSize>) -> Self {
        debug_assert!(escaped.is_sorted());
        Self { escaped }
    }

    pub fn is_empty(&self) -> bool {
        self.escaped.is_empty()
    }

    /// The offset in the document of `offset` in the statement, both relative to the start of
    /// the statement.
    pub fn to_document(&self, offset: TextSize) -> TextSize {
        let before = self.escaped.partition_point(|&escaped| escaped < offset);
        offset + TextSize::from(u32::try_from(before).unwrap_or(u32::MAX))
    }

    pub fn range_to_document(&self, range: TextRange) -> TextRange {
        TextRange::new(
            self.to_document(range.start()),
            self.to_document(range.end()),
        )
    }

    /// The offset in the statement of `offset` in the document, both relative to the start of
    /// the statement. Both bytes of an escaped character map to the character.
    pub fn to_statement(&self, offset: TextSize) -> TextSize {
        // The escaped characters that end before `offset` in the document.
        let before = self
            .escaped
            .iter()
            .enumerate()
            .take_while(|&(i, &escaped)| {
                escaped + TextSize::from(u32::try_from(i).unwrap_or(u32::MAX) + 2) <= offset
            })
            .count();
        let in_escape = self.escaped.get(before).is_some_and(|&escaped| {
            escaped + TextSize::from(u32::try_from(before).unwrap_or(u32::MAX) + 1) == offset
        });
        offset
            - TextSize::from(u32::try_from(before).unwrap_or(u32::MAX))
            - TextSize::from(u32::from(in_escape))
    }
}

#[cfg(test)]
mod tests {
    use pgls_text_size::TextSize;

    use super::StatementOffsets;

    #[test]
    fn maps_offsets_through_doubled_quotes() {
        // The statement `a'b'c`, written `a''b''c` in the document.
        let offsets = StatementOffsets::new(vec![1.into(), 3.into()]);
        let document = [0, 1, 3, 4, 6, 7];
        for (statement, document) in document.into_iter().enumerate() {
            let statement = TextSize::try_from(statement).unwrap();
            let document = TextSize::from(document);
            assert_eq!(offsets.to_document(statement), document);
            assert_eq!(offsets.to_statement(document), statement);
        }
        // The second quote of an escape is the quote.
        assert_eq!(offsets.to_statement(2.into()), 1.into());
        assert_eq!(offsets.to_statement(5.into()), 3.into());

        let identity = StatementOffsets::default();
        assert_eq!(identity.to_document(5.into()), 5.into());
        assert_eq!(identity.to_statement(5.into()), 5.into());
    }
}
