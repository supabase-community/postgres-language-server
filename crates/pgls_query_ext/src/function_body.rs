//! The statements in the body of a SQL function. They are statements of their own, embedded in
//! the `CREATE FUNCTION` statement:
//!
//! - `AS '...'`, `AS E'...'` and `AS $tag$...$tag$`: the string constant, split into statements
//!   like Postgres splits it ([`pg_parse_query`]). Its value can differ from its text: `''` and
//!   backslash escapes stand for other characters, which [`OffsetMap`] maps.
//! - `BEGIN ATOMIC ... END`: each statement up to its `;`.
//! - `RETURN expr`: the expression as `select expr`, which has the same length.
//!
//! [`pg_parse_query`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/tcop/postgres.c#L604

use pgls_query::protobuf::{CreateFunctionStmt, ScanToken, Token};
use pgls_text_size::{TextRange, TextSize};

use crate::utils::find_option_value;

/// A statement embedded in another one, like a statement of a SQL function body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddedStatement {
    /// The text that is parsed. It differs from the source text when the source escapes
    /// characters, and for a `RETURN` body.
    pub sql: String,
    /// Where the statement is in the text of the statement that contains it.
    pub range: TextRange,
    /// Maps offsets in `sql` to offsets in the source text at `range`.
    pub offsets: OffsetMap,
}

/// Maps offsets between the text of a statement and its source, both relative to their start.
/// The two differ where the source escapes characters, like `''` for `'` in a string constant,
/// or where the statement replaces a part of the source, like `select` for `return`. Without
/// those, an offset is the same in both, and the map is empty.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OffsetMap {
    /// In ascending order, and they don't overlap.
    escapes: Vec<Escape>,
}

/// An escape in the source, and the bytes of the statement it stands for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Escape {
    statement: TextRange,
    source: TextRange,
}

impl OffsetMap {
    /// Adds an escape of `source_len` bytes that stands for the `statement_len` bytes at
    /// `statement_offset` in the statement. Escapes must be added in order.
    fn push_escape(
        &mut self,
        statement_offset: TextSize,
        statement_len: TextSize,
        source_len: TextSize,
    ) {
        debug_assert!(
            self.escapes
                .last()
                .is_none_or(|last| last.statement.end() <= statement_offset),
            "escapes are added in order"
        );
        let source_offset = self.to_source(statement_offset);
        self.escapes.push(Escape {
            statement: TextRange::at(statement_offset, statement_len),
            source: TextRange::at(source_offset, source_len),
        });
    }

    /// Whether the text of the statement is the source text as it is, so that offsets and
    /// edits apply to both.
    pub fn is_verbatim(&self) -> bool {
        self.escapes.is_empty()
    }

    /// The offset in the source of `offset` in the statement. An offset inside the bytes that an
    /// escape stands for maps to the start of the escape.
    pub fn to_source(&self, offset: TextSize) -> TextSize {
        let index = self
            .escapes
            .partition_point(|escape| escape.statement.start() <= offset);
        let Some(escape) = index.checked_sub(1).and_then(|i| self.escapes.get(i)) else {
            return offset;
        };
        if offset < escape.statement.end() {
            escape.source.start()
        } else {
            offset - escape.statement.end() + escape.source.end()
        }
    }

    /// The range in the source of `range` in the statement.
    pub fn range_to_source(&self, range: TextRange) -> TextRange {
        TextRange::new(self.to_source(range.start()), self.to_source(range.end()))
    }

    /// The offset in the statement of `offset` in the source. An offset inside an escape maps to
    /// the start of what it stands for.
    pub fn to_statement(&self, offset: TextSize) -> TextSize {
        let index = self
            .escapes
            .partition_point(|escape| escape.source.start() <= offset);
        let Some(escape) = index.checked_sub(1).and_then(|i| self.escapes.get(i)) else {
            return offset;
        };
        if offset < escape.source.end() {
            escape.statement.start()
        } else {
            offset - escape.source.end() + escape.statement.end()
        }
    }

    /// The map of the part of the statement at `range`, relative to its start. `range` doesn't
    /// cut through what an escape stands for, since it is on character boundaries.
    fn slice(&self, range: TextRange) -> OffsetMap {
        let source_start = self.to_source(range.start());
        OffsetMap {
            escapes: self
                .escapes
                .iter()
                .filter(|escape| range.contains_range(escape.statement))
                .map(|escape| Escape {
                    statement: escape.statement - range.start(),
                    source: escape.source - source_start,
                })
                .collect(),
        }
    }
}

