use itertools::Itertools;
use pgt_completions::CompletionItem;
use pgt_fs::PgTPath;
use pgt_text_size::{TextRange, TextSize};

use crate::workspace::{Document, Statement, StatementId};

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

pub(crate) fn get_statement_for_completions<'a>(
    doc: &'a Document,
    position: TextSize,
) -> Option<(Statement, &'a TextRange, &'a str)> {
    let count = doc.statement_count();
    // no arms no cookies
    if count == 0 {
        return None;
    }

    /*
     * We allow an offset of two for the statement:
     *
     * select * from | <-- we want to suggest items for the next token.
     *
     * However, if the current statement is terminated by a semicolon, we don't apply any
     * offset.
     *
     * select * from users; | <-- no autocompletions here.
     */
    let matches_expanding_range = |stmt_id: StatementId, range: &TextRange, position: TextSize| {
        let measuring_range = if doc.is_terminated_by_semicolon(stmt_id).unwrap() {
            *range
        } else {
            range.checked_expand_end(2.into()).unwrap_or(*range)
        };
        measuring_range.contains(position)
    };

    if count == 1 {
        let (stmt, range, txt) = doc.iter_statements_with_text_and_range().next().unwrap();
        if matches_expanding_range(stmt.id, range, position) {
            Some((stmt, range, txt))
        } else {
            None
        }
    } else {
        /*
         * If we have multiple statements, we want to make sure that we do not overlap
         * with the next one.
         *
         * select 1 |select 1;
         */
        let mut stmts = doc.iter_statements_with_text_and_range().tuple_windows();
        stmts
            .find(|((current_stmt, rcurrent, _), (_, rnext, _))| {
                let overlaps_next = rnext.contains(position);
                matches_expanding_range(current_stmt.id, rcurrent, position) && !overlaps_next
            })
            .map(|t| t.0)
    }
}
