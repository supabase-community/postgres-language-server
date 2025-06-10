use std::sync::Arc;

use pgt_completions::CompletionItem;
use pgt_fs::PgTPath;
use pgt_text_size::{TextRange, TextSize};

use crate::workspace::{GetCompletionsFilter, GetCompletionsMapper, ParsedDocument, StatementId};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GetCompletionsParams {
    /// The File for which a completion is requested.
    pub path: PgTPath,
    /// The Cursor position in the file for which a completion is requested.
    pub position: TextSize,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CompletionsResult {
    pub(crate) items: Vec<CompletionItem>,
}

impl IntoIterator for CompletionsResult {
    type Item = CompletionItem;
    type IntoIter = <Vec<CompletionItem> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

pub(crate) fn get_statement_for_completions(
    doc: &ParsedDocument,
    position: TextSize,
) -> Option<(StatementId, TextRange, String, Arc<tree_sitter::Tree>)> {
    let count = doc.count();
    // no arms no cookies
    if count == 0 {
        return None;
    }

    let mut eligible_statements = doc.iter_with_filter(
        GetCompletionsMapper,
        GetCompletionsFilter {
            cursor_position: position,
        },
    );

    if count == 1 {
        eligible_statements.next()
    } else {
        let mut prev_stmt: Option<(StatementId, TextRange, String, Arc<tree_sitter::Tree>)> = None;

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
            if prev_stmt.is_some_and(|prev| {
                current_stmt.1.contains(position) && !current_stmt.0.is_child_of(&prev.0)
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
    use pgt_fs::PgTPath;
    use pgt_text_size::TextSize;

    use crate::workspace::ParsedDocument;

    use super::get_statement_for_completions;

    static CURSOR_POSITION: &str = "€";

    fn get_doc_and_pos(sql: &str) -> (ParsedDocument, TextSize) {
        let pos = sql
            .find(CURSOR_POSITION)
            .expect("Please add cursor position to test sql");

        let pos: u32 = pos.try_into().unwrap();

        (
            ParsedDocument::new(
                PgTPath::new("test.sql"),
                sql.replace(CURSOR_POSITION, ""),
                5,
            ),
            TextSize::new(pos),
        )
    }

    #[test]
    fn finds_matching_statement() {
        let sql = format!(
            r#"
            select * from users;

            update {}users set email = 'myemail@com';

            select 1;
        "#,
            CURSOR_POSITION
        );

        let (doc, position) = get_doc_and_pos(sql.as_str());

        let (_, _, text, _) =
            get_statement_for_completions(&doc, position).expect("Expected Statement");

        assert_eq!(text, "update users set email = 'myemail@com';")
    }

    #[test]
    fn does_not_break_when_no_statements_exist() {
        let sql = CURSOR_POSITION.to_string();

        let (doc, position) = get_doc_and_pos(sql.as_str());

        assert!(get_statement_for_completions(&doc, position).is_none());
    }

    #[test]
    fn does_not_return_overlapping_statements_if_too_close() {
        let sql = format!("select * from {}select 1;", CURSOR_POSITION);

        let (doc, position) = get_doc_and_pos(sql.as_str());

        // make sure these are parsed as two
        assert_eq!(doc.count(), 2);

        assert!(get_statement_for_completions(&doc, position).is_none());
    }

    #[test]
    fn is_fine_with_spaces() {
        let sql = format!("select * from     {}     ;", CURSOR_POSITION);

        let (doc, position) = get_doc_and_pos(sql.as_str());

        let (_, _, text, _) =
            get_statement_for_completions(&doc, position).expect("Expected Statement");

        assert_eq!(text, "select * from          ;")
    }

    #[test]
    fn considers_offset() {
        let sql = format!("select * from {}", CURSOR_POSITION);

        let (doc, position) = get_doc_and_pos(sql.as_str());

        let (_, _, text, _) =
            get_statement_for_completions(&doc, position).expect("Expected Statement");

        assert_eq!(text, "select * from")
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
            CURSOR_POSITION
        );

        let sql = sql.trim();

        let (doc, position) = get_doc_and_pos(sql);

        let (_, _, text, _) =
            get_statement_for_completions(&doc, position).expect("Expected Statement");

        assert_eq!(text.trim(), "select  from cool;")
    }

    #[test]
    fn does_not_consider_too_far_offset() {
        let sql = format!("select * from  {}", CURSOR_POSITION);

        let (doc, position) = get_doc_and_pos(sql.as_str());

        assert!(get_statement_for_completions(&doc, position).is_none());
    }

    #[test]
    fn does_not_consider_offset_if_statement_terminated_by_semi() {
        let sql = format!("select * from users;{}", CURSOR_POSITION);

        let (doc, position) = get_doc_and_pos(sql.as_str());

        assert!(get_statement_for_completions(&doc, position).is_none());
    }
}
