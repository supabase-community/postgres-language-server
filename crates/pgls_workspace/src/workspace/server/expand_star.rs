//! The refactor that replaces a `*` in a select list with the columns it expands to.

use pgls_analyse::{ActionCategory, Applicability, RefactorKind};
use pgls_catalog::resolve::{Reference, StarColumn};
use pgls_catalog::{NamePosition, quote_identifier};
use pgls_query::protobuf::{KeywordKind, ScanToken, Token};
use pgls_text_size::TextSize;

use crate::features::code_actions::{CodeAction, CodeActionKind, EditAction, TextEdit};

/// The action that replaces the `*` or `t.*` of `reference` with its columns. `content` is the
/// text of the file, and the statement of the reference starts at `stmt_start`.
pub(super) fn expand_star_action(
    reference: &Reference,
    content: &str,
    stmt_start: TextSize,
) -> Option<CodeAction> {
    let Reference::Star { span, columns, .. } = reference else {
        return None;
    };
    let range = (*span)? + stmt_start;
    let star = content.get(usize::from(range.start())..usize::from(range.end()))?;
    let new_text = column_list(columns);
    // In the body of a function quoted with `'`, a quote in a column name would end the body.
    if new_text.contains('\'')
        && content
            .get(..usize::from(stmt_start))
            .is_some_and(|before| before.ends_with('\''))
    {
        return None;
    }
    Some(CodeAction {
        title: format!("Expand `{star}` into its columns"),
        kind: CodeActionKind::Edit(EditAction {
            rule: None,
            category: ActionCategory::Refactor(RefactorKind::Rewrite),
            applicability: Applicability::Always,
            edits: vec![TextEdit { range, new_text }],
        }),
        disabled_reason: None,
    })
}

/// Whether `position` is on a `*` that may stand for columns, or on its qualifier, as in
/// `select *` or `select u.*`. The line of the cursor is scanned on its own: `*` in comments
/// and strings, `count(*)`, and multiplications don't count. If the line doesn't scan, like
/// a line inside a multi-line string, any `*` on it counts.
pub(super) fn is_at_star(content: &str, position: TextSize) -> bool {
    let offset = usize::from(position).min(content.len());
    let line_start = content[..offset].rfind('\n').map_or(0, |i| i + 1);
    let line_end = content[offset..]
        .find('\n')
        .map_or(content.len(), |i| offset + i);
    is_at_star_in(&content[line_start..line_end], offset - line_start)
}

/// Whether `cursor` is on a `*` of a select list in `text`, a line or a string constant on it.
fn is_at_star_in(text: &str, cursor: usize) -> bool {
    if !text.contains('*') {
        return false;
    }
    let Ok(scan) = pgls_query::scan(text) else {
        return true;
    };
    // In the body of a function on one line, like `as 'select * from t'`.
    if let Some([_, string]) = scan.tokens.windows(2).find(|pair| {
        pair[0].token() == Token::As
            && pair[1].token() == Token::Sconst
            && usize::try_from(pair[1].start).is_ok_and(|start| start < cursor)
            && usize::try_from(pair[1].end).is_ok_and(|end| cursor < end)
    }) {
        let (Ok(start), Ok(end)) = (usize::try_from(string.start), usize::try_from(string.end))
        else {
            return true;
        };
        let constant = &text[start..end];
        let quote = if constant.starts_with('\'') {
            1
        } else if let Some(after_dollar) = constant.strip_prefix('$') {
            after_dollar.find('$').map_or(1, |tag| tag + 2)
        } else {
            return false;
        };
        let inner = constant
            .get(quote..constant.len().saturating_sub(quote))
            .unwrap_or_default();
        return cursor >= start + quote && is_at_star_in(inner, cursor - start - quote);
    }
    let tokens: Vec<&ScanToken> = scan
        .tokens
        .iter()
        .filter(|token| !matches!(token.token(), Token::SqlComment | Token::CComment))
        .collect();
    tokens.iter().enumerate().any(|(i, star)| {
        if star.token() != Token::Ascii42 {
            return false;
        }
        // What comes before a `*` of a select list: nothing on this line, `SELECT`, `,`, `.`,
        // `RETURNING`, `DISTINCT`, `ALL`, or the `)` of `DISTINCT ON (...)`.
        let mut first = i;
        while first >= 2
            && tokens[first - 1].token() == Token::Ascii46
            && is_name(tokens[first - 2])
        {
            first -= 2;
        }
        let star_like = first < i
            || first == 0
            || matches!(
                tokens[first - 1].token(),
                Token::Select
                    | Token::Ascii44
                    | Token::Returning
                    | Token::Distinct
                    | Token::All
                    | Token::Ascii41
            );
        let start = usize::try_from(tokens[first].start).unwrap_or(usize::MAX);
        let end = usize::try_from(star.end).unwrap_or(0);
        star_like && (start..=end).contains(&cursor)
    })
}