/// The statements of the body of the SQL function that `stmt` creates, in order. `sql` is the
/// text of `stmt`. Empty if `stmt` doesn't create a SQL function, if the body is empty, or if
/// the body can't be found in `sql`, like a `U&'...'` string.
///
/// A string body with a single statement, or one that doesn't parse, is a single statement over
/// the whole body, so that its syntax error is reported for the body.
pub fn sql_function_body(stmt: &pgls_query::NodeEnum, sql: &str) -> Vec<EmbeddedStatement> {
    let pgls_query::NodeEnum::CreateFunctionStmt(create_fn) = stmt else {
        return Vec::new();
    };
    if !is_sql_function(create_fn) {
        return Vec::new();
    }
    let Some(tokens) = tokens(sql) else {
        return Vec::new();
    };
    match create_fn
        .sql_body
        .as_deref()
        .and_then(|body| body.node.as_ref())
    {
        Some(pgls_query::NodeEnum::List(body)) => atomic_body(body, &tokens, sql),
        Some(pgls_query::NodeEnum::ReturnStmt(_)) => {
            return_body(&tokens, sql).into_iter().collect()
        }
        Some(_) => Vec::new(),
        None => string_body(create_fn, &tokens, sql)
            .map(|body| body.statements())
            .unwrap_or_default(),
    }
}

/// Where the string constant of the body of `create_fn` is in `sql`, the text of the statement,
/// and how offsets in the body's value map to it. Works for functions in any language. `None`
/// for `BEGIN ATOMIC` and `RETURN` bodies, and if the constant can't be found, like for a
/// `U&'...'` string.
pub fn function_body_location(
    create_fn: &CreateFunctionStmt,
    sql: &str,
) -> Option<(TextRange, OffsetMap)> {
    let body = string_body(create_fn, &tokens(sql)?, sql)?;
    Some((body.range, body.offsets))
}

/// Whether the function is written in SQL: `LANGUAGE sql`, or a `BEGIN ATOMIC` or `RETURN` body,
/// which only SQL functions have.
fn is_sql_function(create_fn: &CreateFunctionStmt) -> bool {
    create_fn.sql_body.is_some()
        || find_option_value(create_fn, "language")
            .is_some_and(|language| language.eq_ignore_ascii_case("sql"))
}

/// The tokens of `sql` without comments.
fn tokens(sql: &str) -> Option<Vec<ScanToken>> {
    Some(
        pgls_query::scan(sql)
            .ok()?
            .tokens
            .into_iter()
            .filter(|token| !matches!(token.token(), Token::SqlComment | Token::CComment))
            .collect(),
    )
}

fn token_range(token: &ScanToken) -> Option<TextRange> {
    Some(TextRange::new(
        TextSize::try_from(usize::try_from(token.start).ok()?).ok()?,
        TextSize::try_from(usize::try_from(token.end).ok()?).ok()?,
    ))
}

/// A string body: its value, where the value is in the statement, and how they map.
struct StringBody {
    value: String,
    range: TextRange,
    offsets: OffsetMap,
}

impl StringBody {
    /// The statements of the body, split like Postgres splits it.
    fn statements(self) -> Vec<EmbeddedStatement> {
        let whole = TextRange::up_to(TextSize::of(self.value.as_str()));
        let ranges = match pgls_query::parse(&self.value) {
            Ok(parsed) if parsed.protobuf.stmts.len() > 1 => parsed
                .protobuf
                .stmts
                .iter()
                .filter_map(|stmt| statement_range(&self.value, stmt))
                .collect(),
            // An empty body has no statements.
            Ok(parsed) if parsed.protobuf.stmts.is_empty() => Vec::new(),
            _ => vec![whole],
        };
        ranges
            .into_iter()
            .map(|range| EmbeddedStatement {
                sql: self.value[range].to_owned(),
                range: self.offsets.range_to_source(range) + self.range.start(),
                offsets: self.offsets.slice(range),
            })
            .collect()
    }
}

