/// Whether `byte` can continue an unquoted identifier. Postgres treats every byte with the high
/// bit set as a letter, so this covers every non-ASCII character in UTF-8.
/// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L332-L333
fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$') || !byte.is_ascii()
}

/// Whether `byte` can start an unquoted identifier.
fn is_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_' || !byte.is_ascii()
}

pub(super) fn reference_end(sql: &str, start: usize) -> Option<usize> {
    let bytes = sql.as_bytes();
    if start >= bytes.len() {
        return None;
    }
    let mut cursor = start;
    loop {
        if cursor >= bytes.len() {
            break;
        }
        if bytes[cursor] == b'"' {
            cursor += 1;
            while cursor < bytes.len() {
                if bytes[cursor] == b'"' {
                    cursor += 1;
                    if cursor < bytes.len() && bytes[cursor] == b'"' {
                        cursor += 1;
                    } else {
                        break;
                    }
                } else {
                    cursor += 1;
                }
            }
        } else {
            let token_start = cursor;
            while cursor < bytes.len() && is_identifier_byte(bytes[cursor]) {
                cursor += 1;
            }
            if cursor == token_start {
                return None;
            }
        }
        let mut dot = cursor;
        while dot < bytes.len() && bytes[dot].is_ascii_whitespace() {
            dot += 1;
        }
        if dot < bytes.len() && bytes[dot] == b'.' {
            dot += 1;
            while dot < bytes.len() && bytes[dot].is_ascii_whitespace() {
                dot += 1;
            }
            cursor = dot;
        } else {
            break;
        }
    }
    Some(cursor)
}

/// The end of `*`, or of a qualified `t.*` or `"s"."t".*`, starting at `start`.
pub(super) fn star_end(sql: &str, start: usize) -> Option<usize> {
    let bytes = sql.as_bytes();
    let mut cursor = start;
    loop {
        match *bytes.get(cursor)? {
            b'*' => return Some(cursor + 1),
            b'.' => cursor += 1,
            byte if byte.is_ascii_whitespace() => cursor += 1,
            b'"' => {
                cursor += 1;
                loop {
                    if *bytes.get(cursor)? == b'"' {
                        cursor += 1;
                        if bytes.get(cursor) != Some(&b'"') {
                            break;
                        }
                    }
                    cursor += 1;
                }
            }
            byte if byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$') || byte >= 0x80 => {
                while bytes.get(cursor).is_some_and(|byte| {
                    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$') || *byte >= 0x80
                }) {
                    cursor += 1;
                }
            }
            _ => return None,
        }
    }
}

pub(super) fn token_end(sql: &str, start: usize) -> Option<usize> {
    let bytes = sql.as_bytes();
    let mut cursor = start;
    while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
        cursor += 1;
    }
    let first = *bytes.get(cursor)?;
    if is_identifier_start(first) {
        while cursor < bytes.len() && is_identifier_byte(bytes[cursor]) {
            cursor += 1;
        }
        return Some(cursor);
    }
    if first == b'"' {
        return reference_end(sql, cursor);
    }
    while cursor < bytes.len()
        && !bytes[cursor].is_ascii_whitespace()
        && !is_identifier_byte(bytes[cursor])
        && !matches!(bytes[cursor], b'(' | b')' | b',' | b';')
    {
        cursor += 1;
    }
    (cursor > start).then_some(cursor)
}

pub(super) fn cast_end(sql: &str, start: usize) -> Option<usize> {
    let bytes = sql.as_bytes();
    if sql.get(start..)?.get(..5)?.eq_ignore_ascii_case("CAST(") {
        let mut depth = 0usize;
        for (offset, byte) in bytes[start..].iter().enumerate() {
            match byte {
                b'(' => depth += 1,
                b')' => {
                    depth = depth.checked_sub(1)?;
                    if depth == 0 {
                        return Some(start + offset + 1);
                    }
                }
                _ => {}
            }
        }
        return None;
    }
    let mut cursor = start;
    if bytes.get(cursor..cursor + 2) == Some(b"::") {
        cursor += 2;
    }
    while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
        cursor += 1;
    }
    let mut end = reference_end(sql, cursor)?;
    let mut after_type = end;
    while after_type < bytes.len() && bytes[after_type].is_ascii_whitespace() {
        after_type += 1;
    }
    if bytes.get(after_type) == Some(&b'\'') {
        let mut literal = after_type + 1;
        while literal < bytes.len() {
            if bytes[literal] == b'\'' {
                literal += 1;
                if bytes.get(literal) == Some(&b'\'') {
                    literal += 1;
                } else {
                    return Some(literal);
                }
            } else {
                literal += 1;
            }
        }
    }
    if bytes.get(end) == Some(&b'(') {
        let mut depth = 0usize;
        for (offset, byte) in bytes[end..].iter().enumerate() {
            match byte {
                b'(' => depth += 1,
                b')' => {
                    depth = depth.checked_sub(1)?;
                    if depth == 0 {
                        end += offset + 1;
                        break;
                    }
                }
                _ => {}
            }
        }
    }
    Some(end)
}

