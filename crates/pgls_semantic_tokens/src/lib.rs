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
    /// A table, view, or other relation.
    Class,
    /// A column.
    Property,
    Function,
    Type,
    /// A parameter of the SQL function whose body the statement is.
    Parameter,
    /// A string literal that is coerced to an enum type.
    EnumMember,
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

    let mut tokens = Vec::new();
    for reference in &references {
        let Some(span) = reference.span() else {
            continue;
        };
        let Some(text) = params.sql.get(std::ops::Range::<usize>::from(span)) else {
            continue;
        };
        let parts: Vec<TextRange> = name_parts(text)
            .into_iter()
            .map(|part| part + span.start())
            .collect();
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
                    // `table.column`, but not `alias.column`.
                    [table, _]
                        if relation
                            .is_some_and(|(_, name)| identifier(&params.sql[*table]) == *name) =>
                    {
                        push(Some(table), SemanticTokenKind::Class, schema);
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
            Reference::EnumLiteral { type_info, .. } => {
                push(
                    Some(&span),
                    SemanticTokenKind::EnumMember,
                    Some(type_info.schema.as_str()),
                );
            }
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

/// The ranges of the identifiers of a (possibly qualified) name like `public . "My Table"`,
/// without the dots and whitespace between them.
fn name_parts(text: &str) -> Vec<TextRange> {
    let mut parts = Vec::new();
    let mut chars = text.char_indices().peekable();
    while let Some(&(start, c)) = chars.peek() {
        if c.is_whitespace() || c == '.' {
            chars.next();
            continue;
        }
        let mut end = start;
        if c == '"' {
            chars.next();
            // A doubled quote is a quote inside the identifier.
            while let Some((i, c)) = chars.next() {
                if c == '"' && chars.next_if(|&(_, c)| c == '"').is_none() {
                    end = i + 1;
                    break;
                }
            }
        } else {
            while let Some((i, c)) = chars.next_if(|&(_, c)| !c.is_whitespace() && c != '.') {
                end = i + c.len_utf8();
            }
        }
        if let (Ok(start), Ok(end)) = (TextSize::try_from(start), TextSize::try_from(end))
            && start < end
        {
            parts.push(TextRange::new(start, end));
        }
    }
    parts
}

/// The name an identifier refers to: unquoted identifiers are folded to lower case.
fn identifier(text: &str) -> String {
    match text
        .strip_prefix('"')
        .and_then(|text| text.strip_suffix('"'))
    {
        Some(quoted) => quoted.replace("\"\"", "\""),
        None => text.to_lowercase(),
    }
}

#[cfg(test)]
mod tests {
    use super::name_parts;

    fn parts(text: &str) -> Vec<&str> {
        name_parts(text)
            .into_iter()
            .map(|range| &text[range])
            .collect()
    }

    #[test]
    fn splits_qualified_names() {
        assert_eq!(parts("users"), ["users"]);
        assert_eq!(parts("public.users"), ["public", "users"]);
        assert_eq!(parts("public . users"), ["public", "users"]);
        assert_eq!(
            parts(r#""My.Schema"."a ""b"" c".id"#),
            [r#""My.Schema""#, r#""a ""b"" c""#, "id"]
        );
        assert_eq!(parts("$1"), ["$1"]);
    }
}
