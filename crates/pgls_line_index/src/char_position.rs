use pgls_text_size::TextSize;

/// The byte offset in `text` of a 1-based character position, which is how Postgres reports the
/// position of an error ([`parser_errposition`]). The position after the last character, which
/// Postgres reports for an error at the end of the input, is the end of `text`. `None` for 0 and
/// for positions further out, e.g. because the position comes from a different text.
///
/// [`parser_errposition`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_node.c#L106
pub fn offset_of_char_position(text: &str, position: usize) -> Option<TextSize> {
    let index = position.checked_sub(1)?;
    let offset = text
        .char_indices()
        .map(|(offset, _)| offset)
        .chain([text.len()])
        .nth(index)?;
    TextSize::try_from(offset).ok()
}
