use pgls_analyser::StatementOffsets;
use pgls_query::protobuf::{CreateFunctionStmt, FunctionParameterMode, ScanToken, Token};
use pgls_text_size::{TextRange, TextSize};

#[derive(Debug, Clone)]
pub struct ArgType {
    pub schema: Option<String>,
    pub name: String,
    pub is_array: bool,
}

#[derive(Debug, Clone)]
pub struct SQLFunctionArg {
    pub name: Option<String>,
    pub type_: ArgType,
}

#[derive(Debug, Clone)]
pub struct SQLFunctionSignature {
    #[allow(dead_code)]
    pub schema: Option<String>,
    pub name: String,
    pub args: Vec<SQLFunctionArg>,
}

/// A statement in the body of a SQL function.
#[derive(Debug, Clone)]
pub struct SQLFunctionBody {
    /// The range of the statement in the `CREATE FUNCTION` statement.
    pub range: TextRange,
    pub body: String,
    /// Where the characters of `body` are in the range, if it is not a slice of the `CREATE
    /// FUNCTION` statement.
    pub offsets: StatementOffsets,
}

/// Whether the function is written in SQL: `LANGUAGE sql`, or a `BEGIN ATOMIC` or `RETURN` body,
/// whose language defaults to SQL.
fn is_sql_function(create_fn: &CreateFunctionStmt) -> bool {
    match pgls_query_ext::utils::find_option_value(create_fn, "language") {
        Some(language) => language == "sql",
        None => create_fn.sql_body.is_some(),
    }
}

/// Extracts the function signature from a SQL function definition
pub fn get_sql_fn_signature(ast: &pgls_query::NodeEnum) -> Option<SQLFunctionSignature> {
    let create_fn = match ast {
        pgls_query::NodeEnum::CreateFunctionStmt(cf) => cf,
        _ => return None,
    };

    // Only process SQL functions
    if !is_sql_function(create_fn) {
        return None;
    }

    let fn_name = pgls_query_ext::utils::parse_name(&create_fn.funcname)?;

    // we return None if anything is not expected
    let mut fn_args = Vec::new();
    for arg in &create_fn.parameters {
        if let Some(pgls_query::NodeEnum::FunctionParameter(node)) = &arg.node {
            // The body only sees the input parameters, by name and as `$n`
            // ([`get_func_input_arg_names`] in [`prepare_sql_fn_parse_info`]). `OUT` and
            // `RETURNS TABLE` columns only describe the result: their names are not in scope,
            // and they don't count for `$n`, so `$1` in `f(OUT x int, a text)` is `a`. `INOUT`
            // and `VARIADIC` parameters are inputs and stay. The position in `fn_args` is the
            // `n` of `$n`.
            //
            // [`get_func_input_arg_names`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/utils/fmgr/funcapi.c#L1522
            // [`prepare_sql_fn_parse_info`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/executor/functions.c#L251
            if matches!(
                node.mode(),
                FunctionParameterMode::FuncParamOut | FunctionParameterMode::FuncParamTable
            ) {
                continue;
            }

            let arg_name = (!node.name.is_empty()).then_some(node.name.clone());

            let arg_type = node.arg_type.as_ref()?;
            let type_name = pgls_query_ext::utils::parse_name(&arg_type.names)?;
            fn_args.push(SQLFunctionArg {
                name: arg_name,
                type_: ArgType {
                    schema: type_name.0,
                    name: type_name.1,
                    is_array: node
                        .arg_type
                        .as_ref()
                        .map(|t| !t.array_bounds.is_empty())
                        .unwrap_or(false),
                },
            });
        } else {
            return None;
        }
    }

    Some(SQLFunctionSignature {
        schema: fn_name.0,
        name: fn_name.1,
        args: fn_args,
    })
}