/// The range of a statement of a multi-statement body, without the whitespace around it and with
/// its semicolon, like top-level statements.
fn statement_range(body: &str, stmt: &pgls_query::protobuf::RawStmt) -> Option<TextRange> {
    let start = usize::try_from(stmt.stmt_location).ok()?;
    // A length of 0 means the rest of the body.
    let end = match usize::try_from(stmt.stmt_len).ok()? {
        0 => body.len(),
        len => start + len,
    };
    let text = body.get(start..end)?;
    let start = start + (text.len() - text.trim_start().len());
    let mut end = start + text.trim().len();
    if body.get(end..)?.starts_with(';') {
        end += 1;
    }
    Some(TextRange::new(
        TextSize::try_from(start).ok()?,
        TextSize::try_from(end).ok()?,
    ))
}

/// The string constant after an `AS` token whose value is the `AS` option of the function. A
/// default of a parameter can have the same text, so the value alone doesn't locate it.
fn string_body(
    create_fn: &CreateFunctionStmt,
    tokens: &[ScanToken],
    sql: &str,
) -> Option<StringBody> {
    let value = find_option_value(create_fn, "as")?;
    tokens.windows(2).find_map(|pair| {
        let [as_token, string] = pair else {
            return None;
        };
        if as_token.token() != Token::As || string.token() != Token::Sconst {
            return None;
        }
        string_constant(&value, token_range(string)?, sql)
    })
}

/// The body in the string constant at `range` of `sql`, if its value is `value`.
fn string_constant(value: &str, range: TextRange, sql: &str) -> Option<StringBody> {
    let text = sql.get(usize::from(range.start())..usize::from(range.end()))?;
    if let Some(after_dollar) = text.strip_prefix('$') {
        // `$tag$...$tag$`, where the tag can be empty.
        let tag_len = after_dollar.find('$')? + 2;
        let inner = text.get(tag_len..text.len().checked_sub(tag_len)?)?;
        if inner != value {
            return None;
        }
        return Some(StringBody {
            value: inner.to_owned(),
            range: TextRange::at(
                range.start() + TextSize::try_from(tag_len).ok()?,
                TextSize::of(inner),
            ),
            offsets: OffsetMap::default(),
        });
    }
    let (prefix, backslash_escapes) = match text.as_bytes() {
        [b'e' | b'E', b'\'', ..] => (1, true),
        [b'\'', ..] => (0, false),
        // `U&'...'` and `B'...'`.
        _ => return None,
    };
    let inner = text.get(prefix + 1..)?.strip_suffix('\'')?;
    let (unescaped, offsets) = unescape(inner, backslash_escapes)?;
    if unescaped != value {
        return None;
    }
    Some(StringBody {
        value: unescaped,
        range: TextRange::at(
            range.start() + TextSize::try_from(prefix + 1).ok()?,
            TextSize::of(inner),
        ),
        offsets,
    })
}

/// The value of the inside of a string constant, and where its escapes are. With
/// `backslash_escapes`, the escapes of `E'...'` strings are read like the `xe` rules of
/// Postgres' lexer ([`scan.l`]). `None` if a quote ends the constant before its end, like in a
/// constant continued on the next line, or if the value isn't valid UTF-8.
///
/// [`scan.l`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L262
fn unescape(inner: &str, backslash_escapes: bool) -> Option<(String, OffsetMap)> {
    let bytes = inner.as_bytes();
    let mut value = Vec::with_capacity(bytes.len());
    let mut offsets = OffsetMap::default();
    let mut cursor = 0;
    while let Some(&byte) = bytes.get(cursor) {
        let (decoded, len): (Vec<u8>, usize) = match byte {
            b'\'' if bytes.get(cursor + 1) == Some(&b'\'') => (vec![b'\''], 2),
            b'\'' => return None,
            b'\\' if backslash_escapes => escape_sequence(bytes.get(cursor..)?)?,
            _ => {
                value.push(byte);
                cursor += 1;
                continue;
            }
        };
        offsets.push_escape(
            TextSize::try_from(value.len()).ok()?,
            TextSize::try_from(decoded.len()).ok()?,
            TextSize::try_from(len).ok()?,
        );
        value.extend(decoded);
        cursor += len;
    }
    Some((String::from_utf8(value).ok()?, offsets))
}

