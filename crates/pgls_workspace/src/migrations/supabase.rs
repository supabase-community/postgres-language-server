//! Supabase: the projects of the Supabase CLI and their local database. The Supabase CLI runs
//! every migration file as one migration.

use std::ffi::OsStr;
use std::path::Path;

use pgls_configuration::migrations as configuration;
use pgls_fs::{FileSystem, OpenOptions};

use super::{
    MigrationTool,
    project::{LocalDatabase, Project, Root},
};

/// The folder of a project managed by the Supabase CLI, and its configuration file in it.
const SUPABASE_DIRECTORY: &str = "supabase";
const SUPABASE_CONFIG_FILE: &str = "config.toml";

/// The port of the local Supabase database if `[db] port` is not set.
const SUPABASE_DEFAULT_DB_PORT: u16 = 54322;

pub(super) struct Supabase;

impl MigrationTool for Supabase {
    fn name(&self) -> &'static str {
        "Supabase"
    }

    fn setting(&self) -> configuration::MigrationTool {
        configuration::MigrationTool::None
    }

    fn default_migrations_dir(&self) -> Option<&'static str> {
        Some("supabase/migrations")
    }

    /// A `supabase` folder with a `config.toml`, and the local database of `supabase status`.
    fn detect(&self, fs: &dyn FileSystem, root: &Root) -> Option<Project> {
        let path = root.path(SUPABASE_DIRECTORY).join(SUPABASE_CONFIG_FILE);
        if !fs.path_is_file(&path) {
            return None;
        }
        let mut contents = String::new();
        let port = fs
            .open_with_options(&path, OpenOptions::default().read(true))
            .and_then(|mut file| file.read_to_string(&mut contents))
            .ok()
            .and_then(|_| supabase_db_port(&contents));
        let mut project = Project::new(self, root, SUPABASE_DIRECTORY);
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
        Some(project)
    }

    /// The folder is named `supabase` and has a `config.toml` with a `[db]` section.
    fn is_tool_directory(&self, fs: &dyn FileSystem, directory: &Path) -> bool {
        let mut contents = String::new();
        directory.file_name() == Some(OsStr::new(SUPABASE_DIRECTORY))
            && fs
                .open_with_options(
                    &directory.join(SUPABASE_CONFIG_FILE),
                    OpenOptions::default().read(true),
                )
                .and_then(|mut file| file.read_to_string(&mut contents))
                .is_ok()
            && contents
                .parse::<toml::Table>()
                .is_ok_and(|config| config.get("db").is_some_and(toml::Value::is_table))
    }
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
