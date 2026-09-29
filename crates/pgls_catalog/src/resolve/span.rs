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
