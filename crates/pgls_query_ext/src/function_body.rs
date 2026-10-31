//! The statements in the body of a SQL function. They are statements of their own, embedded in
//! the `CREATE FUNCTION` statement:
//!
//! - `AS '...'`, `AS E'...'`, `AS U&'...'` and `AS $tag$...$tag$`: the string constant, split
//!   into statements like Postgres splits it ([`pg_parse_query`]). Its value can differ from its
//!   text: `''`, backslash escapes and Unicode escapes stand for other characters, which
//!   [`OffsetMap`] maps.
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
/// or where the statement replaces a part of the source, like `select` for `return`. The gap
/// between the parts of a string constant continued on the next line is an escape that stands
/// for nothing. Without those, an offset is the same in both, and the map is empty.
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

    /// The range in the source of `range` in the statement. It doesn't include the gaps of a
    /// continued string constant at its start and end.
    pub fn range_to_source(&self, range: TextRange) -> TextRange {
        let start = self.to_source(range.start());
        TextRange::new(start, self.to_source_end(range.end()).max(start))
    }

    /// Like [`Self::to_source`], but for the end of a range: the offset in the source before a
    /// gap at `offset` rather than after it.
    fn to_source_end(&self, offset: TextSize) -> TextSize {
        let index = self
            .escapes
            .partition_point(|escape| escape.statement.start() < offset);
        let Some(escape) = index.checked_sub(1).and_then(|i| self.escapes.get(i)) else {
            return offset;
        };
        if offset < escape.statement.end() {
            escape.source.end()
        } else {
            offset - escape.statement.end() + escape.source.end()
        }
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
    /// cut through what an escape stands for, since it is on character boundaries. A gap at the
    /// start or the end of `range` is outside of it.
    fn slice(&self, range: TextRange) -> OffsetMap {
        let source_start = self.to_source(range.start());
        OffsetMap {
            escapes: self
                .escapes
                .iter()
                .filter(|escape| {
                    range.contains_range(escape.statement)
                        && (!escape.statement.is_empty()
                            || (range.start() < escape.statement.start()
                                && escape.statement.end() < range.end()))
                })
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
/// the body can't be found in `sql`, like a string continued on the next line.
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
        Some(pgls_query::NodeEnum::ReturnStmt(stmt)) => {
            return_body(stmt, &tokens, sql).into_iter().collect()
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
    tokens.iter().enumerate().find_map(|(i, as_token)| {
        if as_token.token() != Token::As {
            return None;
        }
        let string = tokens.get(i + 1)?;
        match string.token() {
            Token::Sconst => string_constant(&value, token_range(string)?, sql, None),
            Token::Usconst => {
                let escape = unicode_escape_character(tokens.get(i + 2..)?, sql)?;
                string_constant(&value, token_range(string)?, sql, Some(escape))
            }
            _ => None,
        }
    })
}

/// The escape character of the `U&'...'` string followed by `tokens`: the character of a
/// `UESCAPE '...'` clause, or `\`. Postgres rejects a statement whose escape character is
/// invalid ([`check_uescapechar`]), so it has no body to find.
///
/// [`check_uescapechar`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parser.c#L352
fn unicode_escape_character(tokens: &[ScanToken], sql: &str) -> Option<u8> {
    let [uescape, string, ..] = tokens else {
        return Some(b'\\');
    };
    if uescape.token() != Token::Uescape {
        return Some(b'\\');
    }
    let range = token_range(string)?;
    match sql
        .get(usize::from(range.start())..usize::from(range.end()))?
        .as_bytes()
    {
        [b'\'', escape, b'\''] => Some(*escape),
        _ => None,
    }
}

/// The body in the string constant at `range` of `sql`, if its value is `value`. `unicode_escape`
/// is the escape character of a `U&'...'` constant.
fn string_constant(
    value: &str,
    range: TextRange,
    sql: &str,
    unicode_escape: Option<u8>,
) -> Option<StringBody> {
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
    let (prefix, escapes) = match (text.as_bytes(), unicode_escape) {
        ([b'u' | b'U', b'&', b'\'', ..], Some(escape)) => (2, Escapes::Unicode(escape)),
        ([b'e' | b'E', b'\'', ..], None) => (1, Escapes::Backslash),
        ([b'\'', ..], None) => (0, Escapes::None),
        // `B'...'` and `X'...'`.
        _ => return None,
    };
    let inner = text.get(prefix + 1..)?.strip_suffix('\'')?;
    let (unescaped, offsets) = unescape(inner, escapes)?;
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

/// The escapes of a string constant besides `''`.
#[derive(Debug, Clone, Copy)]
enum Escapes {
    /// `'...'`.
    None,
    /// `E'...'`.
    Backslash,
    /// `U&'...'`, with its escape character.
    Unicode(u8),
}

/// The value of the inside of a string constant, and where its escapes are. The escapes of
/// `E'...'` strings are read like the `xe` rules of Postgres' lexer ([`scan.l`]), those of
/// `U&'...'` strings like [`str_udeescape`]. A constant continued on the next line, like `'a'`
/// and `'b'` in `'a'\n'b'`, is one value. `None` if an escape is invalid, or if the value isn't
/// valid UTF-8.
///
/// [`scan.l`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L262
/// [`str_udeescape`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parser.c#L372
fn unescape(inner: &str, escapes: Escapes) -> Option<(String, OffsetMap)> {
    let bytes = inner.as_bytes();
    let mut value = Vec::with_capacity(bytes.len());
    let mut offsets = OffsetMap::default();
    let mut cursor = 0;
    while let Some(&byte) = bytes.get(cursor) {
        let (decoded, len): (Vec<u8>, usize) = match (byte, escapes) {
            (b'\'', _) if bytes.get(cursor + 1) == Some(&b'\'') => (vec![b'\''], 2),
            // The constant goes on after a gap, which stands for nothing.
            (b'\'', _) => (Vec::new(), quote_continuation(bytes.get(cursor..)?)?),
            (b'\\', Escapes::Backslash) => escape_sequence(bytes.get(cursor..)?)?,
            (_, Escapes::Unicode(escape)) if byte == escape => {
                unicode_escape(bytes.get(cursor..)?, escape)?
            }
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

/// The bytes that the Unicode escape at the start of `text` stands for, and its length: the
/// escape character twice, `\XXXX` or `\+XXXXXX`, or a UTF-16 surrogate pair of two of those.
fn unicode_escape(text: &[u8], escape: u8) -> Option<(Vec<u8>, usize)> {
    if text.get(1) == Some(&escape) {
        return Some((vec![escape], 2));
    }
    let (first, len) = unicode_code_point(text)?;
    let code_point = match first {
        0xD800..=0xDBFF => {
            // The first half of a surrogate pair, which the next escape completes.
            if text.get(len) != Some(&escape) {
                return None;
            }
            let (second, second_len) = unicode_code_point(text.get(len..)?)?;
            if !(0xDC00..=0xDFFF).contains(&second) {
                return None;
            }
            let code_point = 0x10000 + ((first - 0xD800) << 10) + (second - 0xDC00);
            return Some((char_bytes(code_point)?, len + second_len));
        }
        code_point => code_point,
    };
    Some((char_bytes(code_point)?, len))
}

/// The code point of the `\XXXX` or `\+XXXXXX` escape at the start of `text`, where `\` is the
/// escape character, and its length.
fn unicode_code_point(text: &[u8]) -> Option<(u32, usize)> {
    let (start, digits) = if text.get(1) == Some(&b'+') {
        (2, 6)
    } else {
        (1, 4)
    };
    let hex = text.get(start..start + digits)?;
    if !hex.iter().all(u8::is_ascii_hexdigit) {
        return None;
    }
    let code_point = u32::from_str_radix(std::str::from_utf8(hex).ok()?, 16).ok()?;
    Some((code_point, start + digits))
}

/// The UTF-8 encoding of a code point. `None` for code points that Postgres rejects: zero,
/// beyond U+10FFFF, and the second half of a surrogate pair on its own.
fn char_bytes(code_point: u32) -> Option<Vec<u8>> {
    if code_point == 0 {
        return None;
    }
    Some(char::from_u32(code_point)?.to_string().into_bytes())
}

/// The length of the gap at the start of `text` between two parts of a string constant: a quote,
/// whitespace with a newline, and a quote. Like `quotecontinue` in Postgres' lexer ([`scan.l`]),
/// the whitespace can contain `--` comments.
///
/// [`scan.l`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L226
fn quote_continuation(text: &[u8]) -> Option<usize> {
    let mut cursor = 1;
    let mut newline = false;
    loop {
        match *text.get(cursor)? {
            b'\n' | b'\r' => newline = true,
            b' ' | b'\t' | b'\x0B' | b'\x0C' => {}
            b'-' if text.get(cursor + 1) == Some(&b'-') => {
                // A comment, up to the newline that must follow it.
                cursor += text
                    .get(cursor..)?
                    .iter()
                    .position(|byte| matches!(byte, b'\n' | b'\r'))?;
                continue;
            }
            b'\'' if newline => return Some(cursor + 1),
            _ => return None,
        }
        cursor += 1;
    }
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
    let statements = match body.items.as_slice() {
        [list] => match list.node.as_ref() {
            Some(pgls_query::NodeEnum::List(list)) => &list.items,
            _ => return Vec::new(),
        },
        _ => return Vec::new(),
    };
    if ranges.len() != statements.len() {
        return Vec::new();
    }
    ranges
        .into_iter()
        .zip(statements)
        .filter_map(|(range, statement)| match statement.node.as_ref() {
            // A `RETURN` statement starts with its keyword.
            Some(pgls_query::NodeEnum::ReturnStmt(_)) => returned_expression(range, sql),
            _ => Some(EmbeddedStatement {
                sql: sql
                    .get(usize::from(range.start())..usize::from(range.end()))?
                    .to_owned(),
                range,
                offsets: OffsetMap::default(),
            }),
        })
        .collect()
}

/// The expression of a `RETURN` body. `RETURN` is not a reserved keyword, so a function, a
/// parameter or an option value can be called `return`: the body's keyword is the one that is
/// followed by the expression of `stmt`.
fn return_body(
    stmt: &pgls_query::protobuf::ReturnStmt,
    tokens: &[ScanToken],
    sql: &str,
) -> Option<EmbeddedStatement> {
    let returned = stmt.returnval.as_deref()?;
    let last = tokens
        .iter()
        .rev()
        .find(|token| token.token() != Token::Ascii59)?;
    let end = token_range(last)?.end();
    tokens
        .iter()
        .filter(|token| token.token() == Token::Return)
        .find_map(|keyword| {
            let body = returned_expression(token_range(keyword)?.cover_offset(end), sql)?;
            selects(&body, returned).then_some(body)
        })
}

/// The `RETURN` statement at `range` of `sql`, which starts with its keyword, as a `SELECT` of
/// the same length: `return x + 1` is resolved as `select x + 1`.
fn returned_expression(range: TextRange, sql: &str) -> Option<EmbeddedStatement> {
    let keyword = TextSize::of("return");
    let expression = sql.get(usize::from(range.start() + keyword)..usize::from(range.end()))?;
    // `select` stands for `return`, which has the same length.
    let mut offsets = OffsetMap::default();
    offsets.push_escape(0.into(), TextSize::of("select"), keyword);
    Some(EmbeddedStatement {
        sql: format!("select{expression}"),
        range,
        offsets,
    })
}

/// Whether `body`, a `SELECT` of one expression, selects `expression`, at the locations it has
/// in the text of the statement that contains `body`.
fn selects(body: &EmbeddedStatement, expression: &pgls_query::protobuf::Node) -> bool {
    // Spaces in front of the statement give its nodes the locations they have in the source.
    let padded = format!(
        "{}{}",
        " ".repeat(usize::from(body.range.start())),
        body.sql
    );
    let Ok(parsed) = pgls_query::parse(&padded) else {
        return false;
    };
    let [raw] = parsed.protobuf.stmts.as_slice() else {
        return false;
    };
    let Some(pgls_query::NodeEnum::SelectStmt(select)) =
        raw.stmt.as_ref().and_then(|stmt| stmt.node.as_ref())
    else {
        return false;
    };
    match select.target_list.as_slice() {
        [target] => matches!(
            target.node.as_ref(),
            Some(pgls_query::NodeEnum::ResTarget(target))
                if target.name.is_empty() && target.val.as_deref() == Some(expression)
        ),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use pgls_text_size::{TextRange, TextSize};

    use super::{Escapes, OffsetMap, sql_function_body, unescape};

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
    }

    #[test]
    fn continued_string_bodies_are_one_value() {
        let input = "create function f() returns int language sql as 'select 1'\n'';";
        assert_eq!(bodies(input), [("select 1", "select 1".to_string())]);

        let input =
            "create function f() returns text language sql as 'select 1;'\n  ' select ''a'''";
        let ast = pgls_query::parse(input).unwrap().into_root().unwrap();
        let [first, second] = sql_function_body(&ast, input).try_into().unwrap();
        assert_eq!(first.sql, "select 1;");
        assert_eq!(&input[first.range], "select 1;");
        assert!(first.offsets.is_verbatim());
        assert_eq!(second.sql, "select 'a'");
        assert_eq!(&input[second.range], "select ''a''");
        let a = TextSize::of("select '");
        assert_eq!(
            &input[second.range][usize::from(second.offsets.to_source(a))..],
            "a''"
        );

        // The parts of `E'...'` and `U&'...'` strings have their escapes.
        assert_eq!(
            statements(
                "create function f() returns text language sql as E'select ''\\x41'''\n' || ''b'''"
            ),
            ["select 'A' || 'b'"]
        );
        assert_eq!(
            statements(
                "create function f() returns text language sql as U&'select ''\\00e9'''\n' || ''x'''"
            ),
            ["select '\u{e9}' || 'x'"]
        );

        // A gap inside a statement stands for nothing.
        let input = "create function f() returns int language sql as 'select'\n' 1'";
        let ast = pgls_query::parse(input).unwrap().into_root().unwrap();
        let [body] = sql_function_body(&ast, input).try_into().unwrap();
        assert_eq!(body.sql, "select 1");
        assert_eq!(&input[body.range], "select'\n' 1");
        let source = &input[body.range];
        assert_eq!(
            &source[usize::from(body.offsets.to_source(TextSize::of("select ")))..],
            "1"
        );
        assert_eq!(
            body.offsets.to_statement(TextSize::of("select'\n")),
            TextSize::of("select")
        );
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
        let value = |inner: &str| unescape(inner, Escapes::Backslash).map(|(value, _)| value);
        assert_eq!(value(r"a\tb\x41\101\q\\"), Some("a\tbAAq\\".to_string()));
        assert_eq!(value(r"\U0001F600 é"), Some("\u{1F600} \u{e9}".to_string()));
        assert_eq!(value(r"\xc3\xa9"), Some("\u{e9}".to_string()));
        // Not UTF-8, and a `\u` without four digits.
        assert_eq!(value(r"\xff"), None);
        assert_eq!(value(r"\u12"), None);
        // A constant continued on the next line, with a comment and a vertical tab, and quotes
        // without a newline between them, which don't continue it.
        assert_eq!(value("a' -- c\n\x0B'b\\n"), Some("ab\n".to_string()));
        assert_eq!(value("a' 'b"), None);
        assert_eq!(value("a' -- c'b"), None);
        // Without backslash escapes, backslashes are themselves.
        assert_eq!(
            unescape(r"a\n''", Escapes::None).map(|(value, _)| value),
            Some(r"a\n'".to_string())
        );
    }

    #[test]
    fn unicode_string_bodies_are_mapped() {
        let input = r"create function f(p text) returns text language sql as U&'select ''\00e9'' || \+01F600 || p'";
        let ast = pgls_query::parse(input).unwrap().into_root().unwrap();
        let [body] = sql_function_body(&ast, input).try_into().unwrap();
        assert_eq!(body.sql, "select '\u{e9}' || \u{1F600} || p");
        let source = &input[body.range];
        assert_eq!(source, r"select ''\00e9'' || \+01F600 || p");
        let p = TextSize::of("select '\u{e9}' || \u{1F600} || ");
        assert_eq!(&source[usize::from(body.offsets.to_source(p))..], "p");
        assert_eq!(
            body.offsets
                .to_statement(TextSize::of(source) - TextSize::from(1)),
            p
        );
        assert!(!body.offsets.is_verbatim());

        // With another escape character, and in lower case.
        assert_eq!(
            bodies(
                "create function f() returns text language sql as u&'select ''!00e9\\!!''' uescape '!'"
            ),
            [(r"select ''!00e9\!!''", "select '\u{e9}\\!'".to_string())]
        );
        // Split into statements like other strings.
        assert_eq!(
            statements(
                r"create function f() returns int language sql as U&'select 1; select \0032'"
            ),
            ["select 1;", "select 2"]
        );
    }

    #[test]
    fn unescapes_unicode_like_postgres() {
        let value = |inner: &str, escape: u8| {
            unescape(inner, Escapes::Unicode(escape)).map(|(value, _)| value)
        };
        assert_eq!(
            value(r"a\0041\+000042\\''", b'\\'),
            Some("aAB\\'".to_string())
        );
        // A surrogate pair, in both forms.
        assert_eq!(
            value(r"\D83D\DE00 \+00D83D\+00DE00", b'\\'),
            Some("\u{1F600} \u{1F600}".to_string())
        );
        // Backslashes are themselves with another escape character.
        assert_eq!(value(r"\n#0041##", b'#'), Some(r"\nA#".to_string()));
        // Invalid escapes, a surrogate on its own, zero, and beyond U+10FFFF.
        assert_eq!(value(r"\n", b'\\'), None);
        assert_eq!(value(r"\004", b'\\'), None);
        assert_eq!(value(r"\D83D", b'\\'), None);
        assert_eq!(value(r"\D83Dx", b'\\'), None);
        assert_eq!(value(r"\DE00", b'\\'), None);
        assert_eq!(value(r"\0000", b'\\'), None);
        assert_eq!(value(r"\+110000", b'\\'), None);
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
    fn the_return_keyword_of_the_body_is_found() {
        // `return` names the function, a parameter, an option value and a column.
        assert_eq!(
            bodies("create function return(p int) returns int return p + 1;"),
            [("return p + 1", "select p + 1".to_string())]
        );
        assert_eq!(
            bodies("create function f(return int) returns int return return + 1;"),
            [("return return + 1", "select return + 1".to_string())]
        );
        assert_eq!(
            bodies(
                "create function f(return int) returns int set search_path = return return return"
            ),
            [("return return", "select return".to_string())]
        );
    }

    #[test]
    fn return_statements_of_atomic_bodies_are_selects() {
        let input =
            "create function f(p int) returns int begin atomic select 1; return p + 1; end;";
        let ast = pgls_query::parse(input).unwrap().into_root().unwrap();
        let [first, second] = sql_function_body(&ast, input).try_into().unwrap();
        assert_eq!(first.sql, "select 1;");
        assert!(first.offsets.is_verbatim());
        assert_eq!(&input[second.range], "return p + 1;");
        assert_eq!(second.sql, "select p + 1;");
        assert!(!second.offsets.is_verbatim());
        assert!(pgls_query::parse(&second.sql).is_ok());
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
