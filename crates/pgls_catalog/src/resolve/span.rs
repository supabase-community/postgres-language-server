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
            while cursor < bytes.len()
                && (bytes[cursor].is_ascii_alphanumeric() || matches!(bytes[cursor], b'_' | b'$'))
            {
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

pub(super) fn token_end(sql: &str, start: usize) -> Option<usize> {
    let bytes = sql.as_bytes();
    let mut cursor = start;
    while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
        cursor += 1;
    }
    let first = *bytes.get(cursor)?;
    if first.is_ascii_alphabetic() || first == b'_' {
        while cursor < bytes.len()
            && (bytes[cursor].is_ascii_alphanumeric() || bytes[cursor] == b'_')
        {
            cursor += 1;
        }
        return Some(cursor);
    }
    if first == b'"' {
        return reference_end(sql, cursor);
    }
    while cursor < bytes.len()
        && !bytes[cursor].is_ascii_whitespace()
        && !bytes[cursor].is_ascii_alphanumeric()
        && !matches!(bytes[cursor], b'_' | b'(' | b')' | b',' | b';')
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
    use super::{cast_end, operator_end, reference_end};

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
}