/// The bytes that the backslash escape at the start of `text` stands for, and its length.
fn escape_sequence(text: &[u8]) -> Option<(Vec<u8>, usize)> {
    let digits = |start: usize, max: usize, radix: u32| {
        text.iter()
            .skip(start)
            .take(max)
            .take_while(|byte| char::from(**byte).is_digit(radix))
            .count()
    };
    let number = |start: usize, len: usize, radix: u32| {
        u32::from_str_radix(
            std::str::from_utf8(text.get(start..start + len)?).ok()?,
            radix,
        )
        .ok()
    };
    let escaped = *text.get(1)?;
    match escaped {
        b'0'..=b'7' => {
            let len = digits(1, 3, 8);
            Some((vec![(number(1, len, 8)? & 0xFF) as u8], 1 + len))
        }
        b'x' if digits(2, 2, 16) > 0 => {
            let len = digits(2, 2, 16);
            Some((vec![u8::try_from(number(2, len, 16)?).ok()?], 2 + len))
        }
        b'u' | b'U' => {
            let len = if escaped == b'u' { 4 } else { 8 };
            if digits(2, len, 16) != len {
                return None;
            }
            // Surrogate pairs are not supported.
            let c = char::from_u32(number(2, len, 16)?)?;
            Some((c.to_string().into_bytes(), 2 + len))
        }
        b'b' => Some((vec![0x08], 2)),
        b'f' => Some((vec![0x0C], 2)),
        b'n' => Some((vec![b'\n'], 2)),
        b'r' => Some((vec![b'\r'], 2)),
        b't' => Some((vec![b'\t'], 2)),
        b'v' => Some((vec![0x0B], 2)),
        // Any other character stands for itself.
        _ => {
            let len = std::str::from_utf8(text.get(1..)?)
                .ok()?
                .chars()
                .next()?
                .len_utf8();
            Some((text.get(1..1 + len)?.to_vec(), 1 + len))
        }
    }
}

/// The statements of a `BEGIN ATOMIC ... END` body. Each ends with a `;`. The body ends at the
/// `END` that starts a statement; the other `END`s close `CASE` expressions. Empty if the number
/// of statements doesn't match the parsed body.
fn atomic_body(
    body: &pgls_query::protobuf::List,
    tokens: &[ScanToken],
    sql: &str,
) -> Vec<EmbeddedStatement> {
    let Some(start) = tokens
        .windows(2)
        .position(|pair| pair[0].token() == Token::BeginP && pair[1].token() == Token::Atomic)
    else {
        return Vec::new();
    };
    let mut ranges = Vec::new();
    let mut statement_start: Option<&ScanToken> = None;
    let (mut parens, mut cases) = (0usize, 0usize);
    for token in tokens.iter().skip(start + 2) {
        let Some(first) = statement_start else {
            match token.token() {
                Token::EndP => break,
                // An empty statement.
                Token::Ascii59 => continue,
                Token::Case => cases += 1,
                _ => {}
            }
            statement_start = Some(token);
            continue;
        };
        match token.token() {
            Token::Ascii40 => parens += 1,
            Token::Ascii41 => parens = parens.saturating_sub(1),
            Token::Case => cases += 1,
            Token::EndP => cases = cases.saturating_sub(1),
            Token::Ascii59 if parens == 0 && cases == 0 => {
                if let (Some(first), Some(last)) = (token_range(first), token_range(token)) {
                    ranges.push(first.cover(last));
                }
                statement_start = None;
            }
            _ => {}
        }
    }
    // `BEGIN ATOMIC ... END` is a list that holds the list of statements.
    let expected = match body.items.as_slice() {
        [list] => match list.node.as_ref() {
            Some(pgls_query::NodeEnum::List(list)) => list.items.len(),
            _ => return Vec::new(),
        },
        _ => return Vec::new(),
    };
    if ranges.len() != expected {
        return Vec::new();
    }
    ranges
        .into_iter()
        .filter_map(|range| {
            Some(EmbeddedStatement {
                sql: sql
                    .get(usize::from(range.start())..usize::from(range.end()))?
                    .to_owned(),
                range,
                offsets: OffsetMap::default(),
            })
        })
        .collect()
}

/// The expression of a `RETURN` body, as a `SELECT` of the same length: `return x + 1` is
/// resolved as `select x + 1`.
fn return_body(tokens: &[ScanToken], sql: &str) -> Option<EmbeddedStatement> {
    let return_token = tokens.iter().find(|token| token.token() == Token::Return)?;
    let last = tokens
        .iter()
        .rev()
        .find(|token| token.token() != Token::Ascii59)?;
    let range = token_range(return_token)?.cover(token_range(last)?);
    let expression = sql.get(usize::try_from(return_token.end).ok()?..usize::from(range.end()))?;
    // `select` stands for `return`, which has the same length.
    let keyword = TextSize::of("select");
    let mut offsets = OffsetMap::default();
    offsets.push_escape(0.into(), keyword, token_range(return_token)?.len());
    Some(EmbeddedStatement {
        sql: format!("select{expression}"),
        range,
        offsets,
    })
}