/// The statements of the body of a SQL function: the string after `AS`, the statements of a
/// `BEGIN ATOMIC` body, or the expression of a `RETURN` body as a `SELECT`.
pub fn get_sql_fn_bodies(ast: &pgls_query::NodeEnum, content: &str) -> Vec<SQLFunctionBody> {
    let pgls_query::NodeEnum::CreateFunctionStmt(create_fn) = ast else {
        return Vec::new();
    };
    if !is_sql_function(create_fn) {
        return Vec::new();
    }
    let Ok(scan) = pgls_query::scan(content) else {
        return Vec::new();
    };
    let tokens: Vec<ScanToken> = scan
        .tokens
        .into_iter()
        .filter(|token| !matches!(token.token(), Token::SqlComment | Token::CComment))
        .collect();

    match create_fn
        .sql_body
        .as_deref()
        .and_then(|body| body.node.as_ref())
    {
        Some(pgls_query::NodeEnum::List(body)) => atomic_body(body, &tokens, content),
        Some(pgls_query::NodeEnum::ReturnStmt(_)) => {
            return_body(&tokens, content).into_iter().collect()
        }
        Some(_) => Vec::new(),
        None => pgls_query_ext::utils::find_option_value(create_fn, "as")
            .and_then(|body| string_body_in(&body, &tokens, content))
            .into_iter()
            .collect(),
    }
}

fn token_range(token: &ScanToken) -> Option<TextRange> {
    Some(TextRange::new(
        TextSize::try_from(usize::try_from(token.start).ok()?).ok()?,
        TextSize::try_from(usize::try_from(token.end).ok()?).ok()?,
    ))
}

/// The body of a function defined with `AS '...'`, `AS E'...'` or `AS $$...$$`: the string
/// constant after `AS` whose value is `body`, with the offsets of the escapes in it. Unicode
/// escape strings (`U&'...'`) are not supported.
pub fn string_body(body: &str, content: &str) -> Option<SQLFunctionBody> {
    let scan = pgls_query::scan(content).ok()?;
    let tokens: Vec<ScanToken> = scan
        .tokens
        .into_iter()
        .filter(|token| !matches!(token.token(), Token::SqlComment | Token::CComment))
        .collect();
    string_body_in(body, &tokens, content)
}

/// [`string_body`] with the tokens of `content`, without comments.
fn string_body_in(body: &str, tokens: &[ScanToken], content: &str) -> Option<SQLFunctionBody> {
    tokens.windows(2).find_map(|pair| {
        let [as_token, string] = pair else {
            return None;
        };
        if as_token.token() != Token::As || string.token() != Token::Sconst {
            return None;
        }
        string_constant_body(body, token_range(string)?, content)
    })
}

/// The body in the string constant at `range`, if its value is `body`.
fn string_constant_body(body: &str, range: TextRange, content: &str) -> Option<SQLFunctionBody> {
    let text = &content[range];
    if let Some(after_dollar) = text.strip_prefix('$') {
        let tag_end = after_dollar.find('$')? + 2;
        let inner = text.get(tag_end..text.len().checked_sub(tag_end)?)?;
        if inner != body {
            return None;
        }
        return Some(SQLFunctionBody {
            range: TextRange::at(
                range.start() + TextSize::try_from(tag_end).ok()?,
                TextSize::of(inner),
            ),
            body: body.to_owned(),
            offsets: StatementOffsets::default(),
        });
    }
    let (prefix, escapes) = match text.as_bytes() {
        [b'e' | b'E', b'\'', ..] => (2, true),
        [b'\'', ..] => (1, false),
        _ => return None,
    };
    let inner = text.get(prefix..)?.strip_suffix('\'')?;
    let (unescaped, offsets) = unescape(inner, escapes)?;
    if unescaped != body {
        return None;
    }
    Some(SQLFunctionBody {
        range: TextRange::at(
            range.start() + TextSize::try_from(prefix).ok()?,
            TextSize::of(inner),
        ),
        body: body.to_owned(),
        offsets,
    })
}

