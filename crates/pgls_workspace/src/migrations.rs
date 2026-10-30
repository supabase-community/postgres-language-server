//! Migration tools: how they name migration files, how they split a file into sections, and how
//! to find a project of theirs.
//!
//! - `files`: which files are migrations, and their sequence numbers, for every tool.
//! - `MigrationTool`: one adapter per tool with section markers, selected with
//!   `migrations.tool`. A document gets the adapter from the settings of its file, and
//!   keeps the sections of its current content and what the tool replaces in it before running
//!   it, like goose's `ENVSUB`.
//! - [`supabase`]: finds the projects of the Supabase CLI for `init`.

mod dbmate;
mod files;
mod goose;
mod markers;
pub mod supabase;

use pgls_analyser::MigrationSection;
use pgls_configuration::migrations as configuration;
use pgls_text_size::TextRange;

pub(crate) use files::{get_migration, is_in_migrations_dir};
pub(crate) use markers::SectionMarkerDiagnostic;

use dbmate::Dbmate;
use goose::Goose;

/// A migration tool that puts the up and the down migration in the same file, separated by
/// marker comments, and runs each of them on its own.
pub(crate) trait MigrationTool: Sync {
    /// The sections of a migration file, in file order. A file without markers has none, and is
    /// one migration.
    ///
    /// Markers are found line by line, like the migration tools do, without looking at the SQL.
    fn sections(&self, text: &str) -> Vec<MigrationSection>;

    /// The problems with the markers of a file with the given sections and statements:
    /// statements before the first marker, which the tools reject, and lines that look like
    /// markers but that the tools read as plain comments.
    fn marker_diagnostics(
        &self,
        text: &str,
        sections: &[MigrationSection],
        statements: &[TextRange],
    ) -> Vec<SectionMarkerDiagnostic>;

    /// What the tool replaces in the file before it runs it, like goose's `ENVSUB`, or `None` if
    /// it runs the file as written.
    fn preprocess(&self, _text: &str) -> Option<Preprocessed> {
        None
    }
}

/// The text that the analysis reads instead of the file as written, with placeholders where the
/// migration tool replaces parts of it. It has the length of the file, so that the offsets of
/// diagnostics, fixes and editor positions are the same in both.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Preprocessed {
    pub(crate) text: String,
    /// The ranges that hold placeholders, in file order.
    pub(crate) placeholders: Vec<TextRange>,
}

impl Preprocessed {
    /// Whether `range` overlaps a placeholder. An empty range touches it if it's inside it.
    pub(crate) fn touches_placeholder(&self, range: TextRange) -> bool {
        self.placeholders.iter().any(|placeholder| {
            if range.is_empty() {
                placeholder.start() < range.start() && range.start() < placeholder.end()
            } else {
                placeholder.start() < range.end() && range.start() < placeholder.end()
            }
        })
    }
}

/// The adapter of the tool of `migrations.tool`, or `None` if every file is one migration.
pub(crate) fn migration_tool(
    tool: configuration::MigrationTool,
) -> Option<&'static dyn MigrationTool> {
    match tool {
        configuration::MigrationTool::None => None,
        configuration::MigrationTool::Dbmate => Some(&Dbmate),
        configuration::MigrationTool::Goose => Some(&Goose),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pgls_analyser::Direction;

    pub(super) fn sections(
        text: &str,
        tool: configuration::MigrationTool,
    ) -> Vec<(&str, Direction, bool)> {
        migration_tool(tool)
            .map_or_else(Vec::new, |tool| tool.sections(text))
            .into_iter()
            .map(|section| (&text[section.range], section.direction, section.transaction))
            .collect()
    }

    pub(super) fn marker_problems(
        text: &str,
        tool: configuration::MigrationTool,
    ) -> Vec<(&str, String)> {
        let Some(tool) = migration_tool(tool) else {
            return Vec::new();
        };
        let sections = tool.sections(text);
        let statements: Vec<TextRange> = pgls_statement_splitter::split(text).ranges;
        tool.marker_diagnostics(text, &sections, &statements)
            .into_iter()
            .map(|diagnostic| (&text[diagnostic.span], diagnostic.message.to_string()))
            .collect()
    }

    #[test]
    fn no_markers() {
        let text = "-- migrate:up\nselect 1;\n-- +goose Down\n";
        assert!(sections(text, configuration::MigrationTool::None).is_empty());
        assert!(marker_problems(text, configuration::MigrationTool::None).is_empty());
        assert!(sections("select 1;", configuration::MigrationTool::Dbmate).is_empty());
    }
}
