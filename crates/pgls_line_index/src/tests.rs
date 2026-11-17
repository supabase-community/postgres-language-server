use pgls_text_size::{TextRange, TextSize};

use crate::WideEncoding::{Utf16, Utf32};
use crate::{LineCol, LineIndex, WideLineCol, changed_range, lines, offset_of_char_position};

#[test]
fn line_breaks_and_wide_characters() {
    // Line 0 ends with `\r\n`, line 1 with a lone `\r`, line 2 with `\n`, and line 3 has a
    // 2-byte `é` (1 UTF-16 unit) and a 4-byte `😀` (a UTF-16 surrogate pair).
    let text = "ab\r\nc\rd\né😀x";
    let line_index = LineIndex::new(text);
    let line_col = |offset: u32| line_index.line_col(TextSize::from(offset)).unwrap();
    let at = |line, col| LineCol { line, col };

    assert_eq!(line_index.len(), TextSize::of(text));
    assert_eq!(
        line_index.line_starts(),
        [0, 4, 6, 8].map(TextSize::from).as_slice()
    );

    assert_eq!(line_col(2), at(0, 2));
    // Between the `\r` and the `\n`: the end of the line.
    assert_eq!(line_col(3), at(0, 2));
    assert_eq!(line_col(4), at(1, 0));
    assert_eq!(line_col(5), at(1, 1));
    assert_eq!(line_col(6), at(2, 0));
    assert_eq!(line_col(8), at(3, 0));
    assert_eq!(line_col(10), at(3, 2));
    assert_eq!(line_col(14), at(3, 6));
    assert_eq!(line_col(15), at(3, 7));

    // Past the end of a line: the start of its line break. Past the last line: the end of the
    // text.
    let clamped = |line, col| u32::from(line_index.offset_clamped(at(line, col)));
    assert_eq!(clamped(0, 100), 2);
    assert_eq!(clamped(1, 100), 5);
    assert_eq!(clamped(2, 100), 7);
    assert_eq!(clamped(3, 100), 15);
    assert_eq!(clamped(4, 0), 15);
    // Inside a character: the start of the character.
    assert_eq!(clamped(3, 1), 8);
    assert_eq!(clamped(3, 4), 10);
    let mid_surrogate_pair = line_index.to_utf8(Utf16, WideLineCol { line: 3, col: 2 });
    assert_eq!(u32::from(line_index.offset_clamped(mid_surrogate_pair)), 10);

    for (offset, utf16, utf32) in [(8, 0, 0), (10, 1, 1), (14, 3, 2), (15, 4, 3)] {
        let utf8 = line_col(offset);
        for (enc, col) in [(Utf16, utf16), (Utf32, utf32)] {
            let wide = line_index.to_wide(enc, utf8).unwrap();
            assert_eq!(wide, WideLineCol { line: 3, col }, "{offset} {enc:?}");
            assert_eq!(line_index.to_utf8(enc, wide), utf8, "{offset} {enc:?}");
        }
    }
}

