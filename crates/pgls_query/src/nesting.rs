//! Measures how deeply the messages of a serialized parse tree nest, without recursion.
//!
//! prost decodes, clones and drops a tree recursively, and so do the walkers of the parse tree
//! (the formatter, the catalog, the lint rules). A tree whose depth grows with the input could
//! overflow the stack of any of them, which aborts the process. [`check_nesting`] rejects such a
//! tree before it is decoded.

pgls_query_macros::nesting_codegen!();

/// How deeply the messages of a parse tree may nest. Each level of an expression takes two: the
/// `Node` and the node it holds, e.g. an `A_Expr`.
///
/// This accepts what Postgres accepts with its default `max_stack_depth` of 2 MB, which limits
/// the stack rather than the depth. Asked for the deepest statement of each shape, Postgres 15
/// accepts (the nesting in messages in parentheses):
///
/// | statement | Postgres accepts | nesting |
/// |---|---|---|
/// | `select 1 + 1 + ...`, `'a' \|\| 'a' \|\| ...`, `abs(abs(...))` | 4,091 terms | 8,191 |
/// | `select not not ... true` | 7,703 | 15,415 |
/// | `select (select (select ...))` | 2,111 | 12,675 |
/// | `select 1 union all select 1 ...` | 7,273 | 7,282 |
/// | `... join ... using (a)` | 6,545 | 13,103 |
/// | `case when ... else case ...` | 2,498 | 5,007 |
/// | `1::int::int...` | 13,096 | 26,201 |
///
/// Only the chain of casts is cut short: this limit allows about 8,190 casts, or `1 + 1 + ...`
/// with twice the terms Postgres accepts.
///
/// The deepest tree must fit into the stack of the threads that decode and walk it, see
/// `WORKER_STACK_SIZE_BYTES` in `pgls_workspace`.
///
/// The WASM build accepts 1,000 levels. Its stack can't grow: WebAssembly runs on the stack of the
/// JavaScript engine, whose size the browser decides, and Emscripten's own stack has 1 MiB (see
/// `build-wasm.sh`).
pub const MAX_NESTING: usize = if cfg!(target_os = "emscripten") {
    1_000
} else {
    16_384
};

/// The message nesting of the tree is deeper than [`MAX_NESTING`].
#[derive(Debug)]
pub(crate) struct TooDeep;

/// Checks that the messages of the serialized [`crate::protobuf::ParseResult`] in `buf` nest at
/// most `max` levels deep, and returns how deep they nest.
///
/// Malformed input is left to the decoder, which reports it.
pub(crate) fn check_nesting(buf: &[u8], max: usize) -> Result<usize, TooDeep> {
    // The enclosing messages of the current one, with the offset where each ends.
    let mut stack: Vec<(MessageId, usize)> = Vec::new();
    // The root counts as the first level.
    let mut deepest = 1;
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
                None => return Ok(deepest),
            }
        }

        let Some(tag) = read_varint(buf, &mut pos) else {
            return Ok(deepest);
        };
        let Ok(field) = u32::try_from(tag >> 3) else {
            return Ok(deepest);
        };
        match tag & 0b111 {
            // varint
            0 => {
                if read_varint(buf, &mut pos).is_none() {
                    return Ok(deepest);
                }
            }
            // 64-bit
            1 => pos += 8,
            // length-delimited: a message, a string, bytes or packed scalars
            2 => {
                let Some(len) =
                    read_varint(buf, &mut pos).and_then(|len| usize::try_from(len).ok())
                else {
                    return Ok(deepest);
                };
                let Some(field_end) = pos.checked_add(len).filter(|&field_end| field_end <= end)
                else {
                    return Ok(deepest);
                };
                match nested_message(message, field) {
                    Some(child) => {
                        let depth = stack.len() + 2;
                        if depth > max {
                            return Err(TooDeep);
                        }
                        deepest = deepest.max(depth);
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
            _ => return Ok(deepest),
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

    /// Runs `f` on a thread with the stack of the language server's threads: decoding a tree as
    /// deep as [`MAX_NESTING`] needs more than a test thread has.
    fn on_worker_stack<T: Send>(f: impl FnOnce() -> T + Send) -> T {
        std::thread::scope(|scope| {
            std::thread::Builder::new()
                .stack_size(64 * 1024 * 1024)
                .spawn_scoped(scope, f)
                .unwrap()
                .join()
                .unwrap()
        })
    }

    /// The deepest statements Postgres accepts with its default `max_stack_depth`. prost's own
    /// recursion limit of 100 rejected them.
    #[test]
    fn parses_what_postgres_accepts() {
        on_worker_stack(|| {
            for sql in [
                format!("select 1{};", " + 1".repeat(4_091)),
                format!("select 'a'{};", " || 'a'".repeat(4_091)),
                format!("select {}1{};", "abs(".repeat(4_091), ")".repeat(4_091)),
                format!("select {}true;", "not ".repeat(7_703)),
                format!("select {}1{};", "(select ".repeat(2_111), ")".repeat(2_111)),
                format!("select 1{};", " union all select 1".repeat(7_273)),
                format!(
                    "select {}1{};",
                    "case when true then 1 else ".repeat(2_498),
                    " end".repeat(2_498)
                ),
            ] {
                let result = crate::parse(&sql);
                assert!(result.is_ok(), "{}: {result:?}", &sql[..40]);
            }
        });
    }

    #[test]
    fn rejects_statements_nested_too_deeply() {
        on_worker_stack(|| {
            // `select 1` nests 9 messages deep, and each term adds two.
            let at_limit = |terms: usize| format!("select 1{};", " + 1".repeat(terms));
            assert!(crate::parse(&at_limit((MAX_NESTING - 9) / 2)).is_ok());
            assert_eq!(
                crate::parse(&at_limit((MAX_NESTING - 9) / 2 + 1)).unwrap_err(),
                crate::Error::TooDeep
            );
        });
    }
}
