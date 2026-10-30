//! Semantic tokens for a single statement: what the names in it refer to, as in
//! `public.users` (a schema and a table), `make_point` (a function), or `'happy'` (a member of
//! an enum).
//!
//! Tokens only come from the names that the catalog resolver resolves to exactly one object, so
//! they follow Postgres' name resolution rather than guessing from the name alone. Names that
//! match no object or several, and constructs the resolver doesn't model, get no token and keep
//! the highlighting of the editor's grammar.

use pgls_catalog::CatalogView;
use pgls_catalog::resolve::{ColumnSource, FunctionContext, Reference, ResolveParams, resolve};
use pgls_query::NodeEnum;
use pgls_query::protobuf::{KeywordKind, ScanToken, Token};
use pgls_text_size::{TextRange, TextSize};

pub struct SemanticTokensParams<'a> {
    pub stmt: &'a NodeEnum,
    /// The statement text. The AST's locations point into it.
    pub sql: &'a str,
    pub catalog: &'a dyn CatalogView,
    /// The explicit search path of the session.
    pub search_path: &'a [String],
    /// Set when the statement is the body of a SQL function.
    pub function: Option<&'a FunctionContext>,
}

/// What a token refers to. The names follow the standard token types of the language server
/// protocol, which editor themes color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticTokenKind {
    /// A schema.
    Namespace,
    /// A table, view, or other relation, or a common table expression.
    Class,
    /// A column.
    Property,
    Function,
    Type,
    /// A parameter of the SQL function whose body the statement is.
    Parameter,
    /// A string literal that is coerced to an enum type.
    EnumMember,
    /// The alias of a FROM item, like `u` in `from users u` and in `u.name`.
    Variable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticToken {
    /// The range in the statement.
    pub range: TextRange,
    pub kind: SemanticTokenKind,
    /// Whether the object is built into Postgres: it is in `pg_catalog` or `information_schema`.
    pub default_library: bool,
}

/// The semantic tokens of a statement, ordered by their start. Tokens don't overlap.
pub fn semantic_tokens(params: SemanticTokensParams<'_>) -> Vec<SemanticToken> {
    let references = resolve(ResolveParams {
        stmt: params.stmt,
        catalog: params.catalog,
        search_path: params.search_path,
        function: params.function,
        sql: Some(params.sql),
        collect_references: true,
    })
    .references;

    let Some(scanned) = scan(params.sql) else {
        return Vec::new();
    };

    let mut tokens = Vec::new();
    for reference in &references {
        if let Reference::EnumLiteral {
            span, type_info, ..
        } = reference
        {
            if let Some(span) = span {
                tokens.push(SemanticToken {
                    range: *span,
                    kind: SemanticTokenKind::EnumMember,
                    default_library: is_builtin_schema(&type_info.schema),
                });
            }
            continue;
        }
        let Some((parts, next)) = name_parts(&scanned, reference.location()) else {
            continue;
        };
        let mut push = |range: Option<&TextRange>, kind, schema: Option<&str>| {
            if let Some(range) = range {
                tokens.push(SemanticToken {
                    range: *range,
                    kind,
                    default_library: schema.is_some_and(is_builtin_schema),
                });
            }
        };
        // The schema qualifier of a name, like `public` in `public.users`.
        let qualifier = |parts: &[TextRange]| parts.len().checked_sub(2).map(|i| parts[i]);
        match reference {
            Reference::Relation { relation, .. } => {
                let schema = Some(relation.schema.as_str());
                push(
                    qualifier(&parts).as_ref(),
                    SemanticTokenKind::Namespace,
                    schema,
                );
                push(parts.last(), SemanticTokenKind::Class, schema);
            }
            Reference::Function { function, .. } => {
                let schema = Some(function.schema.as_str());
                push(
                    qualifier(&parts).as_ref(),
                    SemanticTokenKind::Namespace,
                    schema,
                );
                push(parts.last(), SemanticTokenKind::Function, schema);
            }
            Reference::Type { type_info, .. } => {
                let schema = Some(type_info.schema.as_str());
                push(
                    qualifier(&parts).as_ref(),
                    SemanticTokenKind::Namespace,
                    schema,
                );
                push(parts.last(), SemanticTokenKind::Type, schema);
                // The other words of multi-word types like `double precision`, without
                // modifiers like `(3)` in `timestamp(3) with time zone`.
                let end = reference
                    .span()
                    .map_or(TextSize::default(), |span| span.end());
                for token in scanned[next..]
                    .iter()
                    .take_while(|token| token_range(token).is_some_and(|range| range.end() <= end))
                {
                    if token.keyword_kind() != KeywordKind::NoKeyword {
                        push(token_range(token).as_ref(), SemanticTokenKind::Type, schema);
                    }
                }
            }
            Reference::Column { source, .. } => {
                let relation = match source {
                    ColumnSource::Relation { schema, name } => Some((schema.as_str(), name)),
                    ColumnSource::Cte { .. } | ColumnSource::Subquery { .. } => None,
                };
                let schema = relation.map(|(schema, _)| schema);
                match parts.as_slice() {
                    // `schema.table.column`
                    [schema_part, table, _] => {
                        push(Some(schema_part), SemanticTokenKind::Namespace, schema);
                        push(Some(table), SemanticTokenKind::Class, schema);
                    }
                    // `table.column` or `cte.column`, and otherwise `alias.column`: the qualifier
                    // is the name of the FROM item the column comes from.
                    [qualifier, _] => {
                        let qualifier_name = identifier(&params.sql[*qualifier]);
                        let is_own_name = match source {
                            ColumnSource::Relation { name, .. } | ColumnSource::Cte { name } => {
                                qualifier_name == *name
                            }
                            ColumnSource::Subquery { .. } => false,
                        };
                        if is_own_name {
                            push(Some(qualifier), SemanticTokenKind::Class, schema);
                        } else {
                            push(Some(qualifier), SemanticTokenKind::Variable, None);
                        }
                    }
                    _ => {}
                }
                push(parts.last(), SemanticTokenKind::Property, schema);
            }
            Reference::Parameter { .. } => {
                // `function_name.parameter`
                if parts.len() == 2 {
                    push(parts.first(), SemanticTokenKind::Function, None);
                }
                push(parts.last(), SemanticTokenKind::Parameter, None);
            }
            Reference::Cte { .. } => push(parts.last(), SemanticTokenKind::Class, None),
            Reference::Alias { .. } => push(parts.last(), SemanticTokenKind::Variable, None),
            Reference::EnumLiteral { .. } => {}
            // A `*` is not a name, and its qualifier is a table or alias that has a reference of
            // its own where it is defined.
            Reference::Star { .. } => {}
        }
    }

    tokens.sort_by_key(|token| (token.range.start(), token.range.end()));
    let mut end = TextSize::default();
    tokens.retain(|token| {
        let keep = !token.range.is_empty() && token.range.start() >= end;
        if keep {
            end = token.range.end();
        }
        keep
    });
    tokens
}

