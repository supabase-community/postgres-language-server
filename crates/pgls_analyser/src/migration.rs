//! Migration sections. Tools like dbmate and goose put the up and the down migration in the same
//! file and run each of them on its own.

use pgls_catalog::TransactionMode;
use pgls_text_size::{TextRange, TextSize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
}

/// A part of a migration file that the migration tool runs on its own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationSection {
    /// From the start of the section to the start of the next one, or to the end of the file.
    pub range: TextRange,
    pub direction: Direction,
    /// Whether the migration tool runs the section in a transaction.
    pub transaction: bool,
    /// The parts of the section that the migration tool sends to Postgres as one query, like a
    /// whole dbmate section or a goose `StatementBegin` block. Postgres runs a query with more
    /// than one statement in a transaction block, also in a section without a transaction. The
    /// other statements of the section are sent one by one. In order, and they don't overlap.
    pub queries: Vec<TextRange>,
}

impl MigrationSection {
    pub fn transaction_mode(&self) -> TransactionMode {
        if self.transaction {
            TransactionMode::Transaction
        } else {
            TransactionMode::Autocommit
        }
    }

    /// The queries that run in an implicit transaction block: outside a transaction, those with
    /// more than one statement. `statement_ranges` are the ranges of the top-level statements of
    /// the file, parsed or not, in order.
    ///
    /// Cost: O(log statements) per query.
    pub fn implicit_blocks<'a>(
        &'a self,
        statement_ranges: &'a [TextRange],
    ) -> impl Iterator<Item = TextRange> + 'a {
        let starting_before = |offset: TextSize| {
            statement_ranges.partition_point(|statement| statement.start() < offset)
        };
        self.queries.iter().copied().filter(move |query| {
            !self.transaction && starting_before(query.end()) - starting_before(query.start()) > 1
        })
    }
}
