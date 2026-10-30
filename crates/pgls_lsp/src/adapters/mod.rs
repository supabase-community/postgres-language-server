//! The crate contains a set of converters to translate between `lsp-types` and `text_size` (and vice versa) types.

use pgls_line_index::WideEncoding;
use pgls_text_size::TextSize;
use tower_lsp_server::ls_types::{ClientCapabilities, Position, PositionEncodingKind, Uri};

use crate::session::Session;

pub mod from_lsp;
pub mod to_lsp;

pub fn negotiated_encoding(capabilities: &ClientCapabilities) -> PositionEncoding {
    let client_encodings = match &capabilities.general {
        Some(general) => general.position_encodings.as_deref().unwrap_or_default(),
        None => &[],
    };

    for enc in client_encodings {
        if enc == &PositionEncodingKind::UTF8 {
            return PositionEncoding::Utf8;
        } else if enc == &PositionEncodingKind::UTF32 {
            return PositionEncoding::Wide(WideEncoding::Utf32);
        }
        // NB: intentionally prefer just about anything else to utf-16.
    }

    PositionEncoding::Wide(WideEncoding::Utf16)
}

pub fn get_cursor_position(
    session: &Session,
    url: &Uri,
    position: Position,
) -> anyhow::Result<TextSize> {
    let line_index = session
        .document(url)
        .map(|doc| doc.line_index)
        .map_err(|_| anyhow::anyhow!("Document not found."))?;

    let cursor_pos = from_lsp::offset(&line_index, position, session.position_encoding())?;

    Ok(cursor_pos)
}

#[derive(Clone, Copy, Debug)]
pub enum PositionEncoding {
    Utf8,
    Wide(WideEncoding),
}

#[cfg(test)]
mod tests {
    use crate::adapters::PositionEncoding;
    use crate::adapters::from_lsp::offset;
    use crate::adapters::to_lsp::position;
    use pgls_line_index::{LineIndex, WideEncoding};
    use pgls_text_size::TextSize;
    use tower_lsp_server::ls_types::Position;

    macro_rules! check_conversion {
        ($line_index:ident : $position:expr_2021 => $text_size:expr_2021 ) => {
            let position_encoding = PositionEncoding::Wide(WideEncoding::Utf16);

            let offset = offset(&$line_index, $position, position_encoding).ok();
            assert_eq!(offset, Some($text_size));

            let position = position(&$line_index, offset.unwrap(), position_encoding).ok();

            assert_eq!(position, Some($position));
        };
    }

    #[test]
    fn empty_string() {
        let line_index = LineIndex::new("");
        check_conversion!(line_index: Position { line: 0, character: 0 } => TextSize::from(0));
    }

    #[test]
    fn empty_line() {
        let line_index = LineIndex::new("\n\n");
        check_conversion!(line_index: Position { line: 1, character: 0 } => TextSize::from(1));
    }

    #[test]
    fn line_end() {
        let line_index = LineIndex::new("abc\ndef\nghi");
        check_conversion!(line_index: Position { line: 1, character: 3 } => TextSize::from(7));
    }

    #[test]
    fn out_of_range_positions_are_clamped() {
        // No line break at the end: lines 0 and 1.
        let line_index = LineIndex::new("abcde\nfg😀");
        let at = |line, character| {
            offset(
                &line_index,
                Position { line, character },
                PositionEncoding::Wide(WideEncoding::Utf16),
            )
            .unwrap()
        };
        assert_eq!(at(0, 3), TextSize::from(3));
        // Past the end of a line: the end of that line.
        assert_eq!(at(0, 10), TextSize::from(5));
        assert_eq!(at(1, 100), TextSize::from(12));
        assert_eq!(at(1, u32::MAX), TextSize::from(12));
        // Past the last line: the end of the text.
        assert_eq!(at(2, 0), TextSize::from(12));
        assert_eq!(at(u32::MAX, u32::MAX), TextSize::from(12));
    }

    #[test]
    fn with_tabs() {
        let line_index = LineIndex::new(
            r#"
select
    email,
    id
from auth.users u 
join public.client_identities c on u.id = c.user_id;
"#
            .trim(),
        );

        // on `i` of `id` in the select
        // 22 because of:
        // selectemail,i = 13
        // 8 spaces, 2 newlines = 23 characters
        // it's zero indexed => index 22
        check_conversion!(line_index: Position { line: 2, character: 4 } => TextSize::from(22));
    }

    #[test]
    fn unicode() {
        let line_index = LineIndex::new("'Jan 1, 2018 – Jan 1, 2019'");

        check_conversion!(line_index: Position { line: 0, character: 0 } => TextSize::from(0));
        check_conversion!(line_index: Position { line: 0, character: 1 } => TextSize::from(1));
        check_conversion!(line_index: Position { line: 0, character: 12 } => TextSize::from(12));
        check_conversion!(line_index: Position { line: 0, character: 13 } => TextSize::from(15));
        check_conversion!(line_index: Position { line: 0, character: 14 } => TextSize::from(18));
        check_conversion!(line_index: Position { line: 0, character: 15 } => TextSize::from(21));
        check_conversion!(line_index: Position { line: 0, character: 26 } => TextSize::from(32));
        check_conversion!(line_index: Position { line: 0, character: 27 } => TextSize::from(33));
    }
}