fn is_builtin_schema(schema: &str) -> bool {
    matches!(schema, "pg_catalog" | "information_schema")
}

/// The tokens of the statement, without comments.
fn scan(sql: &str) -> Option<Vec<ScanToken>> {
    let scan = pgls_query::scan(sql).ok()?;
    Some(
        scan.tokens
            .into_iter()
            .filter(|token| !matches!(token.token(), Token::SqlComment | Token::CComment))
            .collect(),
    )
}

/// The ranges of the identifiers of the (possibly qualified) name that starts at `location`, like
/// `public . "My Table"`, without the dots, whitespace, and comments between them. Also returns
/// the index of the token after the name.
fn name_parts(tokens: &[ScanToken], location: i32) -> Option<(Vec<TextRange>, usize)> {
    let mut index = tokens.partition_point(|token| token.start < location);
    let first = tokens.get(index).filter(|token| token.start == location)?;
    let mut parts = vec![token_range(first)?];
    index += 1;
    // `$1` is never qualified.
    if first.token() == Token::Param {
        return Some((parts, index));
    }
    if !is_name(first) {
        return None;
    }
    while let [dot, name, ..] = &tokens[index..]
        && dot.token() == Token::Ascii46
        && is_name(name)
    {
        parts.push(token_range(name)?);
        index += 2;
    }
    Some((parts, index))
}

/// Whether the token can be (a part of) a name: an identifier or a keyword.
fn is_name(token: &ScanToken) -> bool {
    matches!(token.token(), Token::Ident | Token::Uident)
        || token.keyword_kind() != KeywordKind::NoKeyword
}

fn token_range(token: &ScanToken) -> Option<TextRange> {
    Some(TextRange::new(
        TextSize::try_from(usize::try_from(token.start).ok()?).ok()?,
        TextSize::try_from(usize::try_from(token.end).ok()?).ok()?,
    ))
}

/// The name an identifier refers to: unquoted identifiers are folded to lower case. Like Postgres,
/// only ASCII letters are folded.
fn identifier(text: &str) -> String {
    match text
        .strip_prefix('"')
        .and_then(|text| text.strip_suffix('"'))
    {
        Some(quoted) => quoted.replace("\"\"", "\""),
        None => text.to_ascii_lowercase(),
    }
}

#[cfg(test)]
mod tests {
    use super::{name_parts, scan};

    /// The parts of the name at the start of `sql`.
    fn parts(sql: &str) -> Vec<&str> {
        name_parts(&scan(sql).unwrap(), 0)
            .map(|(parts, _)| parts.into_iter().map(|range| &sql[range]).collect())
            .unwrap_or_default()
    }

    #[test]
    fn splits_qualified_names() {
        assert_eq!(parts("users"), ["users"]);
        assert_eq!(parts("public.users"), ["public", "users"]);
        assert_eq!(parts("public . users"), ["public", "users"]);
        assert_eq!(
            parts("public /* c */ . -- c\n users x"),
            ["public", "users"]
        );
        assert_eq!(
            parts(r#""My.Schema"."a ""b"" c".id"#),
            [r#""My.Schema""#, r#""a ""b"" c""#, "id"]
        );
        assert_eq!(parts("größe.ölstand + 1"), ["größe", "ölstand"]);
        assert_eq!(parts("current.name"), ["current", "name"]);
        assert_eq!(parts("$1"), ["$1"]);
        assert_eq!(parts("f(x).y"), ["f"]);
        assert!(parts("(u).name").is_empty());
    }
}
