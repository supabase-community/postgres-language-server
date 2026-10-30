//! Measures how deeply the messages of a serialized parse tree nest, without recursion.
//!
//! prost decodes, clones and drops a tree recursively, and so do the walkers of the parse tree
//! (the formatter, the catalog, the lint rules). A tree whose depth grows with the input could
//! overflow the stack of any of them, which aborts the process. [`check_nesting`] rejects such a
//! tree before it is decoded.

pgls_query_macros::nesting_codegen!();

/// How deeply the messages of a parse tree may nest. Each level of an expression takes two: the
/// `Node` and the node it holds, e.g. an `A_Expr`. With this limit, `select 1 + 1 + ...` may have
/// about 500 terms, a chain of `union`s about 1,000 and nested subqueries about 140.
///
/// The deepest tree must fit into the stack of the threads that decode and walk it. In a release
/// build, the language server needs about 1.1 MiB of stack for such a statement (hover, inlay
/// hints, diagnostics, code actions and formatting), well within the 8 MiB of its threads.
/// Postgres accepts about 4,000 terms with its default `max_stack_depth` of 2 MB.
pub const MAX_NESTING: usize = 1_000;

/// The message nesting of the tree is deeper than [`MAX_NESTING`].
#[derive(Debug)]
pub(crate) struct TooDeep;

/// Checks that the messages of the serialized [`crate::protobuf::ParseResult`] in `buf` nest at
/// most `max` levels deep.
///
/// Malformed input is left to the decoder, which reports it.
pub(crate) fn check_nesting(buf: &[u8], max: usize) -> Result<(), TooDeep> {
    // The enclosing messages of the current one, with the offset where each ends.
    let mut stack: Vec<(MessageId, usize)> = Vec::new();
    let mut message = PARSE_RESULT;
    let mut end = buf.len();
    let mut pos = 0;

    loop {
        if pos >= end {
            match stack.pop() {
                Some((parent, parent_end)) => {
                    message = parent;
                    end = parent_end;
                    continue;
                }
                None => return Ok(()),
            }
        }

        let Some(tag) = read_varint(buf, &mut pos) else {
            return Ok(());
        };
        let Ok(field) = u32::try_from(tag >> 3) else {
            return Ok(());
        };
        match tag & 0b111 {
            // varint
            0 => {
                if read_varint(buf, &mut pos).is_none() {
                    return Ok(());
                }
            }
            // 64-bit
            1 => pos += 8,
            // length-delimited: a message, a string, bytes or packed scalars
            2 => {
                let Some(len) =
                    read_varint(buf, &mut pos).and_then(|len| usize::try_from(len).ok())
                else {
                    return Ok(());
                };
                let Some(field_end) = pos.checked_add(len).filter(|&field_end| field_end <= end)
                else {
                    return Ok(());
                };
                match nested_message(message, field) {
                    Some(child) => {
                        // The root counts as the first level.
                        if stack.len() + 2 > max {
                            return Err(TooDeep);
                        }
                        stack.push((message, end));
                        message = child;
                        end = field_end;
                    }
                    None => pos = field_end,
                }
            }
            // 32-bit
            5 => pos += 4,
            // groups aren't used by pg_query.proto
            _ => return Ok(()),
        }
    }
}

fn read_varint(buf: &[u8], pos: &mut usize) -> Option<u64> {
    let mut value = 0u64;
    for shift in (0..64).step_by(7) {
        let byte = *buf.get(*pos)?;
        *pos += 1;
        value |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Some(value);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use prost::Message;

    use super::*;

    fn serialized(sql: &str) -> Vec<u8> {
        crate::parse(sql).unwrap().protobuf.encode_to_vec()
    }

    /// The message depth as prost counts it: one for each message below the root.
    fn depth(buf: &[u8]) -> usize {
        (1..).find(|&max| check_nesting(buf, max).is_ok()).unwrap()
    }

    #[test]
    fn measures_the_deepest_path() {
        // ParseResult > RawStmt > Node > SelectStmt > Node > ResTarget > Node > A_Const > Integer
        assert_eq!(depth(&serialized("select 1")), 9);
        // Each `+` adds a Node and an A_Expr.
        assert_eq!(depth(&serialized("select 1 + 1")), 11);
        assert_eq!(depth(&serialized("select 1 + 1 + 1")), 13);
        // Lists don't nest.
        assert_eq!(
            depth(&serialized("select 1, 2, 3")),
            depth(&serialized("select 1"))
        );
        // `SelectStmt.larg` holds a `SelectStmt` directly.
        assert_eq!(depth(&serialized("select 1 union select 1")), 10);
    }

    #[test]
    fn ignores_strings_and_scalars() {
        let long = format!("select '{}' + 1", "x".repeat(10_000));
        assert_eq!(depth(&serialized(&long)), 11);
    }

    /// prost's own limit of 100 rejected these.
    #[test]
    fn parses_deeply_nested_statements() {
        for sql in [
            format!("select 1{};", " + abs(1)".repeat(400)),
            format!("select 'a'{};", " || 'a'".repeat(400)),
            format!("select 1{};", " union select 1".repeat(900)),
            format!("select {}1{};", "(select ".repeat(130), ")".repeat(130)),
        ] {
            let result = crate::parse(&sql);
            assert!(result.is_ok(), "{}: {result:?}", &sql[..40]);
        }
    }

    #[test]
    fn rejects_statements_nested_too_deeply() {
        let result = crate::parse(&format!("select 1{};", " + abs(1)".repeat(600)));
        assert_eq!(result.unwrap_err(), crate::Error::TooDeep);
    }
}
