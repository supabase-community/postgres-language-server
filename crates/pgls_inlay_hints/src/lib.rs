//! Inlay hints for a single statement: the names of the parameters that the positional arguments
//! of function calls are passed to, as in `make_point(x: 1, y: 2)`.
//!
//! A call only gets hints when it resolves to exactly one function with the catalog's overload
//! resolution. Calls with unknown argument types, or that match no function or more than one,
//! get none.

use pgls_catalog::resolve::{FunctionContext, Reference, ResolveParams, resolve};
use pgls_catalog::{CatalogView, identifier_name};
use pgls_query::NodeEnum;
use pgls_query::protobuf::{ScanToken, Token};
use pgls_text_size::TextSize;

pub struct InlayHintsParams<'a> {
    pub stmt: &'a NodeEnum,
    /// The statement text. The AST's locations point into it.
    pub sql: &'a str,
    pub catalog: &'a dyn CatalogView,
    /// The explicit search path of the session.
    pub search_path: &'a [String],
    /// Set when the statement is the body of a SQL function.
    pub function: Option<&'a FunctionContext>,
    pub parameter_names: ParameterNameHints,
}

/// Which arguments get the name of their parameter.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ParameterNameHints {
    /// Every positional argument.
    #[default]
    All,
    /// Only constants, like `1`, `-1.5`, `'a'`, `true`, or `null`, whose meaning the call
    /// doesn't show otherwise.
    Literals,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InlayHintKind {
    /// The name of the parameter an argument is passed to.
    Parameter,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlayHint {
    /// The offset in the statement that the hint is shown before.
    pub offset: TextSize,
    pub label: String,
    pub kind: InlayHintKind,
}

/// The inlay hints of a statement, ordered by offset.
pub fn inlay_hints(params: InlayHintsParams<'_>) -> Vec<InlayHint> {
    let references = resolve(ResolveParams {
        stmt: params.stmt,
        catalog: params.catalog,
        search_path: params.search_path,
        function: params.function,
        sql: Some(params.sql),
        collect_references: true,
    })
    .references;
    let calls: Vec<_> = references
        .into_iter()
        .filter_map(|reference| match reference {
            Reference::Function {
                location,
                function,
                arguments,
                ..
            } => Some((location, function, arguments)),
            _ => None,
        })
        // Only positional arguments for named parameters get hints. Skip the other calls, like
        // those of most built-in functions, before scanning the statement.
        .filter(|(_, function, arguments)| {
            function.signature.as_ref().is_some_and(|signature| {
                arguments.iter().any(|argument| {
                    !argument.named
                        && signature
                            .arguments
                            .get(argument.parameter)
                            .and_then(|parameter| parameter.name.as_deref())
                            .is_some_and(|name| !name.is_empty())
                })
            })
        })
        .collect();
    if calls.is_empty() {
        return Vec::new();
    }
    let Ok(scan) = pgls_query::scan(params.sql) else {
        return Vec::new();
    };
    let tokens: Vec<ScanToken> = scan
        .tokens
        .into_iter()
        .filter(|token| !matches!(token.token(), Token::SqlComment | Token::CComment))
        .collect();

    let mut hints = Vec::new();
    for (location, function, call_arguments) in calls {
        let Some(signature) = function.signature.as_ref() else {
            continue;
        };
        let Some(arguments) = argument_tokens(&tokens, location) else {
            continue;
        };
        // An aggregate's `ORDER BY` follows its arguments, and may contain more commas.
        if arguments.len() < call_arguments.len() {
            continue;
        }
        let mut previous = None;
        for (argument, tokens) in call_arguments.iter().zip(arguments) {
            // Expanded variadic arguments are only hinted once, at the first.
            let repeated = previous == Some(argument.parameter);
            previous = Some(argument.parameter);
            if argument.named || repeated {
                continue;
            }
            let Some(name) = signature.arguments[argument.parameter]
                .name
                .as_deref()
                .filter(|name| !name.is_empty())
            else {
                continue;
            };
            if is_reference_to(params.sql, tokens, name)
                || (params.parameter_names == ParameterNameHints::Literals && !is_literal(tokens))
            {
                continue;
            }
            let Ok(offset) = u32::try_from(tokens[0].start) else {
                continue;
            };
            hints.push(InlayHint {
                offset: TextSize::new(offset),
                label: format!("{name}:"),
                kind: InlayHintKind::Parameter,
            });
        }
    }
    hints.sort_by_key(|hint| hint.offset);
    hints
}

/// Whether the argument is a constant: a string, number, boolean, or `NULL`, with an optional
/// sign before a number.
fn is_literal(tokens: &[ScanToken]) -> bool {
    let number = |token: &ScanToken| matches!(token.token(), Token::Iconst | Token::Fconst);
    match tokens {
        [token] => {
            number(token)
                || matches!(
                    token.token(),
                    Token::Sconst
                        | Token::Usconst
                        | Token::Bconst
                        | Token::Xconst
                        | Token::TrueP
                        | Token::FalseP
                        | Token::NullP
                )
        }
        [sign, token] => matches!(sign.token(), Token::Ascii43 | Token::Ascii45) && number(token),
        _ => false,
    }
}

/// The tokens of each argument of the call whose name starts at `location`, without a leading
/// `DISTINCT` or `ALL` of an aggregate.
fn argument_tokens(tokens: &[ScanToken], location: i32) -> Option<Vec<&[ScanToken]>> {
    // The tokens are ordered by their start.
    let name = tokens.partition_point(|token| token.start < location);
    if tokens.get(name)?.start != location {
        return None;
    }
    // The name is at most `schema.name`.
    let open = name
        + tokens[name..]
            .iter()
            .take(4)
            .position(|token| token.token() == Token::Ascii40)?;

    let mut arguments = Vec::new();
    let mut depth = 0usize;
    let mut start = open + 1;
    for (i, token) in tokens.iter().enumerate().skip(open + 1) {
        match token.token() {
            Token::Ascii40 | Token::Ascii91 => depth += 1,
            Token::Ascii41 | Token::Ascii93 if depth == 0 => {
                if i > start {
                    arguments.push(&tokens[start..i]);
                }
                return Some(arguments);
            }
            Token::Ascii41 | Token::Ascii93 => depth -= 1,
            Token::Ascii44 if depth == 0 => {
                arguments.push(tokens.get(start..i).filter(|tokens| !tokens.is_empty())?);
                start = i + 1;
            }
            Token::Distinct | Token::All if i == open + 1 => start = i + 1,
            _ => {}
        }
    }
    None
}

/// Whether the argument is a column or parameter with the name of the parameter, like `x` or
/// `t.x` for the parameter `x`. Hinting it would only repeat the name.
fn is_reference_to(sql: &str, tokens: &[ScanToken], name: &str) -> bool {
    let tokens = match tokens.first().map(ScanToken::token) {
        Some(Token::Variadic) => &tokens[1..],
        _ => tokens,
    };
    let text = |token: &ScanToken| {
        let start = usize::try_from(token.start).ok()?;
        let end = usize::try_from(token.end).ok()?;
        sql.get(start..end)
    };
    let well_formed = tokens.len() % 2 == 1
        && tokens.iter().enumerate().all(|(i, token)| {
            let is_dot = token.token() == Token::Ascii46;
            if i % 2 == 1 {
                is_dot
            } else {
                !is_dot && text(token).is_some_and(is_identifier)
            }
        });
    well_formed
        && tokens
            .last()
            .and_then(text)
            .is_some_and(|last| identifier_name(last) == name)
}

/// Postgres treats every non-ASCII character as a letter in identifiers.
/// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L332-L333
fn is_identifier(text: &str) -> bool {
    text.starts_with('"')
        || text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$' || !c.is_ascii())
}