#[cfg(test)]
mod tests {
    use pgls_text_size::{TextRange, TextSize};

    use super::{OffsetMap, sql_function_body, unescape};

    /// The statements of the body: their text in `input`, and the text that is parsed.
    fn bodies(input: &str) -> Vec<(&str, String)> {
        let ast = pgls_query::parse(input).unwrap().into_root().unwrap();
        sql_function_body(&ast, input)
            .into_iter()
            .map(|body| (&input[body.range], body.sql))
            .collect()
    }

    /// The parsed text of the statements of the body.
    fn statements(input: &str) -> Vec<String> {
        bodies(input).into_iter().map(|(_, sql)| sql).collect()
    }

    #[test]
    fn string_bodies_are_split_like_postgres_splits_them() {
        // A single statement is the whole body.
        assert_eq!(
            statements("create function f() returns int language sql as $$\n  select 1;\n$$"),
            ["\n  select 1;\n"]
        );
        assert_eq!(
            statements(
                "create function f(a int) returns int language sql as $$\n  insert into t values (a);\n  select 1\n$$"
            ),
            ["insert into t values (a);", "select 1"]
        );
        assert_eq!(
            statements("create function f() returns int language sql as 'select 1; select 2;'"),
            ["select 1;", "select 2;"]
        );
        // An empty body has no statements.
        assert!(
            statements("create function f() returns void language sql as $$ -- nothing $$")
                .is_empty()
        );
        // A body that doesn't parse is reported as a whole.
        assert_eq!(
            statements("create function f() returns void language sql as 'select 1; selec 2'"),
            ["select 1; selec 2"]
        );
        // Other languages have no statements.
        assert!(
            statements("create function f() returns void language plpgsql as $$ begin end $$")
                .is_empty()
        );
    }

    #[test]
    fn string_bodies_are_found_by_their_token() {
        // The default has the same text as the body, and comes first.
        assert_eq!(
            bodies(
                "create function f(t text default 'select 1') returns int language sql as 'select 1'"
            ),
            [("select 1", "select 1".to_string())]
        );
        assert_eq!(
            bodies("create function f() returns int language sql as $body$ select 1 $body$"),
            [(" select 1 ", " select 1 ".to_string())]
        );
        // `U&'...'` strings are not supported.
        assert!(bodies("create function f() returns int language sql as U&'select 1'").is_empty());
    }

    #[test]
    fn doubled_quotes_are_mapped() {
        let input = "create function f(p text) returns text language sql as 'select ''a'' || p'";
        let ast = pgls_query::parse(input).unwrap().into_root().unwrap();
        let [body] = sql_function_body(&ast, input).try_into().unwrap();
        assert_eq!(body.sql, "select 'a' || p");
        assert_eq!(&input[body.range], "select ''a'' || p");
        let p = TextSize::of("select 'a' || ");
        assert_eq!(
            &input[body.range][usize::from(body.offsets.to_source(p))..],
            "p"
        );
        assert!(!body.offsets.is_verbatim());
    }

    #[test]
    fn escapes_are_mapped_in_each_statement() {
        let input = "create function f(p text) returns text language sql as 'select ''a''; select p || ''b'''";
        let ast = pgls_query::parse(input).unwrap().into_root().unwrap();
        let [first, second] = sql_function_body(&ast, input).try_into().unwrap();
        assert_eq!(first.sql, "select 'a';");
        assert_eq!(&input[first.range], "select ''a'';");
        assert_eq!(second.sql, "select p || 'b'");
        assert_eq!(&input[second.range], "select p || ''b''");
        let b = TextSize::of("select p || '");
        assert_eq!(
            &input[second.range][usize::from(second.offsets.to_source(b))..],
            "b''"
        );
        assert_eq!(second.offsets.to_source(TextSize::of("select p")), 8.into());
    }

