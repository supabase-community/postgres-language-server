//! The refactor that replaces a `*` in a select list with the columns it expands to.

use pgls_analyse::{ActionCategory, Applicability, RefactorKind};
use pgls_catalog::resolve::{Reference, StarColumn};
use pgls_query::protobuf::KeywordKind;
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

/// The columns as a select list, e.g. `u.id, u."Name"`.
fn column_list(columns: &[StarColumn]) -> String {
    columns
        .iter()
        .map(|column| {
            column
                .qualifier
                .iter()
                .chain(std::iter::once(&column.name))
                .map(|name| quote_identifier(name))
                .collect::<Vec<_>>()
                .join(".")
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// `name`, quoted unless it reads as the same name without quotes: lower case, and not a keyword
/// other than an unreserved one.
fn quote_identifier(name: &str) -> String {
    let plain = name
        .chars()
        .next()
        .is_some_and(|first| first.is_ascii_lowercase() || first == '_')
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '_' | '$'));
    let keyword = || {
        pgls_query::scan(name).map_or(true, |scan| {
            scan.tokens.first().is_none_or(|token| {
                !matches!(
                    token.keyword_kind(),
                    KeywordKind::NoKeyword | KeywordKind::UnreservedKeyword
                )
            })
        })
    };
    if plain && !keyword() {
        name.to_owned()
    } else {
        format!("\"{}\"", name.replace('"', "\"\""))
    }
}

#[cfg(test)]
mod tests {
    use super::{StarColumn, column_list, quote_identifier};

    #[test]
    fn quotes_names_that_need_it() {
        assert_eq!(quote_identifier("user_id"), "user_id");
        assert_eq!(quote_identifier("_x$1"), "_x$1");
        // Unreserved keywords can be column names.
        assert_eq!(quote_identifier("name"), "name");
        assert_eq!(quote_identifier("Name"), "\"Name\"");
        assert_eq!(quote_identifier("1st"), "\"1st\"");
        assert_eq!(quote_identifier("my col"), "\"my col\"");
        assert_eq!(quote_identifier("a\"b"), "\"a\"\"b\"");
        assert_eq!(quote_identifier("select"), "\"select\"");
        assert_eq!(quote_identifier("between"), "\"between\"");
        assert_eq!(quote_identifier("left"), "\"left\"");
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
        assert_eq!(
            column_list(&columns),
            r#"u.id, public."Posts"."user", title"#
        );
    }
}