/// The end of the string constant starting at `start`: `'...'`, with an `E`, `B`, `X`, `N`, or
/// `U&` prefix, or `$tag$...$tag$`. Constants continued on a new line (`'a'\n'b'`) end after
/// their last part.
pub(super) fn literal_end(sql: &str, start: usize) -> Option<usize> {
    let bytes = sql.as_bytes();
    let tail = sql.get(start..)?;
    if let Some(after_dollar) = tail.strip_prefix('$') {
        let tag_len = after_dollar.find('$')? + 2;
        let tag = &tail[..tag_len];
        let tag_is_valid = tag[1..tag_len - 1]
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_');
        if !tag_is_valid {
            return None;
        }
        let close = tail[tag_len..].find(tag)?;
        return Some(start + tag_len + close + tag_len);
    }
    let (prefix, escapes) = match bytes.get(start..start + 2)? {
        [b'u' | b'U', b'&'] => (2, false),
        [b'e' | b'E', b'\''] => (1, true),
        [b'b' | b'B' | b'x' | b'X' | b'n' | b'N', b'\''] => (1, false),
        [b'\'', _] => (0, false),
        _ => return None,
    };
    let mut cursor = start + prefix;
    loop {
        // The opening quote.
        cursor += 1;
        loop {
            match bytes.get(cursor)? {
                b'\\' if escapes => cursor += 2,
                b'\'' if bytes.get(cursor + 1) == Some(&b'\'') => cursor += 2,
                b'\'' => break,
                _ => cursor += 1,
            }
        }
        cursor += 1;
        let mut next = cursor;
        let mut newline = false;
        while let Some(byte) = bytes.get(next).filter(|byte| byte.is_ascii_whitespace()) {
            newline |= *byte == b'\n';
            next += 1;
        }
        if !(newline && bytes.get(next) == Some(&b'\'')) {
            return Some(cursor);
        }
        cursor = next;
    }
}

pub(super) fn operator_end(sql: &str, start: usize) -> Option<usize> {
    let tail = sql.get(start..)?;
    if tail
        .get(..9)
        .is_some_and(|head| head.eq_ignore_ascii_case("operator("))
    {
        let close = tail.find(')')?;
        return Some(start + close + 1);
    }
    token_end(sql, start)
}

#[cfg(test)]
mod tests {
    use super::{cast_end, literal_end, operator_end, reference_end, star_end, token_end};

    fn text(sql: &str, start: usize, end: Option<usize>) -> &str {
        &sql[start..end.expect("span")]
    }

    #[test]
    fn type_finding_spans_cover_the_diagnostic_tokens() {
        let sql = "select a + b, OPERATOR(pg_catalog.+)(a,b), pg_catalog.f(a), x::bad, CAST(x AS bad), date 'today'";
        let start = sql.find('+').unwrap();
        assert_eq!(text(sql, start, operator_end(sql, start)), "+");
        let start = sql.find("OPERATOR").unwrap();
        assert_eq!(
            text(sql, start, operator_end(sql, start)),
            "OPERATOR(pg_catalog.+)"
        );
        let sql_lower = "select operator(pg_catalog.||)(a, b)";
        let start = sql_lower.find("operator").unwrap();
        assert_eq!(
            text(sql_lower, start, operator_end(sql_lower, start)),
            "operator(pg_catalog.||)"
        );
        let start = sql.find("pg_catalog.f").unwrap();
        assert_eq!(text(sql, start, reference_end(sql, start)), "pg_catalog.f");
        let start = sql.find("::bad").unwrap();
        assert_eq!(text(sql, start, cast_end(sql, start)), "::bad");
        let start = sql.find("CAST(").unwrap();
        assert_eq!(text(sql, start, cast_end(sql, start)), "CAST(x AS bad)");
        let start = sql.find("date '").unwrap();
        assert_eq!(text(sql, start, cast_end(sql, start)), "date 'today'");
    }

    #[test]
    fn literal_spans_cover_the_whole_constant() {
        let cases = [
            ("'happy'", "'happy'"),
            ("'it''s' = x", "'it''s'"),
            ("E'a\\'b' || x", "E'a\\'b'"),
            ("U&'d\\0061t' x", "U&'d\\0061t'"),
            ("$$a'b$$ x", "$$a'b$$"),
            ("$t$a$$b$t$ x", "$t$a$$b$t$"),
            ("'a'\n  'b' x", "'a'\n  'b'"),
            ("'a' 'b'", "'a'"),
        ];
        for (sql, expected) in cases {
            assert_eq!(text(sql, 0, literal_end(sql, 0)), expected, "{sql}");
        }
        assert_eq!(literal_end("x", 0), None);
        assert_eq!(literal_end("'open", 0), None);
    }

    #[test]
    fn star_spans_cover_the_qualifier() {
        let sql = r#"select *, t.*, "My ""T""" . *, s.t.* from t"#;
        let start = sql.find('*').unwrap();
        assert_eq!(text(sql, start, star_end(sql, start)), "*");
        let start = sql.find("t.*").unwrap();
        assert_eq!(text(sql, start, star_end(sql, start)), "t.*");
        let start = sql.find("\"My").unwrap();
        assert_eq!(text(sql, start, star_end(sql, start)), r#""My ""T""" . *"#);
        let start = sql.find("s.t").unwrap();
        assert_eq!(text(sql, start, star_end(sql, start)), "s.t.*");
    }

    #[test]
    fn non_ascii_identifiers() {
        let sql = "select größe, élan, café.öl, \"Ölstand\", x::straße from t";
        for name in ["größe", "élan", "café.öl", "\"Ölstand\""] {
            let start = sql.find(name).unwrap();
            assert_eq!(text(sql, start, reference_end(sql, start)), name);
            assert_eq!(
                text(sql, start, token_end(sql, start)),
                name.split('.').next().unwrap()
            );
        }
        let start = sql.find("::").unwrap();
        assert_eq!(text(sql, start, cast_end(sql, start)), "::straße");
    }
}
