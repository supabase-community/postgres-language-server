//! The marker comments of the migration tools, like `-- migrate:up` or `-- +goose Up`. The tools
//! find them line by line, without looking at the SQL, and so does the language server.

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
