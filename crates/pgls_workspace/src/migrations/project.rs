//! The projects of the migration tools: `init` finds them and points the configuration at them.

use std::path::{Path, PathBuf};

use pgls_configuration::PartialConfiguration;
use pgls_configuration::migrations::{self as configuration, PartialMigrationsConfiguration};
use pgls_fs::FileSystem;

use super::{MigrationTool, TOOLS};

/// Folders that never hold the project of a migration tool in a monorepo.
const SKIPPED_DIRECTORIES: &[&str] = &["node_modules", "target", "supabase"];

/// A project of a migration tool, and the settings it tells.
#[derive(Debug)]
pub struct Project {
    /// The name of the tool.
    pub tool: &'static str,
    /// The folder of the tool in the project, like `supabase` or `apps/web/supabase`, relative to
    /// the folder of the configuration file, with `/` as the separator.
    pub directory: String,
    /// The folder of the migrations, relative to the folder of the configuration file.
    pub migrations_dir: Option<String>,
    /// The value of `migrations.tool` for the tool, if it isn't the default.
    pub setting: Option<configuration::MigrationTool>,
    /// The local database of the project, if the tool runs one.
    pub database: Option<LocalDatabase>,
    /// A problem with the project that doesn't stop `init`, like a port it couldn't read.
    pub warning: Option<String>,
}

/// The local database of a project, like the one of `supabase start`.
#[derive(Debug)]
pub struct LocalDatabase {
    pub host: &'static str,
    pub port: u16,
    pub username: &'static str,
    pub password: &'static str,
    pub database: &'static str,
}

impl Project {
    /// A project of `tool` with the tool's folder `directory` in `root`, and the tool's default
    /// migrations folder.
    pub(super) fn new(tool: &dyn MigrationTool, root: &Root, directory: &str) -> Self {
        Self {
            tool: tool.name(),
            directory: root.relative(directory),
            migrations_dir: tool
                .default_migrations_dir()
                .map(|migrations_dir| root.relative(migrations_dir)),
            setting: Some(tool.setting())
                .filter(|setting| *setting != configuration::MigrationTool::None),
            database: None,
            warning: None,
        }
    }

    /// Points the database connection at the local database of the project, and the migrations
    /// at the migrations of the project.
    pub fn configure(&self, config: &mut PartialConfiguration) {
        if let Some(database) = &self.database {
            let db = config.db.get_or_insert_with(Default::default);
            db.host = Some(database.host.to_string());
            db.port = Some(database.port);
            db.username = Some(database.username.to_string());
            db.password = Some(database.password.to_string());
            db.database = Some(database.database.to_string());
            // Statement execution stays off: enabling it is the user's call.
        }
        if self.migrations_dir.is_some() || self.setting.is_some() {
            config.migrations = Some(PartialMigrationsConfiguration {
                migrations_dir: self.migrations_dir.clone(),
                tool: self.setting,
                ..Default::default()
            });
        }
    }
}

/// A folder that may hold the project of a migration tool.
pub(crate) struct Root {
    /// The folder, for the file system.
    path: PathBuf,
    /// The folder relative to the folder of the configuration file, with `/` as the separator and
    /// at the end, or empty for that folder.
    prefix: String,
}

impl Root {
    /// A path in the folder, for the file system.
    pub(super) fn path(&self, relative: &str) -> PathBuf {
        self.path.join(relative)
    }

    /// A path in the folder, relative to the folder of the configuration file.
    pub(super) fn relative(&self, relative: &str) -> String {
        format!("{}{relative}", self.prefix)
    }
}

/// The folder of the configuration, relative to the working directory: the folder above it if
/// the working directory is the folder of a tool inside a project, like `supabase`, so that the
/// configuration applies to the whole project like the tool's does, and the editor finds it
/// when it opens the project.
pub fn configuration_directory(fs: &dyn FileSystem) -> &'static Path {
    let working_directory = fs.working_directory().unwrap_or_default();
    if TOOLS
        .iter()
        .any(|tool| tool.is_tool_directory(fs, &working_directory))
    {
        Path::new("..")
    } else {
        Path::new("")
    }
}

/// The projects of the migration tools for the configuration in `configuration_directory`,
/// relative to the working directory: the ones in it, or else, if it is the working directory,
/// the ones in the folders one or two levels below it, as in a monorepo (`supabase/`,
/// `db/supabase/`, `apps/web/supabase/`).
pub fn find_projects(fs: &dyn FileSystem, configuration_directory: &Path) -> Vec<Project> {
    let working_directory = fs.working_directory().unwrap_or_default();
    let detect = |relative: &str| {
        let root = Root {
            path: working_directory
                .join(configuration_directory)
                .join(relative),
            prefix: if relative.is_empty() {
                String::new()
            } else {
                format!("{relative}/")
            },
        };
        TOOLS
            .iter()
            .filter_map(|tool| tool.detect(fs, &root))
            .collect::<Vec<_>>()
    };
    let found = detect("");
    if !found.is_empty() || !configuration_directory.as_os_str().is_empty() {
        return found;
    }
    let mut found = Vec::new();
    for child in subdirectories(&working_directory) {
        let projects = detect(&child);
        if !projects.is_empty() {
            found.extend(projects);
            continue;
        }
        for grandchild in subdirectories(&working_directory.join(&child)) {
            found.extend(detect(&format!("{child}/{grandchild}")));
        }
    }
    found
}