#[test]
fn lines_end_like_in_the_line_index() {
    let collect = |text| lines(text).collect::<Vec<_>>();
    let line = |start: u32, text| (TextSize::from(start), text);

    assert_eq!(collect(""), [line(0, "")]);
    assert_eq!(collect("a"), [line(0, "a")]);
    assert_eq!(
        collect("a\r\nb\rc\n"),
        [line(0, "a"), line(3, "b"), line(5, "c"), line(7, "")]
    );
    assert_eq!(
        collect("\r\r\n\n"),
        [line(0, ""), line(1, ""), line(3, ""), line(4, "")]
    );
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
        ("select 1;\rselect 2;", "select 1;\r\nselect 2;"),
        ("select 1;\r\nselect 2;", "select 1;\rselect 2;"),
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

mod properties {
    use pgls_text_size::TextSize;
    use proptest::prelude::*;

    use crate::WideEncoding::{Utf16, Utf32};
    use crate::{LineCol, LineIndex, changed_range, lines};

    /// Texts of line breaks and characters of 1, 2 and 4 bytes in UTF-8, so that every line break,
    /// `\r\n` split or not, and every width of character shows up next to each other.
    fn text() -> impl Strategy<Value = String> {
        proptest::collection::vec(
            proptest::sample::select(vec!['a', 'é', '😀', '\r', '\n']),
            0..24,
        )
        .prop_map(|chars| chars.into_iter().collect())
    }

    /// Whether `offset` is between the `\r` and the `\n` of a line break.
    fn splits_crlf(text: &str, offset: usize) -> bool {
        text[..offset].ends_with('\r') && text[offset..].starts_with('\n')
    }

    proptest! {
        /// `lines` returns the lines of the line index, and between two lines there is exactly one
        /// line break.
        #[test]
        fn lines_are_the_lines_of_the_index(text in text()) {
            let line_index = LineIndex::new(&text);
            prop_assert_eq!(line_index.len(), TextSize::of(text.as_str()));
            let lines: Vec<_> = lines(&text).collect();
            let starts: Vec<_> = lines.iter().map(|(start, _)| *start).collect();
            prop_assert_eq!(line_index.line_starts(), starts.as_slice());

            for (i, (start, line)) in lines.iter().enumerate() {
                let start = usize::from(*start);
                prop_assert_eq!(&text[start..start + line.len()], *line);
                prop_assert!(!line.contains(['\r', '\n']));
                let next = lines.get(i + 1).map_or(text.len(), |(next, _)| usize::from(*next));
                let line_break = &text[start + line.len()..next];
                if i + 1 < lines.len() {
                    prop_assert!(["\n", "\r\n", "\r"].contains(&line_break), "{line_break:?}");
                } else {
                    prop_assert_eq!(line_break, "");
                }
            }
        }

        /// Every offset converts to a position and back, except one between the `\r` and the
        /// `\n` of a line break, which is the end of its line. An offset inside a character is
        /// no char boundary.
        #[test]
        fn positions_round_trip(text in text()) {
            let line_index = LineIndex::new(&text);
            let starts: Vec<_> = lines(&text).map(|(start, _)| usize::from(start)).collect();
            for offset in 0..=text.len() + 1 {
                let size = TextSize::try_from(offset).unwrap();
                if offset > text.len() || !text.is_char_boundary(offset) {
                    prop_assert!(!line_index.is_char_boundary(size), "{offset}");
                    continue;
                }
                prop_assert!(line_index.is_char_boundary(size), "{offset}");
                let line_col = line_index.line_col(size).unwrap();
                let at = if splits_crlf(&text, offset) { offset - 1 } else { offset };
                let line = starts.partition_point(|&start| start <= at) - 1;
                prop_assert_eq!(
                    line_col,
                    LineCol { line: line as u32, col: (at - starts[line]) as u32 },
                    "{}", offset
                );
                prop_assert_eq!(line_index.offset(line_col), Some(TextSize::try_from(at).unwrap()));
                prop_assert_eq!(line_index.offset_clamped(line_col), TextSize::try_from(at).unwrap());
                for enc in [Utf16, Utf32] {
                    let wide = line_index.to_wide(enc, line_col).unwrap();
                    prop_assert_eq!(line_index.to_utf8(enc, wide), line_col);
                }
            }
        }

        /// A column past the end of a line is the start of its line break.
        #[test]
        fn columns_past_the_end_of_a_line_are_clamped(text in text()) {
            let line_index = LineIndex::new(&text);
            for (line, (start, content)) in lines(&text).enumerate() {
                let end = start + TextSize::of(content);
                let line_col = LineCol { line: line as u32, col: u32::from(TextSize::of(content)) + 3 };
                prop_assert_eq!(line_index.offset_clamped(line_col), end);
            }
        }

        /// The edit turns `old` into `new`, replaces no more than what changed, and neither of its
        /// ends is between the `\r` and the `\n` of a line break.
        #[test]
        fn changed_range_is_the_change(old in text(), new in text()) {
            let (range, new_text) = changed_range(&old, &new).unwrap();
            let mut applied = old.clone();
            applied.replace_range(std::ops::Range::<usize>::from(range), new_text);
            prop_assert_eq!(&applied, &new);

            let (start, end) = (usize::from(range.start()), usize::from(range.end()));
            prop_assert!(!splits_crlf(&old, start) && !splits_crlf(&old, end));
            let prefix = old
                .char_indices()
                .zip(new.chars())
                .find(|((_, a), b)| a != b)
                .map_or(old.len().min(new.len()), |((index, _), _)| index);
            // Only a `\r\n` that the change would split is taken in whole.
            prop_assert!(
                start == prefix || (start + 1 == prefix && splits_crlf(&old, prefix)),
                "{:?} {}", range, prefix
            );
            let suffix: usize = old[prefix..]
                .chars()
                .rev()
                .zip(new[prefix..].chars().rev())
                .take_while(|(a, b)| a == b)
                .map(|(a, _)| a.len_utf8())
                .sum();
            let unchanged_end = old.len() - suffix;
            prop_assert!(
                end == unchanged_end
                    || (end == unchanged_end + 1 && splits_crlf(&old, unchanged_end)),
                "{:?} {}", range, unchanged_end
            );
        }
    }
}
