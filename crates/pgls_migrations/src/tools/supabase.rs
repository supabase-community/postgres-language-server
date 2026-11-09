//! Supabase: the projects of the Supabase CLI and their local database. The Supabase CLI runs
//! every migration file as one migration.

use std::ffi::OsStr;
use std::path::Path;

use pgls_fs::{FileSystem, OpenOptions};

use crate::{
    Database, Detected, FileInfo, History, LocalDatabase, MigrationTool, Root, Script, ToolId,
    Version, naming,
};

/// The folder of a project managed by the Supabase CLI, and its configuration file in it.
const SUPABASE_DIRECTORY: &str = "supabase";
const SUPABASE_CONFIG_FILE: &str = "config.toml";

/// The port of the local Supabase database if `[db] port` is not set.
const SUPABASE_DEFAULT_DB_PORT: u16 = 54322;

pub struct Supabase;

impl MigrationTool for Supabase {
    fn id(&self) -> ToolId {
        ToolId::Supabase
    }

    fn name(&self) -> &'static str {
        "Supabase"
    }

    fn default_dir(&self) -> Option<&'static str> {
        Some("supabase/migrations")
    }

    /// A `supabase` folder with a `config.toml`, and the local database of `supabase status`.
    ///
    /// Run inside the `supabase` folder itself, which has a `config.toml` with a `[db]` section,
    /// `init` configures the project above it.
    fn detect(&self, fs: &dyn FileSystem, root: &Root) -> Option<Detected> {
        let path = root.path(SUPABASE_DIRECTORY).join(SUPABASE_CONFIG_FILE);
        if fs.path_is_file(&path) {
            return Some(self.project(root, read(fs, &path)));
        }

        let path = root.path(SUPABASE_CONFIG_FILE);
        let contents = read(fs, &path)?;
        let in_supabase_directory = root.is_working_directory()
            && path.parent().and_then(Path::file_name) == Some(OsStr::new(SUPABASE_DIRECTORY))
            && contents
                .parse::<toml::Table>()
                .is_ok_and(|config| config.get("db").is_some_and(toml::Value::is_table));
        in_supabase_directory.then(|| Detected {
            configuration_directory: Path::new(".."),
            ..self.project(root, Some(contents))
        })
    }

    fn database(&self) -> Database {
        Database {
            // The Supabase CLI keeps its migrations and seed files in the schema.
            history: Some(History::Schema("supabase_migrations")),
        }
    }

    fn file(&self, path: &Path, migrations_dir: &Path) -> FileInfo {
        naming::numbered(path, migrations_dir, |text| self.version(text))
    }

    fn version(&self, text: &str) -> Option<Version> {
        naming::number(text)
    }

    /// The Supabase CLI runs every migration file as one migration.
    fn read(&self, _text: &str) -> Script {
        Script::default()
    }
}

impl Supabase {
    /// The project with the configuration `contents`, if it could be read. Its folders are
    /// relative to `root`.
    fn project(&self, root: &Root, contents: Option<String>) -> Detected {
        let port = contents.as_deref().and_then(supabase_db_port);
        let mut project = Detected::new(self, root, SUPABASE_DIRECTORY);
        if port.is_none() {
            project.warning = Some(format!(
                "Could not read the database port from {}/{SUPABASE_CONFIG_FILE}, using the default port {SUPABASE_DEFAULT_DB_PORT}.",
                project.directory
            ));
        }
        project.database = Some(LocalDatabase {
            host: "127.0.0.1",
            port: port.unwrap_or(SUPABASE_DEFAULT_DB_PORT),
            username: "postgres",
            password: "postgres",
            database: "postgres",
        });
        project
    }
}

/// The contents of the file at `path`, if it can be read.
fn read(fs: &dyn FileSystem, path: &Path) -> Option<String> {
    let mut contents = String::new();
    fs.open_with_options(path, OpenOptions::default().read(true))
        .and_then(|mut file| file.read_to_string(&mut contents))
        .ok()?;
    Some(contents)
}

/// Returns `[db] port` of a Supabase configuration, or the default port if it is not set.
/// Returns `None` if the configuration is not valid TOML or the port is not a valid port.
fn supabase_db_port(contents: &str) -> Option<u16> {
    let config = contents.parse::<toml::Table>().ok()?;
    let Some(db) = config.get("db") else {
        return Some(SUPABASE_DEFAULT_DB_PORT);
    };
    match db.as_table()?.get("port") {
        Some(port) => port
            .as_integer()
            .and_then(|port| u16::try_from(port).ok())
            .filter(|&port| port != 0),
        None => Some(SUPABASE_DEFAULT_DB_PORT),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ToolId, tests::marker_problems};

    #[test]
    fn supabase_files_are_one_migration() {
        let text = "-- migrate:up\nselect 1;\n-- +goose Down\n";
        assert!(Supabase.read(text).units.is_empty());
        assert!(marker_problems(text, ToolId::Supabase).is_empty());
    }

    #[test]
    fn reads_the_port_from_the_db_section() {
        assert_eq!(supabase_db_port("[db]\nport = 6543\n"), Some(6543));
    }

    #[test]
    fn defaults_the_port_without_db_section() {
        assert_eq!(
            supabase_db_port("project_id = \"test\"\n"),
            Some(SUPABASE_DEFAULT_DB_PORT)
        );
    }

    #[test]
    fn defaults_the_port_without_port_in_db_section() {
        assert_eq!(
            supabase_db_port("[db]\nmajor_version = 17\n"),
            Some(SUPABASE_DEFAULT_DB_PORT)
        );
    }

    #[test]
    fn rejects_invalid_config() {
        assert_eq!(supabase_db_port("[db\nport = 6543\n"), None);
        assert_eq!(supabase_db_port("[db]\nport = \"6543\"\n"), None);
        assert_eq!(supabase_db_port("[db]\nport = 70000\n"), None);
        assert_eq!(supabase_db_port("[db]\nport = 0\n"), None);
    }
}
