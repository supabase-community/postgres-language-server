use std::ffi::{CStr, CString};

use crate::NodeEnum;
use crate::bindings::*;
use crate::error::*;
use crate::nesting::{MAX_NESTING, TooDeep, check_nesting};
use crate::protobuf;

use prost::Message;

/// Parses the given SQL statement into the given abstract syntax tree.
///
/// # Example
///
/// ```rust
/// use pgls_query::parse;
///
/// let result = parse("SELECT * FROM contacts");
/// assert!(result.is_ok());
/// let result = result.unwrap();
/// assert_eq!(result.protobuf.stmts.len(), 1);
/// ```
pub fn parse(statement: &str) -> Result<ParseResult> {
    let input = CString::new(statement)?;
    let serialize_stack = statement.len().saturating_mul(STACK_PER_BYTE);
    // Emscripten has no threads.
    if serialize_stack <= INLINE_STACK || cfg!(target_os = "emscripten") {
        return parse_c_string(&input, INLINE_STACK);
    }
    // The thread has room for decoding the deepest tree too.
    let stack_size = serialize_stack
        .saturating_add(MAX_DECODE_STACK)
        .min(MAX_STACK);
    on_thread(stack_size, || parse_c_string(&input, MAX_DECODE_STACK))
}

/// The stack that libpg_query needs per byte of a statement, with a margin.
///
/// It serializes the parse tree recursively, before [`MAX_NESTING`] can be checked, so the
/// stack grows with the depth of the tree, which grows with the length of the statement. The
/// most measured is about 420 bytes per byte, for a chain of prefix operators such as
/// `select -+-+-+x`; `select 1 + 1 + ...` takes about 50.
const STACK_PER_BYTE: usize = 1024;

/// The stack that prost needs per level of nesting to decode a tree, with a margin. The most
/// measured is about 900 bytes, in a release build and in the dev profile, which builds this
/// crate with `opt-level = 1`.
const DECODE_STACK_PER_LEVEL: usize = 2 * 1024;

/// The stack for decoding the deepest tree that [`MAX_NESTING`] allows.
const MAX_DECODE_STACK: usize = MAX_NESTING * DECODE_STACK_PER_LEVEL;

/// A statement that needs at most this much stack, to serialize or to decode, is parsed or
/// decoded on the calling thread. Every thread of the language server has far more, see
/// `WORKER_STACK_SIZE_BYTES`.
const INLINE_STACK: usize = 1024 * 1024;

/// The largest stack a statement is parsed with: about 5 MB of `select 1 + 1 + ...`.
const MAX_STACK: usize = 1024 * 1024 * 1024;

/// Runs `f` on a thread with a stack of `stack_size` bytes.
///
/// A stack overflow aborts the process, so a long or deep statement, which may need more stack
/// than the calling thread has, is parsed or decoded on a thread of its own. The stack is virtual
/// memory, which is only committed as it is used.
fn on_thread<T: Send>(stack_size: usize, f: impl FnOnce() -> Result<T> + Send) -> Result<T> {
    std::thread::scope(|scope| {
        let handle = std::thread::Builder::new()
            .name("pgls_query::parse".to_string())
            .stack_size(stack_size)
            .spawn_scoped(scope, f)
            .map_err(|err| Error::TooLong(err.to_string()))?;
        handle
            .join()
            .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
    })
}

/// Parses `input`, decoding the tree on the calling thread if that needs at most `decode_stack`
/// bytes of stack.
fn parse_c_string(input: &CString, decode_stack: usize) -> Result<ParseResult> {
    let result = unsafe { pg_query_parse_protobuf(input.as_ptr()) };
    let parse_result = if !result.error.is_null() {
        let message = unsafe { CStr::from_ptr((*result.error).message) }
            .to_string_lossy()
            .to_string();
        Err(Error::Parse(message))
    } else {
        let data = unsafe {
            std::slice::from_raw_parts(result.parse_tree.data as *const u8, result.parse_tree.len)
        };
        let stderr = unsafe { CStr::from_ptr(result.stderr_buffer) }
            .to_string_lossy()
            .to_string();
        // prost's own recursion limit is off: it would reject a tree that is 100 messages deep,
        // e.g. `select 1 + 1 + ...` with 50 terms.
        check_nesting(data, MAX_NESTING)
            .map_err(|TooDeep| Error::TooDeep)
            .and_then(|depth| {
                let decode = || protobuf::ParseResult::decode(data).map_err(Error::Decode);
                let needed = depth.saturating_mul(DECODE_STACK_PER_LEVEL);
                if needed <= decode_stack || cfg!(target_os = "emscripten") {
                    decode()
                } else {
                    on_thread(needed.saturating_add(INLINE_STACK), decode)
                }
            })
            .map(|result| ParseResult::new(result, stderr))
    };
    unsafe { pg_query_free_protobuf_parse_result(result) };
    parse_result
}

