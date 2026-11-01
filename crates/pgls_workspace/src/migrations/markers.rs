//! What the adapters of the migration tools share: reading a file line by line, building its
//! sections from the markers, and the diagnostics for misplaced markers.

use pgls_analyser::{Direction, MigrationSection};
use pgls_diagnostics::{Diagnostic, MessageAndDescription};
use pgls_text_size::{TextRange, TextSize};

/// The lines of `text` without their line endings, with the range of the whole line including
/// its line ending.
pub(super) fn lines(text: &str) -> impl Iterator<Item = (TextRange, &str)> {
    let mut offset = TextSize::from(0);
    text.split_inclusive('\n').map(move |line| {
        let range = TextRange::at(offset, TextSize::of(line));
        offset = range.end();
        (range, line.trim_end_matches(['\n', '\r']))
    })
}

/// The sections of a file, built from its markers in file order.
pub(super) struct SectionsBuilder {
    sections: Vec<MigrationSection>,
    end: TextSize,
}

impl SectionsBuilder {
    pub(super) fn new(text: &str) -> Self {
        Self {
            sections: Vec::new(),
            end: TextSize::of(text),
        }
    }

    /// Starts a section at the marker at `start`. It ends where the next one starts, or at the
    /// end of the file.
    pub(super) fn start(&mut self, start: TextSize, direction: Direction, transaction: bool) {
        if let Some(previous) = self.sections.last_mut() {
            previous.range = TextRange::new(previous.range.start(), start);
        }
        self.sections.push(MigrationSection {
            range: TextRange::new(start, self.end),
            direction,
            transaction,
            queries: Vec::new(),
        });
    }

    /// The section that the line being read is in.
    pub(super) fn current(&mut self) -> Option<&mut MigrationSection> {
        self.sections.last_mut()
    }

    pub(super) fn finish(self) -> Vec<MigrationSection> {
        self.sections
    }
}

/// A problem with the section markers of a migration file that makes the migration tool reject
/// the file, or ignore a marker.
#[derive(Debug, Diagnostic)]
#[diagnostic(category = "migrations/sectionMarkers", severity = Warning)]
pub(crate) struct SectionMarkerDiagnostic {
    #[location(span)]
    pub(super) span: TextRange,
    #[message]
    #[description]
    pub(super) message: MessageAndDescription,
}

impl SectionMarkerDiagnostic {
    fn new(span: TextRange, message: &str) -> Self {
        Self {
            span,
            message: MessageAndDescription::from(message.to_string()),
        }
    }
}

/// The markers of a migration tool, as far as the diagnostics need them.
pub(super) struct MarkerSyntax {
    /// The name of the tool.
    pub(super) tool: &'static str,
    /// The marker that starts the first section, like `-- migrate:up`.
    pub(super) first_marker: &'static str,
    /// Whether the tool reads the line as a marker.
    pub(super) is_marker: fn(&str) -> bool,
    /// What the tool does with a line that looks like a marker but isn't one.
    pub(super) look_alike_message: fn(&str) -> &'static str,
}

/// See [`super::MigrationTool::marker_diagnostics`].
pub(super) fn marker_diagnostics(
    syntax: &MarkerSyntax,
    text: &str,
    sections: &[MigrationSection],
    statements: &[TextRange],
) -> Vec<SectionMarkerDiagnostic> {
    let is_marker = syntax.is_marker;

    let mut diagnostics = Vec::new();
    if let Some(first) = sections.first() {
        let message = format!(
            "{} rejects a migration with statements before the first `{}`.",
            syntax.tool, syntax.first_marker
        );
        diagnostics.extend(
            statements
                .iter()
                .take_while(|statement| statement.start() < first.range.start())
                .map(|statement| SectionMarkerDiagnostic::new(*statement, &message)),
        );
    }

    for (range, line) in lines(text) {
        let looks_like_marker = || {
            let trimmed = line.trim_start();
            is_marker(trimmed) || is_marker(&trimmed.to_ascii_lowercase())
        };
        if !is_marker(line) && looks_like_marker() {
            let span = TextRange::at(range.start(), TextSize::of(line));
            diagnostics.push(SectionMarkerDiagnostic::new(
                span,
                (syntax.look_alike_message)(line),
            ));
        }
    }
    diagnostics.sort_by_key(|diagnostic| diagnostic.span.start());
    diagnostics
}