fn is_name(token: &ScanToken) -> bool {
    matches!(token.token(), Token::Ident | Token::Uident)
        || token.keyword_kind() != KeywordKind::NoKeyword
}

/// The columns as a select list, e.g. `u.id, u."Name"`. The first name of each column is a
/// `ColId`, and the names after a dot are labels, which can be any keyword.
fn column_list(columns: &[StarColumn]) -> String {
    columns
        .iter()
        .map(|column| {
            column
                .qualifier
                .iter()
                .chain(std::iter::once(&column.name))
                .enumerate()
                .map(|(i, name)| {
                    let position = if i == 0 {
                        NamePosition::ColId
                    } else {
                        NamePosition::ColLabel
                    };
                    quote_identifier(name, position)
                })
                .collect::<Vec<_>>()
                .join(".")
        })
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use pgls_text_size::TextSize;

    use super::{StarColumn, column_list};

    #[test]
    fn only_stars_of_select_lists_count() {
        let at = |line: &str| {
            let position = line.find('|').unwrap();
            let content = line.replace('|', "");
            super::is_at_star(&content, TextSize::try_from(position).unwrap())
        };
        assert!(at("select |* from users"));
        assert!(at("select *| from users"));
        assert!(at("select id, |* from users"));
        assert!(at("select |u.* from users u"));
        assert!(at("select public.us|ers.* from users"));
        assert!(at("insert into t values (1) returning |*"));
        assert!(at("  |*\n"));
        assert!(!at("select * |from users"));
        assert!(!at("select count(|*) from users"));
        assert!(!at("select a |* b from t"));
        assert!(!at("select 1 /* |comment */"));
        assert!(!at("select '|*' from t"));
        assert!(!at("select 1| -- *"));
        // In the body of a function on the line.
        assert!(at(
            "create function f() returns setof t as 'select |* from t' language sql"
        ));
        assert!(at(
            "create function f() returns setof t as $f$ select |* from t $f$"
        ));
        assert!(!at(
            "create function f() returns setof t as 'select |count(*) from t'"
        ));
    }

    #[test]
    fn qualifies_columns() {
        let columns = [
            StarColumn {
                qualifier: vec!["u".into()],
                name: "id".into(),
            },
            StarColumn {
                qualifier: vec!["public".into(), "Posts".into()],
                name: "user".into(),
            },
            StarColumn {
                qualifier: vec![],
                name: "title".into(),
            },
        ];
        assert_eq!(column_list(&columns), r#"u.id, public."Posts".user, title"#);
    }

    #[test]
    fn quotes_names_where_postgres_needs_it() {
        let column = |qualifier: &[&str], name: &str| StarColumn {
            qualifier: qualifier.iter().map(|part| part.to_string()).collect(),
            name: name.into(),
        };
        // Postgres reads a column name keyword (`between`) as a column, but not a reserved
        // keyword (`select`) or a type or function name keyword (`left`). After a dot, any
        // keyword is a column.
        let columns = [
            column(&[], "between"),
            column(&[], "select"),
            column(&[], "left"),
            column(&["t"], "select"),
            column(&["left"], "id"),
            column(&[], "Name"),
            column(&[], "my col"),
            column(&[], "a\"b"),
            column(&[], "größe"),
            column(&[], "Café"),
        ];
        assert_eq!(
            column_list(&columns),
            r#"between, "select", "left", t.select, "left".id, "Name", "my col", "a""b", größe, "Café""#
        );
    }
}
