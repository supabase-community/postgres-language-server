//! Migration tools: everything the language server knows about a tool is one [`MigrationTool`].
//!
//! - How to find a project of the tool, and what `init` sets for it ([`project`]).
//! - Which files are migrations, and their sequence numbers, for `migrations.after`.
//! - How a file splits into up and down sections, and the diagnostics for its markers.
//!
//! The workspace runs a file with the tool of `migrations.sectionMarkers`, see
//! [`migration_tool`]. A document gets the tool from the settings of its file, and keeps the
//! sections of its current content.
//!
//! The tools:
//! - [`Plain`]: no tool, every file is one migration.
//! - [`Supabase`]: the Supabase CLI. Every file is one migration.
//! - [`Dbmate`] and [`Goose`]: up and down sections in one file, separated by marker comments.

mod dbmate;
mod files;
mod goose;
mod markers;
pub mod project;
mod supabase;

use std::path::Path;

use pgls_analyser::MigrationSection;
use pgls_configuration::migrations::SectionMarkers;
use pgls_fs::FileSystem;
use pgls_text_size::TextRange;

pub(crate) use files::Migration;
pub(crate) use markers::SectionMarkerDiagnostic;

use dbmate::Dbmate;
use goose::Goose;
use project::{Project, Root};
use supabase::Supabase;

/// What the language server knows about a migration tool. Every method but the name and the
/// setting has a default that fits a tool that runs every file as one migration.
pub(crate) trait MigrationTool: Sync {
    /// The name of the tool, as its users know it.
    fn name(&self) -> &'static str;

    /// The value of `migrations.sectionMarkers` that makes the workspace run files with the tool.
    fn section_markers(&self) -> SectionMarkers;

    /// The folder of the migrations in a project of the tool, relative to the project, if the
    /// tool has one by default.
    fn default_migrations_dir(&self) -> Option<&'static str> {
        None
    }

    /// The project of the tool in `root`, for `init`. Only tools with a reliable sign of their
    /// projects find them.
    fn detect(&self, _fs: &dyn FileSystem, _root: &Root) -> Option<Project> {
        None
    }

    /// Whether `directory` is the folder of the tool inside a project, like `supabase`, so that
    /// `init` configures the project above it.
    fn is_tool_directory(&self, _fs: &dyn FileSystem, _directory: &Path) -> bool {
        false
    }

    /// Whether the file is a migration, given `migrations.migrationsDir`. Migration-only rules
    /// don't run on other files.
    fn is_migration(&self, path: &Path, migrations_dir: &Path) -> bool {
        files::is_in_migrations_dir(path, migrations_dir)
    }

    /// The migration of a file in the migrations folder, with its sequence number, if its name
    /// is one of a migration.
    fn migration(&self, path: &Path, migrations_dir: &Path) -> Option<Migration> {
        files::get_migration(path, migrations_dir)
    }

    /// The sections of a migration file, in file order. A file without markers has none, and is
    /// one migration.
    ///
    /// Markers are found line by line, like the migration tools do, without looking at the SQL.
    fn sections(&self, _text: &str) -> Vec<MigrationSection> {
        Vec::new()
    }

    /// The problems with the markers of a file with the given sections and statements:
    /// statements before the first marker, which the tools reject, and lines that look like
    /// markers but that the tools read as plain comments.
    fn marker_diagnostics(
        &self,
        _text: &str,
        _sections: &[MigrationSection],
        _statements: &[TextRange],
    ) -> Vec<SectionMarkerDiagnostic> {
        Vec::new()
    }
}

/// Migration files without a tool: every file is one migration.
pub(crate) struct Plain;

impl MigrationTool for Plain {
    fn name(&self) -> &'static str {
        "plain SQL"
    }

    fn section_markers(&self) -> SectionMarkers {
        SectionMarkers::None
    }
}

/// The tools, in the order `init` reports their projects.
const TOOLS: &[&dyn MigrationTool] = &[&Supabase, &Dbmate, &Goose, &Plain];

/// The tool that the workspace runs files with for `migrations.sectionMarkers`. Supabase runs
/// every file as one migration, so its projects use [`Plain`].
pub(crate) fn migration_tool(markers: SectionMarkers) -> &'static dyn MigrationTool {
    match markers {
        SectionMarkers::None => &Plain,
        SectionMarkers::Dbmate => &Dbmate,
        SectionMarkers::Goose => &Goose,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pgls_analyser::Direction;

    pub(super) fn sections(text: &str, markers: SectionMarkers) -> Vec<(&str, Direction, bool)> {
        migration_tool(markers)
            .sections(text)
            .into_iter()
            .map(|section| (&text[section.range], section.direction, section.transaction))
            .collect()
    }

    pub(super) fn marker_problems(text: &str, markers: SectionMarkers) -> Vec<(&str, String)> {
        let tool = migration_tool(markers);
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

    #[test]
    fn supabase_files_are_one_migration() {
        let text = "-- migrate:up\nselect 1;\n-- +goose Down\n";
        assert!(Supabase.sections(text).is_empty());
        assert!(
            Supabase
                .marker_diagnostics(text, &[], &[TextRange::default()])
                .is_empty()
        );
    }

    #[test]
    fn selects_the_tool_of_the_section_markers() {
        for markers in [
            SectionMarkers::None,
            SectionMarkers::Dbmate,
            SectionMarkers::Goose,
        ] {
            assert_eq!(migration_tool(markers).section_markers(), markers);
        }
        assert_eq!(migration_tool(SectionMarkers::None).name(), "plain SQL");
        assert_eq!(migration_tool(SectionMarkers::Dbmate).name(), "dbmate");
        assert_eq!(migration_tool(SectionMarkers::Goose).name(), "goose");
        // Supabase has no markers of its own, so its files run like plain ones.
        assert_eq!(
            migration_tool(Supabase.section_markers()).name(),
            "plain SQL"
        );
    }
}