/// The result of parsing a SQL query
#[derive(Debug)]
pub struct ParseResult {
    /// The parsed protobuf result
    pub protobuf: protobuf::ParseResult,
    /// Warnings captured during parsing
    pub warnings: Vec<String>,
}

impl ParseResult {
    /// Create a new ParseResult
    pub fn new(protobuf: protobuf::ParseResult, stderr: String) -> Self {
        let warnings = stderr
            .lines()
            .filter_map(|l| {
                if l.starts_with("WARNING") {
                    Some(l.trim().into())
                } else {
                    None
                }
            })
            .collect();

        Self { protobuf, warnings }
    }

    pub fn deparse(&self) -> Result<String> {
        crate::deparse(&self.protobuf)
    }

    pub fn stmts(&self) -> Vec<&NodeEnum> {
        self.protobuf
            .stmts
            .iter()
            .filter_map(|s| s.stmt.as_ref().and_then(|s| s.node.as_ref()))
            .collect()
    }

    pub fn stmts_mut(&mut self) -> Vec<&mut NodeEnum> {
        self.protobuf
            .stmts
            .iter_mut()
            .filter_map(|s| s.stmt.as_mut().and_then(|s| s.node.as_mut()))
            .collect()
    }

    /// Returns a reference to the root node of the parse tree.
    ///
    /// Returns None if there is not exactly one statement in the parse result.
    pub fn root(&self) -> Option<&NodeEnum> {
        if self.protobuf.stmts.len() != 1 {
            return None;
        }

        // Get the first (and only) statement
        let raw_stmt = &self.protobuf.stmts[0];

        // Navigate: RawStmt -> Node -> NodeEnum
        raw_stmt.stmt.as_ref().and_then(|stmt| stmt.node.as_ref())
    }

    /// Consumes the ParseResult and returns the root node of the parse tree.
    ///
    /// Returns None if there is not exactly one statement in the parse result.
    /// This method avoids cloning by taking ownership of the ParseResult.
    pub fn into_root(self) -> Option<NodeEnum> {
        if self.protobuf.stmts.len() != 1 {
            return None;
        }

        // Extract the first (and only) statement by taking ownership
        let raw_stmt = self.protobuf.stmts.into_iter().next()?;

        // Navigate: RawStmt -> Node -> NodeEnum
        raw_stmt.stmt.and_then(|stmt| stmt.node)
    }

    /// Returns a mutable reference to the root node of the parse tree.
    ///
    /// Returns None if there is not exactly one statement in the parse result.
    pub fn root_mut(&mut self) -> Option<&mut NodeEnum> {
        if self.protobuf.stmts.len() != 1 {
            return None;
        }

        // Get the first (and only) statement
        let raw_stmt = &mut self.protobuf.stmts[0];

        // Navigate: RawStmt -> Node -> NodeEnum
        raw_stmt.stmt.as_mut().and_then(|stmt| stmt.node.as_mut())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Error, parse};

    #[test]
    fn it_parses_parameter_queries() {
        assert!(parse("select $0 + $1 + $2 + $3 + $4 + $5").is_ok());
    }

    /// libpg_query needs more stack for these than a thread with the default 2 MiB has, so they
    /// aborted the process with a stack overflow.
    #[test]
    fn long_statements_get_the_stack_they_need() {
        let statements = [
            format!("select 1{};", " + abs(1)".repeat(50_000)),
            format!("select {}x;", "-+".repeat(4_000)),
        ];
        let results = std::thread::Builder::new()
            .stack_size(2 * 1024 * 1024)
            .spawn(move || statements.map(|sql| parse(&sql).map(|_| ())))
            .unwrap()
            .join()
            .unwrap();
        // The first is too deep to decode, but parsing returns.
        assert_eq!(results, [Err(Error::TooDeep), Ok(())]);
    }

    /// A short statement can nest deeply: decoding these needs about 1 MiB of stack.
    #[test]
    fn deep_trees_get_the_stack_they_need_to_decode() {
        let sql = format!("select 1{};", "+1".repeat(495));
        assert!(sql.len() * super::STACK_PER_BYTE <= super::INLINE_STACK);
        let result = std::thread::Builder::new()
            .stack_size(512 * 1024)
            .spawn(move || parse(&sql).map(|_| ()))
            .unwrap()
            .join()
            .unwrap();
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn long_statements_parse_on_their_own_thread() {
        let values = vec!["(1, 'a')"; 10_000].join(", ");
        let result = parse(&format!("insert into t values {values};")).unwrap();
        assert_eq!(result.stmts().len(), 1);
    }
}