/// The names of the folders in `directory`, sorted, without hidden and skipped ones.
fn subdirectories(directory: &Path) -> Vec<String> {
    entries(directory, true)
        .into_iter()
        .filter(|name| !name.starts_with('.') && !SKIPPED_DIRECTORIES.contains(&name.as_str()))
        .collect()
}

/// The names of the files in `directory`, sorted.
pub(super) fn files(directory: &Path) -> Vec<String> {
    entries(directory, false)
}

fn entries(directory: &Path, directories: bool) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_type()
                .is_ok_and(|file_type| file_type.is_dir() == directories)
        })
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect();
    names.sort();
    names
}

#[cfg(test)]
mod tests {
    use std::fs;

    use pgls_fs::OsFileSystem;
    use tempfile::TempDir;

    use super::*;

    fn find(dir: &TempDir) -> Vec<(String, &'static str, Option<String>)> {
        let fs = OsFileSystem::new(dir.path().to_path_buf());
        find_projects(&fs, Path::new(""))
            .into_iter()
            .map(|project| (project.directory, project.tool, project.migrations_dir))
            .collect()
    }

    fn write(dir: &TempDir, path: &str, contents: &str) {
        let path = dir.path().join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }

    #[test]
    fn finds_a_supabase_project() {
        let dir = TempDir::new().unwrap();
        write(&dir, "supabase/config.toml", "[db]\nport = 6543\n");
        let fs = OsFileSystem::new(dir.path().to_path_buf());
        let [project] = find_projects(&fs, Path::new("")).try_into().unwrap();
        assert_eq!(project.tool, "Supabase");
        assert_eq!(project.directory, "supabase");
        assert_eq!(
            project.migrations_dir.as_deref(),
            Some("supabase/migrations")
        );
        assert_eq!(project.setting, None);
        assert_eq!(project.database.map(|database| database.port), Some(6543));
        assert!(project.warning.is_none());
    }

    #[test]
    fn finds_a_dbmate_project() {
        let dir = TempDir::new().unwrap();
        write(
            &dir,
            "db/migrations/20240101000000_create_users.sql",
            "-- migrate:up\ncreate table users (id int);\n\n-- migrate:down\ndrop table users;\n",
        );
        let fs = OsFileSystem::new(dir.path().to_path_buf());
        let [project] = find_projects(&fs, Path::new("")).try_into().unwrap();
        assert_eq!(project.tool, "dbmate");
        assert_eq!(project.directory, "db");
        assert_eq!(project.migrations_dir.as_deref(), Some("db/migrations"));
        assert_eq!(project.setting, Some(configuration::MigrationTool::Dbmate));
        assert!(project.database.is_none());

        let mut config = PartialConfiguration::default();
        project.configure(&mut config);
        let migrations = config.migrations.unwrap();
        assert_eq!(migrations.migrations_dir.as_deref(), Some("db/migrations"));
        assert_eq!(migrations.tool, Some(configuration::MigrationTool::Dbmate));
        assert!(config.db.is_none());
    }

    #[test]
    fn db_migrations_without_dbmate_markers_are_no_project() {
        let dir = TempDir::new().unwrap();
        write(
            &dir,
            "db/migrations/20240101000000_create_users.sql",
            "create table users (id int);\n",
        );
        write(&dir, "db/migrations/notes.sql", "-- migrate:up\n");
        assert!(find(&dir).is_empty());
    }

    #[test]
    fn finds_the_projects_below_the_working_directory() {
        let dir = TempDir::new().unwrap();
        write(&dir, "apps/web/supabase/config.toml", "");
        write(
            &dir,
            "services/api/db/migrations/1_init.sql",
            "-- migrate:up\n",
        );
        write(&dir, "node_modules/pkg/supabase/config.toml", "");
        assert_eq!(
            find(&dir),
            vec![
                (
                    "apps/web/supabase".to_string(),
                    "Supabase",
                    Some("apps/web/supabase/migrations".to_string())
                ),
                (
                    "services/api/db".to_string(),
                    "dbmate",
                    Some("services/api/db/migrations".to_string())
                ),
            ]
        );
    }

    #[test]
    fn the_projects_in_the_working_directory_come_first() {
        let dir = TempDir::new().unwrap();
        write(&dir, "supabase/config.toml", "");
        write(&dir, "db/migrations/1_init.sql", "-- migrate:up\n");
        write(&dir, "apps/web/supabase/config.toml", "");
        assert_eq!(
            find(&dir)
                .into_iter()
                .map(|(directory, tool, _)| (directory, tool))
                .collect::<Vec<_>>(),
            vec![
                ("supabase".to_string(), "Supabase"),
                ("db".to_string(), "dbmate")
            ]
        );
    }

    #[test]
    fn configures_the_project_above_the_supabase_folder() {
        let dir = TempDir::new().unwrap();
        write(&dir, "supabase/config.toml", "[db]\nport = 6543\n");
        let fs = OsFileSystem::new(dir.path().join("supabase"));
        assert_eq!(configuration_directory(&fs), Path::new(".."));
        let fs = OsFileSystem::new(dir.path().to_path_buf());
        assert_eq!(configuration_directory(&fs), Path::new(""));
    }
}
