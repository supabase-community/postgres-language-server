//! Migration tools: everything the language server knows about a tool is one [`MigrationTool`].
//!
//! Every method of the enum matches every tool, so one place shows what each tool does. What is
//! longer than a line lives in the tool's module, like the markers of dbmate in `dbmate.rs`. What
//! tools share is written once: the file names here, the marker comments and the units of a file
//! in `markers.rs`, and finding projects in [`detect`].
//!
//! The workspace asks the tool of a file's settings two things, and keeps the answers:
//! - [`MigrationTool::file`]: what the path says, like the version of the migration.
//! - [`MigrationTool::read`]: what the text says, like its up and down sections, and what the
//!   tool replaces in it before it runs it, like goose's `ENVSUB`.
//!
//! `init` asks every tool for its projects with [`detect::find_projects`].
//!
//! The tools:
//! - [`MigrationTool::Plain`]: no tool, every file is one migration.
//! - [`MigrationTool::Supabase`]: the Supabase CLI. Every file is one migration.
//! - [`MigrationTool::Dbmate`] and [`MigrationTool::Goose`]: up and down sections in one file,
//!   separated by marker comments.

mod dbmate;
pub mod detect;
mod goose;
mod markers;
mod supabase;

use std::path::Path;

use pgls_fs::FileSystem;
use pgls_text_size::TextRange;

pub use detect::{Detected, LocalDatabase, Root};
pub use markers::{Direction, SectionMarkerDiagnostic, Unit};

/// A migration tool.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MigrationTool {
    /// No tool: plain SQL files, every file is one migration.
    Plain,
    Supabase,
    Dbmate,
    Goose,
}

impl MigrationTool {
    /// The tools, in the order `init` reports their projects.
    pub const ALL: [MigrationTool; 4] = [Self::Supabase, Self::Dbmate, Self::Goose, Self::Plain];

