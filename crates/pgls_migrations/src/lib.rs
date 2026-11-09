//! Migration tools: everything the language server knows about a tool is one [`MigrationTool`].
//!
//! Every tool answers every question of the trait, so its file says all it does. What tools
//! share are helpers that they call: [`naming`] for the file names, [`directives`] for the
//! marker comments, [`units`] for the parts of a file that run on their own, and [`detect`] for
//! finding projects.
//!
//! The workspace asks the tool of a file's settings two things, and keeps the answers:
//! - [`MigrationTool::file`]: what the path says, like the version of the migration.
//! - [`MigrationTool::read`]: what the text says, like its up and down sections.
//!
//! `init` asks every tool for its projects with [`detect::find_projects`].
//!
//! The tools:
//! - [`Plain`]: no tool, every file is one migration.
//! - [`Supabase`]: the Supabase CLI. Every file is one migration.
//! - [`Dbmate`] and [`Goose`]: up and down sections in one file, separated by marker comments.

pub mod detect;
mod directives;
mod naming;
mod tools;
mod units;

use std::path::Path;

use pgls_fs::FileSystem;
use pgls_text_size::TextRange;

pub use detect::{Detected, LocalDatabase, Root};
pub use directives::SectionMarkerDiagnostic;
pub use tools::{Dbmate, Goose, Plain, Supabase};
pub use units::{Direction, Unit};

/// A migration tool. Implementations have no defaults: each one says what the tool does, and
/// calls the shared helpers for what it does like other tools.
pub trait MigrationTool: Sync {
    /// The tool, as the settings name it.
    fn id(&self) -> ToolId;

    /// The name of the tool, as its users know it.
    fn name(&self) -> &'static str;

    /// The project of the tool in `root`, for `init`, if the tool has a reliable sign of its
    /// projects.
    fn detect(&self, fs: &dyn FileSystem, root: &Root) -> Option<Detected>;

    /// What the tool keeps in the database it migrates.
    fn database(&self) -> Database;

    /// The folder of the migrations in a project of the tool, relative to the project, if the
    /// tool has one by default.
    fn default_dir(&self) -> Option<&'static str>;

    /// What the path of a file says, given `migrations.migrationsDir`: whether it is a
    /// migration, and its version.
    fn file(&self, path: &Path, migrations_dir: &Path) -> FileInfo;

    /// The version of a migration in the tool's format, like the number that starts the name of
    /// its file.
    fn version(&self, text: &str) -> Option<Version>;

    /// How the tool runs the text of a migration file: its units, and the problems with its
    /// markers.
    ///
    /// Markers are found line by line, like the migration tools do, without looking at the SQL.
    fn read(&self, text: &str) -> Script;
}

/// The migration tools, as the settings name them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolId {
    Plain,
    Supabase,
    Dbmate,
    Goose,
}

impl ToolId {
    pub fn tool(self) -> &'static dyn MigrationTool {
        match self {
            ToolId::Plain => &Plain,
            ToolId::Supabase => &Supabase,
            ToolId::Dbmate => &Dbmate,
            ToolId::Goose => &Goose,
        }
    }
}

/// The tools, in the order `init` reports their projects.
pub const TOOLS: &[&dyn MigrationTool] = &[&Supabase, &Dbmate, &Goose, &Plain];

/// What a migration tool keeps in the database it migrates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Database {
    /// Where the tool records the migrations it ran, if it does.
    pub history: Option<History>,
}

/// Where a migration tool records the migrations it ran. Completions rank it low, since a
/// migration rarely queries it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum History {
    /// A table of this name, in any schema.
    Table(&'static str),
    /// Every table of this schema.
    Schema(&'static str),
}

/// What the path of a file says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileInfo {
    pub role: Role,
    /// The version of a migration whose name has one. `migrations.after` compares it.
    pub version: Option<Version>,
}

/// What a file is to the tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// A migration, in the migrations folder. Migration-only rules run on it.
    Migration,
    /// Any other file.
    Other,
}

/// The version of a migration. Versions of one tool are ordered.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Version {
    /// A number, like the timestamp of `20240101120000_init.sql` or the `1` of `0001_init.sql`.
    Number(u64),
}

/// How a tool runs the text of a migration file.
#[derive(Debug, Default)]
pub struct Script {
    /// The parts of the file that the tool runs on their own, in file order. A file without
    /// markers has none, and is one migration.
    pub units: Vec<Unit>,
    /// Lines that look like markers, but that the tool reads differently, with what the tool
    /// does with them.
    look_alikes: Vec<(TextRange, &'static str)>,
    /// What the tool says about statements before the first unit, if it rejects them.
    before_first_unit: Option<String>,
}

impl Script {
    /// The problems with the markers of the file, given its statements: statements before the
    /// first marker, which the tools reject, and lines that look like markers but that the tools
    /// read as plain comments.
    pub fn marker_diagnostics(&self, statements: &[TextRange]) -> Vec<SectionMarkerDiagnostic> {
        let mut diagnostics: Vec<SectionMarkerDiagnostic> = self
            .look_alikes
            .iter()
            .map(|(span, message)| SectionMarkerDiagnostic::new(*span, message))
            .collect();
        if let (Some(first), Some(message)) = (self.units.first(), &self.before_first_unit) {
            diagnostics.extend(
                statements
                    .iter()
                    .take_while(|statement| statement.start() < first.range.start())
                    .map(|statement| SectionMarkerDiagnostic::new(*statement, message)),
            );
        }
        diagnostics.sort_by_key(|diagnostic| diagnostic.span.start());
        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) fn sections(text: &str, tool: ToolId) -> Vec<(&str, Direction, bool)> {
        tool.tool()
            .read(text)
            .units
            .into_iter()
            .map(|unit| (&text[unit.range], unit.direction, unit.transaction))
            .collect()
    }

    pub(crate) fn marker_problems(text: &str, tool: ToolId) -> Vec<(&str, String)> {
        let statements: Vec<TextRange> = pgls_statement_splitter::split(text).ranges;
        tool.tool()
            .read(text)
            .marker_diagnostics(&statements)
            .into_iter()
            .map(|diagnostic| (&text[diagnostic.span], diagnostic.message.to_string()))
            .collect()
    }

    #[test]
    fn every_tool_has_its_id() {
        for tool in TOOLS {
            assert_eq!(tool.id().tool().name(), tool.name());
        }
    }
}
