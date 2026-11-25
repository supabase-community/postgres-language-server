//! The splitter compared with Postgres' parser, for the corpus check and the fuzz target
//! `splitter_vs_parser`.

use pgls_pretty_print::normalize_ast;
use pgls_query::NodeEnum;
use pgls_text_size::TextSize;

use crate::SplitResult;

impl SplitResult {
    /// Where the statements, each parsed on its own, first differ from the statements the parser
    /// reads from the whole of `sql`, or `None` if they agree or `sql` doesn't parse. A statement
    /// that doesn't parse on its own differs at its start, and statements of `sql` left over after
    /// the last one at the end of `sql`.
    pub fn parser_difference(&self, sql: &str) -> Option<TextSize> {
        // The statements of a text that parses, without locations.
        let statements = |sql: &str| -> Option<Vec<NodeEnum>> {
            let parsed = pgls_query::parse(sql).ok()?;
            Some(
                parsed
                    .protobuf
                    .stmts
                    .into_iter()
                    .filter_map(|raw| raw.stmt?.node)
                    .map(|mut node| {
                        normalize_ast(&mut node);
                        node
                    })
                    .collect(),
            )
        };
        let whole = statements(sql)?;
        let mut whole = whole.iter();
        for &range in &self.ranges {
            let Some(pieces) = sql
                .get(std::ops::Range::<usize>::from(range))
                .and_then(statements)
            else {
                return Some(range.start());
            };
            if !pieces.iter().all(|piece| whole.next() == Some(piece)) {
                return Some(range.start());
            }
        }
        whole.next().map(|_| TextSize::of(sql))
    }
}
