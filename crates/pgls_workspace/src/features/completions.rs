use std::{borrow::Cow, sync::Arc};

use pgls_completions::CompletionItem;
use pgls_fs::PgLSPath;
use pgls_query_ext::function_body::{OffsetMap, Quoting};
use pgls_text_size::TextSize;

use crate::workspace::{Document, DocumentStatement, GetCompletionsFilter, WithCSTMapper};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GetCompletionsParams {
    pub project_key: crate::workspace::ProjectKey,
    /// The File for which a completion is requested.
    pub path: PgLSPath,
    /// The Cursor position in the file for which a completion is requested.
    pub position: TextSize,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CompletionsResult {
    pub(crate) items: Vec<CompletionItem>,
}

impl CompletionsResult {
    /// The completions of the statement `stmt`, whose ranges are relative to its text, with the
    /// ranges in the document. In the body of a function in a string constant, the text is
    /// escaped like the body, see [`OffsetMap::escape`]. An item without a text edit, which the
    /// client inserts as its label, is left out if its label would need escaping.
    pub(crate) fn in_document(items: Vec<CompletionItem>, stmt: &DocumentStatement) -> Self {
        let items = items
            .into_iter()
            .filter_map(|mut item| {
                match &mut item.completion_text {
                    Some(text_edit) => {
                        text_edit.range = stmt.document_range(text_edit.range);
                        if let Cow::Owned(escaped) =
                            escape(&stmt.offsets, &text_edit.text, text_edit.is_snippet)
                        {
                            text_edit.text = escaped;
                        }
                    }
                    None if stmt.offsets.escape(&item.label) != item.label => return None,
                    None => {}
                }
                Some(item)
            })
            .collect();

        Self { items }
    }
}

/// `text` as it is written in the source of a statement with the offsets `offsets`. In a
/// snippet, `$`, `{` and `}` are the snippet's syntax, and a backslash is written `\\`.
fn escape<'a>(offsets: &OffsetMap, text: &'a str, is_snippet: bool) -> Cow<'a, str> {
    if !is_snippet {
        return offsets.escape(text);
    }
    if offsets.quoting() == Quoting::None {
        return Cow::Borrowed(text);
    }
    let mut escaped = String::with_capacity(text.len() + 2);
    for c in text.chars() {
        if matches!(c, '$' | '{' | '}') {
            escaped.push(c);
            continue;
        }
        for c in offsets.escape(c.encode_utf8(&mut [0; 4])).chars() {
            if c == '\\' {
                escaped.push('\\');
            }
            escaped.push(c);
        }
    }
    Cow::Owned(escaped)
}

impl IntoIterator for CompletionsResult {
    type Item = CompletionItem;
    type IntoIter = <Vec<CompletionItem> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

pub(crate) fn get_statement_for_completions(
    doc: &Document,
    position: TextSize,
) -> Option<(DocumentStatement, Arc<tree_sitter::Tree>)> {
    let count = doc.count();
    // no arms no cookies
    if count == 0 {
        return None;
    }

    let mut eligible_statements = doc.iter_with_filter(
        WithCSTMapper,
        GetCompletionsFilter {
            cursor_position: position,
        },
    );

    if count == 1 {
        eligible_statements.next()
    } else {
        let mut prev_stmt: Option<(DocumentStatement, Arc<tree_sitter::Tree>)> = None;

        for current_stmt in eligible_statements {
            /*
             * If we have multiple statements, we want to make sure that we do not overlap
             * with the next one.
             *
             * select 1 |select 1;
             *
             * This is however ok if the current statement is a child of the previous one,
             * such as in CREATE FUNCTION bodies.
             */
            if prev_stmt.is_some_and(|(prev, _)| {
                current_stmt.0.range.contains(position) && !current_stmt.0.id.is_child_of(&prev.id)
            }) {
                return None;
            }

            prev_stmt = Some(current_stmt)
        }

        prev_stmt
    }
}

#[cfg(test)]
mod tests {
    use pgls_completions::{CompletionItem, CompletionItemKind, CompletionText};
    use pgls_text_size::{TextRange, TextSize};

    use crate::workspace::Document;

    use super::{CompletionsResult, get_statement_for_completions};

    use pgls_test_utils::QueryWithCursorPosition;

    fn get_doc_and_pos(sql: &str) -> (Document, TextSize) {
        let pos = sql
            .find(QueryWithCursorPosition::cursor_marker())
            .expect("Please add cursor position to test sql");

        let pos: u32 = pos.try_into().unwrap();

        (
            Document::new(sql.replace(QueryWithCursorPosition::cursor_marker(), ""), 5),
            TextSize::new(pos),
        )
    }

    /// The completions of a statement in the body of `function` at the cursor, with their
    /// text, and how they come out in the document.
    fn completed(function: &str, texts: &[(&str, bool)]) -> Vec<(String, String)> {
        let (doc, position) = get_doc_and_pos(function);
        let (stmt, _) =
            get_statement_for_completions(&doc, position).expect("a statement at the cursor");
        assert!(stmt.id.is_child(), "the body of the function");
        let at = stmt.statement_offset(position);
        let items = texts
            .iter()
            .map(|(text, is_snippet)| CompletionItem {
                label: (*text).to_string(),
                description: String::new(),
                preselected: false,
                kind: CompletionItemKind::Column,
                sort_text: String::new(),
                detail: None,
                completion_text: Some(CompletionText {
                    text: (*text).to_string(),
                    range: TextRange::empty(at),
                    is_snippet: *is_snippet,
                }),
            })
            .collect();
        CompletionsResult::in_document(items, &stmt)
            .into_iter()
            .map(|item| {
                let edit = item.completion_text.expect("a text edit");
                assert_eq!(edit.range, TextRange::empty(position));
                (item.label, edit.text)
            })
            .collect()
    }

