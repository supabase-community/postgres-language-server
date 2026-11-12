use pgls_text_size::{TextRange, TextSize};

use crate::WideEncoding::{Utf16, Utf32};
use crate::{LineCol, LineIndex, WideLineCol, changed_range, offset_of_char_position};

#[test]
fn line_breaks_and_wide_characters() {
    // Line 0 ends with `\r\n`, line 1 contains a lone `\r`, which doesn't end it, and line 2
    // has a 2-byte `é` (1 UTF-16 unit) and a 4-byte `😀` (a UTF-16 surrogate pair).
    let text = "ab\r\nc\rd\né😀x";
    let line_index = LineIndex::new(text);
    let line_col = |offset: u32| line_index.line_col(TextSize::from(offset)).unwrap();
    let at = |line, col| LineCol { line, col };

    assert_eq!(line_col(2), at(0, 2));
    // Between the `\r` and the `\n`: the end of the line.
    assert_eq!(line_col(3), at(0, 2));
    assert_eq!(line_col(4), at(1, 0));
    assert_eq!(line_col(6), at(1, 2));
    assert_eq!(line_col(8), at(2, 0));
    assert_eq!(line_col(10), at(2, 2));
    assert_eq!(line_col(14), at(2, 6));
    assert_eq!(line_col(15), at(2, 7));

    // Past the end of a line: the start of its line break. Past the last line: the end of the
    // text.
    let clamped = |line, col| u32::from(line_index.offset_clamped(at(line, col)));
    assert_eq!(clamped(0, 100), 2);
    assert_eq!(clamped(1, 100), 7);
    assert_eq!(clamped(2, 100), 15);
    assert_eq!(clamped(3, 0), 15);
    // Inside a character: the start of the character.
    assert_eq!(clamped(2, 1), 8);
    assert_eq!(clamped(2, 4), 10);
    let mid_surrogate_pair = line_index.to_utf8(Utf16, WideLineCol { line: 2, col: 2 });
    assert_eq!(u32::from(line_index.offset_clamped(mid_surrogate_pair)), 10);

    for (offset, utf16, utf32) in [(8, 0, 0), (10, 1, 1), (14, 3, 2), (15, 4, 3)] {
        let utf8 = line_col(offset);
        for (enc, col) in [(Utf16, utf16), (Utf32, utf32)] {
            let wide = line_index.to_wide(enc, utf8).unwrap();
            assert_eq!(wide, WideLineCol { line: 2, col }, "{offset} {enc:?}");
            assert_eq!(line_index.to_utf8(enc, wide), utf8, "{offset} {enc:?}");
        }
    }
}

#[test]
fn out_of_bounds_line() {
    let line_index = LineIndex::new("abcde\nfghij\n");

    let offset = line_index.offset(LineCol { line: 5, col: 0 });
    assert!(offset.is_none());
}

#[ignore]
#[test]
fn test_every_chars() {
    let text: String = {
        let mut chars: Vec<char> = ((0 as char)..char::MAX).collect();
        chars.extend("\n".repeat(chars.len() / 16).chars());
        chars.into_iter().collect()
    };

    let line_index = LineIndex::new(&text);

    let mut lin_col = LineCol { line: 0, col: 0 };
    let mut col_utf16 = 0;
    let mut col_utf32 = 0;
    for (offset, char) in text.char_indices() {
        let got_offset = line_index.offset(lin_col).unwrap();
        assert_eq!(usize::from(got_offset), offset);

        let got_lin_col = line_index.line_col(got_offset).unwrap();
        assert_eq!(got_lin_col, lin_col);

        for enc in [Utf16, Utf32] {
            let wide_lin_col = line_index.to_wide(enc, lin_col).unwrap();
            let got_lin_col = line_index.to_utf8(enc, wide_lin_col);
            assert_eq!(got_lin_col, lin_col);

            let want_col = match enc {
                Utf16 => col_utf16,
                Utf32 => col_utf32,
            };
            assert_eq!(wide_lin_col.col, want_col)
        }

        if char == '\n' {
            lin_col.line += 1;
            lin_col.col = 0;
            col_utf16 = 0;
            col_utf32 = 0;
        } else {
            lin_col.col += char.len_utf8() as u32;
            col_utf16 += char.len_utf16() as u32;
            col_utf32 += 1;
        }
    }
}

#[test]
fn changed_range_keeps_the_common_start_and_end() {
    let range = |start: u32, end: u32| TextRange::new(start.into(), end.into());
    assert_eq!(
        changed_range("drop index ä_idx;", "drop index concurrently ä_idx;"),
        Some((range(11, 11), "concurrently "))
    );
    assert_eq!(changed_range("a", "a"), Some((range(1, 1), "")));
    assert_eq!(changed_range("ab", "b"), Some((range(0, 1), "")));
}

#[test]
fn changed_range_keeps_crlf_whole() {
    for (old, new) in [
        ("select 1;\r\nselect 2;", "select 1;\nselect 2;"),
        ("select 1;\nselect 2;", "select 1;\r\nselect 2;"),
        ("select 1;\r\n", "select 1;\r\r\n"),
        ("select 1;\r\nselect 2;", "select 1;\r\n\r\nselect 2;"),
    ] {
        let (range, new_text) = changed_range(old, new).unwrap();
        let mut applied = old.to_string();
        applied.replace_range(std::ops::Range::<usize>::from(range), new_text);
        assert_eq!(applied, new, "{range:?}");

        // Neither end is between the `\r` and the `\n`, so a position can point at both.
        let line_index = LineIndex::new(old);
        for offset in [range.start(), range.end()] {
            let line_col = line_index.line_col(offset).unwrap();
            assert_eq!(
                line_index.offset_clamped(line_col),
                offset,
                "{old:?} {offset:?}"
            );
        }
    }
}

#[test]
fn character_positions_are_one_based() {
    let text = "select 'prüfen', missing from t";
    let missing = text.find("missing").unwrap();
    let position = text[..missing].chars().count() + 1;
    let chars = text.chars().count();

    assert_eq!(
        offset_of_char_position(text, position),
        Some(TextSize::from(missing as u32))
    );
    assert_eq!(offset_of_char_position(text, 1), Some(TextSize::from(0)));
    assert_eq!(
        offset_of_char_position(text, chars + 1),
        Some(TextSize::of(text))
    );
    assert_eq!(offset_of_char_position(text, 0), None);
    assert_eq!(offset_of_char_position(text, chars + 2), None);
    assert_eq!(offset_of_char_position("", 1), Some(TextSize::from(0)));
}