/// The value of the inside of a string constant, and where its escapes are. With `escapes`, the
/// backslash escapes of `E'...'` strings are read, like the `xe` rules of Postgres' lexer:
/// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L262
/// `None` if a quote ends the constant before its end (a constant continued on a new line), or
/// if the value isn't valid UTF-8.
fn unescape(inner: &str, escapes: bool) -> Option<(String, StatementOffsets)> {
    let bytes = inner.as_bytes();
    let mut value = Vec::with_capacity(bytes.len());
    let mut offsets = StatementOffsets::default();
    let mut cursor = 0;
    while let Some(&byte) = bytes.get(cursor) {
        let (decoded, len): (Vec<u8>, usize) = match byte {
            b'\'' => {
                if bytes.get(cursor + 1) != Some(&b'\'') {
                    return None;
                }
                (vec![b'\''], 2)
            }
            b'\\' if escapes => escape_sequence(&bytes[cursor..])?,
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
            Some((vec![number(2, len, 16)? as u8], 2 + len))
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
/// `END` that starts a statement; the other `END`s close `CASE` expressions.
fn atomic_body(
    body: &pgls_query::protobuf::List,
    tokens: &[ScanToken],
    content: &str,
) -> Vec<SQLFunctionBody> {
    let Some(start) = tokens
        .windows(2)
        .position(|pair| pair[0].token() == Token::BeginP && pair[1].token() == Token::Atomic)
    else {
        return Vec::new();
    };
    let mut statements = Vec::new();
    let mut statement_start: Option<&ScanToken> = None;
    let (mut parens, mut cases) = (0usize, 0usize);
    for token in &tokens[start + 2..] {
        let Some(first) = statement_start else {
            match token.token() {
                Token::EndP => break,
                // An empty statement.
                Token::Ascii59 => continue,
                _ => {}
            }
            statement_start = Some(token);
            if token.token() == Token::Case {
                cases += 1;
            }
            continue;
        };
        match token.token() {
            Token::Ascii40 => parens += 1,
            Token::Ascii41 => parens = parens.saturating_sub(1),
            Token::Case => cases += 1,
            Token::EndP => cases = cases.saturating_sub(1),
            Token::Ascii59 if parens == 0 && cases == 0 => {
                if let (Some(first), Some(last)) = (token_range(first), token_range(token)) {
                    statements.push(first.cover(last));
                }
                statement_start = None;
            }
            _ => {}
        }
    }
    // The body has one list of statements.
    let expected = match body.items.as_slice() {
        [list] => match list.node.as_ref() {
            Some(pgls_query::NodeEnum::List(list)) => list.items.len(),
            _ => return Vec::new(),
        },
        _ => return Vec::new(),
    };
    if statements.len() != expected {
        return Vec::new();
    }
    statements
        .into_iter()
        .map(|range| SQLFunctionBody {
            range,
            body: content[range].to_owned(),
            offsets: StatementOffsets::default(),
        })
        .collect()
}

/// The expression of a `RETURN` body, as a `SELECT` of the same length: `return x + 1` is
/// resolved as `select x + 1`.
fn return_body(tokens: &[ScanToken], content: &str) -> Option<SQLFunctionBody> {
    let return_token = tokens.iter().find(|token| token.token() == Token::Return)?;
    let last = tokens
        .iter()
        .rev()
        .find(|token| token.token() != Token::Ascii59)?;
    let range = token_range(return_token)?.cover(token_range(last)?);
    let expression =
        content.get(usize::try_from(return_token.end).ok()?..usize::from(range.end()))?;
    Some(SQLFunctionBody {
        range,
        body: format!("select{expression}"),
        offsets: StatementOffsets::default(),
    })
}

/// Puts the formatted body into `CREATE FUNCTION ... AS '<body>'` for a `LANGUAGE sql` function.
///
/// A reformatted body goes on lines of its own between the quotes. A body that formatting leaves
/// alone keeps the layout it was written with. Returns true when the body was replaced.
pub fn set_formatted_sql_fn_body(ast: &mut pgls_query::NodeEnum, formatted: &str) -> bool {
    let Some(body) = sql_fn_body_mut(ast) else {
        return false;
    };

    if body.trim() == formatted.trim() {
        return false;
    }

    *body = format!("\n{formatted}\n");
    true
}

fn sql_fn_body_mut(ast: &mut pgls_query::NodeEnum) -> Option<&mut String> {
    let pgls_query::NodeEnum::CreateFunctionStmt(create_fn) = ast else {
        return None;
    };

    let language = pgls_query_ext::utils::find_option_value(create_fn, "language");
    if language.as_deref() != Some("sql") {
        return None;
    }

    let def_elem = create_fn
        .options
        .iter_mut()
        .find_map(|option| match option.node.as_mut() {
            Some(pgls_query::NodeEnum::DefElem(def_elem)) if def_elem.defname == "as" => {
                Some(def_elem)
            }
            _ => None,
        })?;

    match def_elem.arg.as_mut()?.node.as_mut()? {
        pgls_query::NodeEnum::String(s) => Some(&mut s.sval),
        pgls_query::NodeEnum::List(list) if list.items.len() == 1 => {
            match list.items[0].node.as_mut()? {
                pgls_query::NodeEnum::String(s) => Some(&mut s.sval),
                _ => None,
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sql_function_signature() {
        let input = "CREATE FUNCTION add(test0 integer, test1 integer) RETURNS integer
    AS 'select $1 + $2;'
    LANGUAGE SQL
    IMMUTABLE
    RETURNS NULL ON NULL INPUT;";

        let ast = pgls_query::parse(input).unwrap().into_root().unwrap();

        let sig = get_sql_fn_signature(&ast);

        assert!(sig.is_some());

        let sig = sig.unwrap();

        let arg1 = sig.args.first().unwrap();

        assert_eq!(arg1.name, Some("test0".to_string()));
        assert_eq!(arg1.type_.name, "int4");

        let arg2 = sig.args.get(1).unwrap();
        assert_eq!(arg2.name, Some("test1".to_string()));
        assert_eq!(arg2.type_.name, "int4");
    }

    #[test]
    fn output_parameters_are_not_arguments() {
        let names = |input: &str| {
            let ast = pgls_query::parse(input).unwrap().into_root().unwrap();
            get_sql_fn_signature(&ast)
                .unwrap()
                .args
                .into_iter()
                .map(|arg| arg.name)
                .collect::<Vec<_>>()
        };

        assert_eq!(
            names(
                "create function f(p_above bigint) returns table(id bigint, received_at timestamptz) language sql as 'select 1, now()'"
            ),
            [Some("p_above".to_string())]
        );
        assert_eq!(
            names(
                "create function f(out x int, a text, inout b int, variadic c int[]) language sql as 'select 1, 2'"
            ),
            [
                Some("a".to_string()),
                Some("b".to_string()),
                Some("c".to_string())
            ]
        );
    }

    /// The statements of the body, as their text in `input` and the text that is resolved.
    fn bodies(input: &str) -> Vec<(&str, String)> {
        let ast = pgls_query::parse(input).unwrap().into_root().unwrap();
        get_sql_fn_bodies(&ast, input)
            .into_iter()
            .map(|body| (&input[body.range], body.body))
            .collect()
    }

    #[test]
    fn quoted_bodies_are_found_by_their_token() {
        // The default is the same text as the body, and comes first.
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
    fn doubled_quotes_are_mapped() {
        let input = "create function f(p text) returns text language sql as 'select ''a'' || p'";
        let ast = pgls_query::parse(input).unwrap().into_root().unwrap();
        let [body] = get_sql_fn_bodies(&ast, input).try_into().unwrap();
        assert_eq!(body.body, "select 'a' || p");
        assert_eq!(&input[body.range], "select ''a'' || p");
        let p = TextSize::of("select 'a' || ");
        assert_eq!(
            &input[body.range][usize::from(body.offsets.to_document(p))..],
            "p"
        );
    }

    #[test]
    fn escape_string_bodies_are_mapped() {
        let input = r"create function f(p text) returns text language sql as E'select ''\u00e9'' ||\n E\'\\t\' || p'";
        let ast = pgls_query::parse(input).unwrap().into_root().unwrap();
        let [body] = get_sql_fn_bodies(&ast, input).try_into().unwrap();
        assert_eq!(body.body, "select '\u{e9}' ||\n E'\\t' || p");
        let p = TextSize::of("select '\u{e9}' ||\n E'\\t' || ");
        let source = &input[body.range];
        assert_eq!(&source[usize::from(body.offsets.to_document(p))..], "p");
        assert_eq!(
            body.offsets
                .to_statement(TextSize::of(source) - TextSize::from(1)),
            p
        );
    }

    #[test]
    fn unescapes_like_postgres() {
        let value = |inner: &str| super::unescape(inner, true).map(|(value, _)| value);
        assert_eq!(value(r"a\tb\x41\101\q\\"), Some("a\tbAAq\\".to_string()));
        assert_eq!(value(r"\U0001F600 é"), Some("\u{1F600} \u{e9}".to_string()));
        assert_eq!(value(r"\xc3\xa9"), Some("\u{e9}".to_string()));
        // Not UTF-8, a `\u` without four digits, and a quote that ends the constant.
        assert_eq!(value(r"\xff"), None);
        assert_eq!(value(r"\u12"), None);
        assert_eq!(value("a'\n'b"), None);
        // Without escapes, backslashes are themselves.
        assert_eq!(
            super::unescape(r"a\n''", false).map(|(value, _)| value),
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
        let input = "create function f(p int) returns int return p + 1;";
        assert_eq!(
            bodies(input),
            [("return p + 1", "select p + 1".to_string())]
        );
        let ast = pgls_query::parse(input).unwrap().into_root().unwrap();
        assert!(get_sql_fn_signature(&ast).is_some());
    }

    #[test]
    fn array_type() {
        let input = "CREATE FUNCTION add(test0 integer[], test1 integer) RETURNS integer
    AS 'select $1 + $2;'
    LANGUAGE SQL
    IMMUTABLE
    RETURNS NULL ON NULL INPUT;";

        let ast = pgls_query::parse(input).unwrap().into_root().unwrap();

        let sig = get_sql_fn_signature(&ast);

        assert!(sig.is_some());

        let sig = sig.unwrap();

        assert!(
            sig.args
                .iter()
                .find(|arg| arg.type_.is_array)
                .map(|arg| {
                    assert_eq!(arg.type_.name, "int4");
                    assert!(arg.type_.is_array);
                })
                .is_some()
        );
    }

    #[test]
    fn set_sql_function_body() {
        let input = "CREATE FUNCTION add(a integer, b integer) RETURNS integer
    AS 'SELECT $1+$2;'
    LANGUAGE SQL
    IMMUTABLE;";

        let mut ast = pgls_query::parse(input).unwrap().into_root().unwrap();

        assert!(set_formatted_sql_fn_body(&mut ast, "select $1 + $2;"));

        let create_fn = match &ast {
            pgls_query::NodeEnum::CreateFunctionStmt(stmt) => stmt,
            _ => panic!("Expected CreateFunctionStmt"),
        };

        let body = pgls_query_ext::utils::find_option_value(create_fn, "as");
        assert_eq!(body, Some("\nselect $1 + $2;\n".to_string()));
    }

    #[test]
    fn an_unchanged_sql_function_body_keeps_its_layout() {
        let input = "CREATE FUNCTION one() RETURNS integer AS $$\n  select 1;\n$$ LANGUAGE SQL;";

        let mut ast = pgls_query::parse(input).unwrap().into_root().unwrap();

        assert!(!set_formatted_sql_fn_body(&mut ast, "select 1;"));
    }
}