    /// The name of the tool, as its users know it.
    pub fn name(self) -> &'static str {
        match self {
            Self::Plain => "plain SQL",
            Self::Supabase => "Supabase",
            Self::Dbmate => "dbmate",
            Self::Goose => "goose",
        }
    }

    /// The project of the tool in `root`, for `init`, if the tool has a reliable sign of its
    /// projects.
    pub fn detect(self, fs: &dyn FileSystem, root: &Root) -> Option<Detected> {
        match self {
            // Plain SQL files have no sign of a project.
            Self::Plain => None,
            Self::Supabase => supabase::detect(fs, root),
            Self::Dbmate => dbmate::detect(fs, root),
            // goose has no configuration file and no migrations folder of its own.
            Self::Goose => None,
        }
    }

    /// What the tool keeps in the database it migrates.
    pub fn database(self) -> Database {
        let history = match self {
            // Plain SQL files are run by hand, which records nothing.
            Self::Plain => None,
            // The Supabase CLI keeps its migrations and seed files in the schema.
            Self::Supabase => Some(History::Schema("supabase_migrations")),
            // The default of `--migrations-table`.
            Self::Dbmate => Some(History::Table("schema_migrations")),
            Self::Goose => Some(History::Table("goose_db_version")),
        };
        Database { history }
    }

    /// The folder of the migrations in a project of the tool, relative to the project, if the
    /// tool has one by default.
    pub fn default_dir(self) -> Option<&'static str> {
        match self {
            Self::Plain | Self::Goose => None,
            Self::Supabase => Some("supabase/migrations"),
            Self::Dbmate => Some(dbmate::DBMATE_MIGRATIONS_DIR),
        }
    }

    /// What the path of a file says, given `migrations.migrationsDir`: whether it is a
    /// migration, and its version.
    ///
    /// Every tool names its files the same way, `<version>_<name>.sql`, see [`numbered`].
    pub fn file(self, path: &Path, migrations_dir: &Path) -> FileInfo {
        numbered(path, migrations_dir, |text| self.version(text))
    }

    /// The version of a migration in the tool's format, like the number that starts the name of
    /// its file.
    ///
    /// Every tool numbers its migrations, see [`number`].
    pub fn version(self, text: &str) -> Option<Version> {
        number(text)
    }

    /// How the tool runs the text of a migration file: its units, the problems with its markers,
    /// and what the tool replaces in it.
    ///
    /// Markers are found line by line, like the migration tools do, without looking at the SQL.
    pub fn read(self, text: &str) -> Script {
        match self {
            // The file is one migration, whatever its comments say.
            Self::Plain => Script::default(),
            // The Supabase CLI runs every migration file as one migration.
            Self::Supabase => Script::default(),
            Self::Dbmate => dbmate::read(text),
            Self::Goose => goose::read(text),
        }
    }
}

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
#[derive(Debug, Default, Clone)]
pub struct Script {
    /// The parts of the file that the tool runs on their own, in file order. A file without
    /// markers has none, and is one migration.
    pub units: Vec<Unit>,
    /// Lines that look like markers, but that the tool reads differently, with what the tool
    /// does with them.
    look_alikes: Vec<(TextRange, &'static str)>,
    /// What the tool says about statements before the first unit, if it rejects them.
    before_first_unit: Option<String>,
    /// What the tool replaces in the file before it runs it, like goose's `ENVSUB`, or `None` if
    /// it runs the file as written. Its text is what the analysis splits and parses.
    pub preprocessed: Option<Preprocessed>,
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

/// The text that the analysis reads instead of the file as written, with placeholders where the
/// migration tool replaces parts of it, like goose's `ENVSUB`. It has the length of the file, so
/// that the offsets of diagnostics, fixes and editor positions are the same in both.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preprocessed {
    pub text: String,
    /// The ranges that the tool replaces when it runs the file, in file order. The text holds a
    /// placeholder there, or the file as written where a placeholder wouldn't parse, like in a
    /// string literal.
    pub placeholders: Vec<TextRange>,
}

impl Preprocessed {
    /// Whether `range` overlaps a placeholder. An empty range touches it if it's inside it.
    pub fn touches_placeholder(&self, range: TextRange) -> bool {
        self.placeholders.iter().any(|placeholder| {
            if range.is_empty() {
                placeholder.start() < range.start() && range.start() < placeholder.end()
            } else {
                placeholder.start() < range.end() && range.start() < placeholder.end()
            }
        })
    }
}

/// A file in the migrations folder is a migration. Its version is the number that starts its
/// name, `<version>_<name>.sql`, or the name of its folder, `<version>_<name>/up.sql`.
fn numbered(
    path: &Path,
    migrations_dir: &Path,
    version: impl Fn(&str) -> Option<Version>,
) -> FileInfo {
    if !is_in_migrations_dir(path, migrations_dir) {
        return FileInfo {
            role: Role::Other,
            version: None,
        };
    }

    // In the "root" pattern, all files are directly within the migrations directory, and their
    // names follow <timestamp>_<name>.sql. This is used by Supabase. In the "subdirectory"
    // pattern, each migration is in a subdirectory named <timestamp>_<name>. This is used by
    // Prisma and Drizzle.
    let version_of = |name: Option<&std::ffi::OsStr>| {
        let (number, _name) = name?.to_str()?.split_once('_')?;
        version(number)
    };
    FileInfo {
        role: Role::Migration,
        version: version_of(path.file_name())
            .or_else(|| version_of(path.parent().and_then(Path::file_name))),
    }
}

/// A number with leading zeros, like `000201` or `20240101120000`.
fn number(text: &str) -> Option<Version> {
    text.trim_start_matches('0')
        .parse()
        .ok()
        .map(Version::Number)
}

/// Whether the path is inside the migrations directory.
fn is_in_migrations_dir(path: &Path, migrations_dir: &Path) -> bool {
    if path.starts_with(migrations_dir) {
        return true;
    }
    path.canonicalize()
        .ok()
        .and_then(|canonical_child| {
            migrations_dir
                .canonicalize()
                .ok()
                .map(|canonical_dir| canonical_child.starts_with(&canonical_dir))
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    pub(crate) fn sections(text: &str, tool: MigrationTool) -> Vec<(&str, Direction, bool)> {
        tool.read(text)
            .units
            .into_iter()
            .map(|unit| (&text[unit.range], unit.direction, unit.transaction))
            .collect()
    }

    pub(crate) fn marker_problems(text: &str, tool: MigrationTool) -> Vec<(&str, String)> {
        let statements: Vec<TextRange> = pgls_statement_splitter::split(text).ranges;
        tool.read(text)
            .marker_diagnostics(&statements)
            .into_iter()
            .map(|diagnostic| (&text[diagnostic.span], diagnostic.message.to_string()))
            .collect()
    }

    #[test]
    fn plain_sql_has_no_markers() {
        let text = "-- migrate:up\nselect 1;\n-- +goose Down\n";
        assert!(sections(text, MigrationTool::Plain).is_empty());
        assert!(marker_problems(text, MigrationTool::Plain).is_empty());
        assert!(sections("select 1;", MigrationTool::Dbmate).is_empty());
    }

    fn setup() -> TempDir {
        TempDir::new().expect("Failed to create temp dir")
    }

    fn version_of(path: &Path, migrations_dir: &Path) -> Option<Version> {
        let file = numbered(path, migrations_dir, number);
        assert_eq!(file.role == Role::Migration, file.version.is_some());
        file.version
    }

    #[test]
    fn test_get_migration_root_pattern() {
        let temp_dir = setup();
        let migrations_dir = temp_dir.path().to_path_buf();
        let path = migrations_dir.join("1234567890_create_users.sql");
        fs::write(&path, "").unwrap();

        assert_eq!(
            version_of(&path, &migrations_dir),
            Some(Version::Number(1234567890))
        );
    }

    #[test]
    fn test_get_migration_subdirectory_pattern() {
        let temp_dir = setup();
        let migrations_dir = temp_dir.path().to_path_buf();
        let subdir = migrations_dir.join("1234567890_create_users");
        fs::create_dir(&subdir).unwrap();
        let path = subdir.join("up.sql");
        fs::write(&path, "").unwrap();

        assert_eq!(
            version_of(&path, &migrations_dir),
            Some(Version::Number(1234567890))
        );
    }

    #[test]
    fn test_get_migration_prefix_number() {
        let temp_dir = setup();
        let migrations_dir = temp_dir.path().to_path_buf();
        let path = migrations_dir.join("000201_a_migration.sql");
        fs::write(&path, "").unwrap();

        assert_eq!(
            version_of(&path, &migrations_dir),
            Some(Version::Number(201))
        );
    }

    #[test]
    fn test_get_migration_not_timestamp_in_filename() {
        let migrations_dir = PathBuf::from("/tmp/migrations");
        let path = migrations_dir.join("not_a_migration.sql");

        let file = numbered(&path, &migrations_dir, number);
        assert_eq!(file.role, Role::Migration);
        assert_eq!(file.version, None);
    }

    #[test]
    fn test_get_migration_outside_migrations_dir() {
        let migrations_dir = PathBuf::from("/tmp/migrations");
        let path = PathBuf::from("/tmp/other/1234567890_create_users.sql");

        let file = numbered(&path, &migrations_dir, number);
        assert_eq!(file.role, Role::Other);
        assert_eq!(file.version, None);
    }
}
