//! The units of a migration file: the parts that the tool runs on their own, like dbmate's up and
//! down sections.

use pgls_text_size::{TextRange, TextSize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
}

/// A part of a migration file that the migration tool runs on its own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unit {
    /// From the start of the unit to the start of the next one, or to the end of the file.
    pub range: TextRange,
    pub direction: Direction,
    /// Whether the migration tool runs the unit in a transaction.
    pub transaction: bool,
    /// The parts of the unit that the migration tool sends to Postgres as one query, like a
    /// whole dbmate section or a goose `StatementBegin` block. Postgres runs a query with more
    /// than one statement in a transaction block, also in a unit without a transaction. The
    /// other statements of the unit are sent one by one. In order, and they don't overlap.
    pub queries: Vec<TextRange>,
}

impl Unit {
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

/// The units of a file, built from its markers in file order.
pub(crate) struct UnitsBuilder {
    units: Vec<Unit>,
    end: TextSize,
}

impl UnitsBuilder {
    pub(crate) fn new(text: &str) -> Self {
        Self {
            units: Vec::new(),
            end: TextSize::of(text),
        }
    }

    /// Starts a unit at the marker at `start`. It ends where the next one starts, or at the end
    /// of the file.
    pub(crate) fn start(&mut self, start: TextSize, direction: Direction, transaction: bool) {
        if let Some(previous) = self.units.last_mut() {
            previous.range = TextRange::new(previous.range.start(), start);
        }
        self.units.push(Unit {
            range: TextRange::new(start, self.end),
            direction,
            transaction,
            queries: Vec::new(),
        });
    }

    /// The unit that the line being read is in.
    pub(crate) fn current(&mut self) -> Option<&mut Unit> {
        self.units.last_mut()
    }

    pub(crate) fn finish(self) -> Vec<Unit> {
        self.units
    }
}
