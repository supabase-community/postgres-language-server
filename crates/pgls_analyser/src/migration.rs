//! Migration sections. Tools like dbmate and goose put the up and the down migration in the same
//! file and run each of them on its own.

use pgls_catalog::TransactionMode;
use pgls_text_size::TextRange;

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
    /// other statements of the section are sent one by one.
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
}
