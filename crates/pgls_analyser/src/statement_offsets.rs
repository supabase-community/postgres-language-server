use pgls_text_size::{TextRange, TextSize};

/// Where the text of a statement is in the document, when it is not a slice of the document: the
/// body of a SQL function in a string constant with escapes, like `''` in `'...'`, or `\n` and
/// `é` in `E'...'`. Empty for every other statement, whose offsets are the same in the
/// statement and the document.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StatementOffsets {
    /// The escapes, in ascending order.
    escapes: Vec<Escape>,
}

/// The bytes of the statement that an escape in the document stands for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Escape {
    /// The bytes in the statement, relative to its start.
    statement: TextRange,
    /// The escape in the document, relative to the start of the statement.
    document: TextRange,
}

impl StatementOffsets {
    /// Adds an escape of `document_len` bytes in the document that stands for the
    /// `statement_len` bytes at `statement_offset` in the statement. Escapes are added in order.
    pub fn push_escape(
        &mut self,
        statement_offset: TextSize,
        statement_len: TextSize,
        document_len: TextSize,
    ) {
        let document_offset = self.to_document(statement_offset);
        debug_assert!(
            self.escapes
                .last()
                .is_none_or(|last| last.statement.end() <= statement_offset)
        );
        self.escapes.push(Escape {
            statement: TextRange::at(statement_offset, statement_len),
            document: TextRange::at(document_offset, document_len),
        });
    }

    pub fn is_empty(&self) -> bool {
        self.escapes.is_empty()
    }

    /// The offset in the document of `offset` in the statement, both relative to the start of
    /// the statement. Offsets inside the bytes of an escape map to the start of the escape.
    pub fn to_document(&self, offset: TextSize) -> TextSize {
        let index = self
            .escapes
            .partition_point(|escape| escape.statement.start() <= offset);
        let Some(escape) = index.checked_sub(1).and_then(|i| self.escapes.get(i)) else {
            return offset;
        };
        if offset < escape.statement.end() {
            escape.document.start()
        } else {
            offset - escape.statement.end() + escape.document.end()
        }
    }

    pub fn range_to_document(&self, range: TextRange) -> TextRange {
        TextRange::new(
            self.to_document(range.start()),
            self.to_document(range.end()),
        )
    }

    /// The offset in the statement of `offset` in the document, both relative to the start of
    /// the statement. Offsets inside an escape map to the start of what it stands for.
    pub fn to_statement(&self, offset: TextSize) -> TextSize {
        let index = self
            .escapes
            .partition_point(|escape| escape.document.start() <= offset);
        let Some(escape) = index.checked_sub(1).and_then(|i| self.escapes.get(i)) else {
            return offset;
        };
        if offset < escape.document.end() {
            escape.statement.start()
        } else {
            offset - escape.document.end() + escape.statement.end()
        }
    }
}

#[cfg(test)]
mod tests {
    use pgls_text_size::TextSize;

    use super::StatementOffsets;

    #[test]
    fn maps_offsets_through_doubled_quotes() {
        // The statement `a'b'c`, written `a''b''c` in the document.
        let mut offsets = StatementOffsets::default();
        offsets.push_escape(1.into(), 1.into(), 2.into());
        offsets.push_escape(3.into(), 1.into(), 2.into());
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

    #[test]
    fn maps_offsets_through_longer_escapes() {
        // The statement `aéb` (`é` takes 2 bytes), written `aéb` in the document.
        let mut offsets = StatementOffsets::default();
        offsets.push_escape(1.into(), 2.into(), 6.into());
        assert_eq!(offsets.to_document(1.into()), 1.into());
        assert_eq!(offsets.to_document(3.into()), 7.into());
        assert_eq!(offsets.to_document(4.into()), 8.into());
        assert_eq!(offsets.to_statement(7.into()), 3.into());
        assert_eq!(offsets.to_statement(4.into()), 1.into());
        assert_eq!(offsets.to_statement(0.into()), 0.into());
    }
}
