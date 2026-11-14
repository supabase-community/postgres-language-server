//! The marker comments of the migration tools, like `-- migrate:up` or `-- +goose Up`, and the
//! units they split a migration file into: the parts that the tool runs on their own, like
//! dbmate's up and down sections. The tools find the markers line by line, without looking at the
//! SQL, and so does the language server.

use pgls_diagnostics::{Diagnostic, MessageAndDescription};
use pgls_text_size::{TextRange, TextSize};

/// The lines of `text` without their line endings, with the range of the whole line including
/// its line ending.
pub(crate) fn lines(text: &str) -> impl Iterator<Item = (TextRange, &str)> {
    let mut offset = TextSize::from(0);
    text.split_inclusive('\n').map(move |line| {
        let range = TextRange::at(offset, TextSize::of(line));
        offset = range.end();
        (range, line.trim_end_matches(['\n', '\r']))
    })
}

/// The lines that look like markers but that the tool doesn't read as one: they only are with
/// the leading whitespace removed or in lowercase. `message` says what the tool does with the
/// line instead.
pub(crate) fn look_alikes(
    text: &str,
    is_marker: impl Fn(&str) -> bool,
    message: impl Fn(&str) -> &'static str,
) -> Vec<(TextRange, &'static str)> {
    lines(text)
        .filter(|(_, line)| {
            let trimmed = line.trim_start();
            !is_marker(line) && (is_marker(trimmed) || is_marker(&trimmed.to_ascii_lowercase()))
        })
        .map(|(range, line)| {
            (
                TextRange::at(range.start(), TextSize::of(line)),
                message(line),
            )
        })
        .collect()
}

/// A problem with the section markers of a migration file that makes the migration tool reject
/// the file, or ignore a marker.
#[derive(Debug, Diagnostic)]
#[diagnostic(category = "migrations/sectionMarkers", severity = Warning)]
pub struct SectionMarkerDiagnostic {
    #[location(span)]
    pub span: TextRange,
    #[message]
    #[description]
    pub message: MessageAndDescription,
}

impl SectionMarkerDiagnostic {
    pub(crate) fn new(span: TextRange, message: &str) -> Self {
        Self {
            span,
            message: MessageAndDescription::from(message.to_string()),
        }
    }
}

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