    #[test]
    fn escape_string_bodies_are_mapped() {
        let input = r"create function f(p text) returns text language sql as E'select ''\u00e9'' ||\n E\'\\t\' || p'";
        let ast = pgls_query::parse(input).unwrap().into_root().unwrap();
        let [body] = sql_function_body(&ast, input).try_into().unwrap();
        assert_eq!(body.sql, "select '\u{e9}' ||\n E'\\t' || p");
        let p = TextSize::of("select '\u{e9}' ||\n E'\\t' || ");
        let source = &input[body.range];
        assert_eq!(&source[usize::from(body.offsets.to_source(p))..], "p");
        assert_eq!(
            body.offsets
                .to_statement(TextSize::of(source) - TextSize::from(1)),
            p
        );
    }

    #[test]
    fn unescapes_like_postgres() {
        let value = |inner: &str| unescape(inner, true).map(|(value, _)| value);
        assert_eq!(value(r"a\tb\x41\101\q\\"), Some("a\tbAAq\\".to_string()));
        assert_eq!(value(r"\U0001F600 é"), Some("\u{1F600} \u{e9}".to_string()));
        assert_eq!(value(r"\xc3\xa9"), Some("\u{e9}".to_string()));
        // Not UTF-8, a `\u` without four digits, and a quote that ends the constant.
        assert_eq!(value(r"\xff"), None);
        assert_eq!(value(r"\u12"), None);
        assert_eq!(value("a'\n'b"), None);
        // Without backslash escapes, backslashes are themselves.
        assert_eq!(
            unescape(r"a\n''", false).map(|(value, _)| value),
            Some(r"a\n'".to_string())
        );
    }

    #[test]
    fn atomic_bodies_have_a_statement_each() {
        assert_eq!(
            bodies(
                "create function f(p int) returns int language sql begin atomic \
                 select case when p > 0 then 1 else 2 end; select (p); end"
            ),
            [
                (
                    "select case when p > 0 then 1 else 2 end;",
                    "select case when p > 0 then 1 else 2 end;".to_string()
                ),
                ("select (p);", "select (p);".to_string()),
            ]
        );
        // The language defaults to SQL.
        assert_eq!(
            bodies("create function f(p int) returns int begin atomic; select p; end"),
            [("select p;", "select p;".to_string())]
        );
    }

    #[test]
    fn return_bodies_are_selects() {
        let input = "create function f(p int) returns int return p + 1;";
        let ast = pgls_query::parse(input).unwrap().into_root().unwrap();
        let [body] = sql_function_body(&ast, input).try_into().unwrap();
        assert_eq!(&input[body.range], "return p + 1");
        assert_eq!(body.sql, "select p + 1");
        // The text is not the source, but offsets are the same.
        assert!(!body.offsets.is_verbatim());
        assert_eq!(body.offsets.to_source(TextSize::of("select ")), 7.into());
    }

    #[test]
    fn maps_offsets_through_doubled_quotes() {
        // The statement `a'b'c`, written `a''b''c` in the source.
        let mut offsets = OffsetMap::default();
        offsets.push_escape(1.into(), 1.into(), 2.into());
        offsets.push_escape(3.into(), 1.into(), 2.into());
        let source = [0, 1, 3, 4, 6, 7];
        for (statement, source) in source.into_iter().enumerate() {
            let statement = TextSize::try_from(statement).unwrap();
            let source = TextSize::from(source);
            assert_eq!(offsets.to_source(statement), source);
            assert_eq!(offsets.to_statement(source), statement);
        }
        // The second quote of an escape is the quote.
        assert_eq!(offsets.to_statement(2.into()), 1.into());
        assert_eq!(offsets.to_statement(5.into()), 3.into());
        assert_eq!(
            offsets.range_to_source(TextRange::new(1.into(), 4.into())),
            TextRange::new(1.into(), 6.into())
        );

        let identity = OffsetMap::default();
        assert_eq!(identity.to_source(5.into()), 5.into());
        assert_eq!(identity.to_statement(5.into()), 5.into());
    }

    #[test]
    fn maps_offsets_through_longer_escapes() {
        // The statement `aéb` (`é` takes 2 bytes), written `a\u00e9b` in the source.
        let mut offsets = OffsetMap::default();
        offsets.push_escape(1.into(), 2.into(), 6.into());
        assert_eq!(offsets.to_source(1.into()), 1.into());
        assert_eq!(offsets.to_source(3.into()), 7.into());
        assert_eq!(offsets.to_source(4.into()), 8.into());
        assert_eq!(offsets.to_statement(7.into()), 3.into());
        assert_eq!(offsets.to_statement(4.into()), 1.into());
        assert_eq!(offsets.to_statement(0.into()), 0.into());
    }
}