    #[test]
    fn completions_in_quoted_bodies_are_escaped() {
        let cursor = QueryWithCursorPosition::cursor_marker();
        let texts = [
            ("it's", false),
            (r"a\b", false),
            (r"f(${1:it's}, ${2:a\b})", true),
        ];
        assert_eq!(
            completed(
                &format!(
                    "create function f() returns int language sql as 'select {cursor} from t'"
                ),
                &texts
            ),
            [
                ("it's".into(), "it''s".into()),
                (r"a\b".into(), r"a\b".into()),
                (
                    r"f(${1:it's}, ${2:a\b})".into(),
                    r"f(${1:it''s}, ${2:a\\b})".into()
                ),
            ]
        );
        assert_eq!(
            completed(
                &format!(
                    "create function f() returns int language sql as E'select {cursor} from t'"
                ),
                &texts
            ),
            [
                ("it's".into(), "it''s".into()),
                (r"a\b".into(), r"a\\b".into()),
                (
                    r"f(${1:it's}, ${2:a\b})".into(),
                    r"f(${1:it''s}, ${2:a\\\\b})".into()
                ),
            ]
        );
        // A dollar-quoted body needs no escaping.
        assert_eq!(
            completed(
                &format!(
                    "create function f() returns int language sql as $$ select {cursor} from t $$"
                ),
                &texts
            ),
            texts.map(|(text, _)| (text.to_string(), text.to_string()))
        );
    }

    #[test]
    fn completions_without_an_edit_that_need_escaping_are_left_out() {
        let (doc, position) = get_doc_and_pos(&format!(
            "create function f() returns int language sql as 'select {} from t'",
            QueryWithCursorPosition::cursor_marker()
        ));
        let (stmt, _) = get_statement_for_completions(&doc, position).expect("a statement");
        let item = |label: &str| CompletionItem {
            label: label.to_string(),
            description: String::new(),
            preselected: false,
            kind: CompletionItemKind::Schema,
            sort_text: String::new(),
            detail: None,
            completion_text: None,
        };
        let labels: Vec<String> =
            CompletionsResult::in_document(vec![item("public"), item("it's")], &stmt)
                .into_iter()
                .map(|item| item.label)
                .collect();
        assert_eq!(labels, ["public"]);
    }

    #[test]
    fn finds_matching_statement() {
        let sql = format!(
            r#"
            select * from users;

            update {}users set email = 'myemail@com';

            select 1;
        "#,
            QueryWithCursorPosition::cursor_marker()
        );

        let (doc, position) = get_doc_and_pos(sql.as_str());

        let (stmt, _) = get_statement_for_completions(&doc, position).expect("Expected Statement");

        assert_eq!(stmt.id.content(), "update users set email = 'myemail@com';")
    }

    #[test]
    fn does_not_break_when_no_statements_exist() {
        let sql = QueryWithCursorPosition::cursor_marker().to_string();

        let (doc, position) = get_doc_and_pos(sql.as_str());

        assert!(get_statement_for_completions(&doc, position).is_none());
    }

    #[test]
    fn does_not_return_overlapping_statements_if_too_close() {
        let sql = format!(
            "select * from {}select 1;",
            QueryWithCursorPosition::cursor_marker()
        );

        let (doc, position) = get_doc_and_pos(sql.as_str());

        // make sure these are parsed as two
        assert_eq!(doc.count(), 2);

        assert!(get_statement_for_completions(&doc, position).is_none());
    }

    #[test]
    fn is_fine_with_spaces() {
        let sql = format!(
            "select * from     {}     ;",
            QueryWithCursorPosition::cursor_marker()
        );

        let (doc, position) = get_doc_and_pos(sql.as_str());

        let (stmt, _) = get_statement_for_completions(&doc, position).expect("Expected Statement");

        assert_eq!(stmt.id.content(), "select * from          ;")
    }

    #[test]
    fn considers_offset() {
        let sql = format!("select * from {}", QueryWithCursorPosition::cursor_marker());

        let (doc, position) = get_doc_and_pos(sql.as_str());

        let (stmt, _) = get_statement_for_completions(&doc, position).expect("Expected Statement");

        assert_eq!(stmt.id.content(), "select * from")
    }

    #[test]
    fn identifies_nested_stmts() {
        let sql = format!(
            r#"
            create or replace function one()
            returns integer
            language sql
            as $$
                select {} from cool;
            $$;
        "#,
            QueryWithCursorPosition::cursor_marker()
        );

        let sql = sql.trim();

        let (doc, position) = get_doc_and_pos(sql);

        let (stmt, _) = get_statement_for_completions(&doc, position).expect("Expected Statement");

        assert_eq!(stmt.id.content().trim(), "select  from cool;")
    }

    #[test]
    fn does_not_consider_too_far_offset() {
        let sql = format!(
            "select * from  {}",
            QueryWithCursorPosition::cursor_marker()
        );

        let (doc, position) = get_doc_and_pos(sql.as_str());

        assert!(get_statement_for_completions(&doc, position).is_none());
    }

    #[test]
    fn does_not_consider_offset_if_statement_terminated_by_semi() {
        let sql = format!(
            "select * from users;{}",
            QueryWithCursorPosition::cursor_marker()
        );

        let (doc, position) = get_doc_and_pos(sql.as_str());

        assert!(get_statement_for_completions(&doc, position).is_none());
    }
}
