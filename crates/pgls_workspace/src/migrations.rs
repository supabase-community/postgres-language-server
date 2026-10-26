//! Migration tools: how they name migration files, how they split a file into sections, and how
//! to find a project of theirs.
//!
//! - `files`: which files are migrations, and their sequence numbers, for every tool.
//! - `MigrationTool`: one adapter per tool with section markers, selected with
//!   `migrations.sectionMarkers`. A document gets the adapter from the settings of its file, and
//!   keeps the sections of its current content.
//! - [`supabase`]: finds the projects of the Supabase CLI for `init`.

mod dbmate;
mod files;
mod goose;
mod markers;
pub mod supabase;

use pgls_analyser::MigrationSection;
use pgls_configuration::migrations::SectionMarkers;
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
}

/// The migration tool of `migrations.sectionMarkers`, or `None` if every file is one migration.
pub(crate) fn migration_tool(markers: SectionMarkers) -> Option<&'static dyn MigrationTool> {
    match markers {
        SectionMarkers::None => None,
        SectionMarkers::Dbmate => Some(&Dbmate),
        SectionMarkers::Goose => Some(&Goose),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pgls_analyser::Direction;

    pub(super) fn sections(text: &str, markers: SectionMarkers) -> Vec<(&str, Direction, bool)> {
        migration_tool(markers)
            .map_or_else(Vec::new, |tool| tool.sections(text))
            .into_iter()
            .map(|section| (&text[section.range], section.direction, section.transaction))
            .collect()
    }

    pub(super) fn marker_problems(text: &str, markers: SectionMarkers) -> Vec<(&str, String)> {
        let Some(tool) = migration_tool(markers) else {
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
        assert!(sections(text, SectionMarkers::None).is_empty());
        assert!(marker_problems(text, SectionMarkers::None).is_empty());
        assert!(sections("select 1;", SectionMarkers::Dbmate).is_empty());
    }
}
