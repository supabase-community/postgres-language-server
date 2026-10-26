use crate::{CliDiagnostic, CliSession};
use pgls_configuration::PartialConfiguration;
use pgls_configuration::migrations::PartialMigrationsConfiguration;
use pgls_console::{ConsoleExt, markup};
use pgls_fs::{ConfigName, FileSystem, OpenOptions};
use pgls_workspace::configuration::create_config;
use std::ffi::OsStr;
use std::path::Path;

/// The folder of a project managed by the Supabase CLI, and its configuration file in it.
const SUPABASE_DIRECTORY: &str = "supabase";
const SUPABASE_CONFIG_FILE: &str = "config.toml";

/// The port of the local Supabase database if `[db] port` is not set.
const SUPABASE_DEFAULT_DB_PORT: u16 = 54322;

/// Folders that never hold the Supabase project of a monorepo.
const SKIPPED_DIRECTORIES: &[&str] = &["node_modules", "target", SUPABASE_DIRECTORY];

/// A Supabase project and its local database, as reported by `supabase status`.
struct SupabaseProject {
    /// The `supabase` folder, relative to the folder of the configuration file, with `/` as the
    /// separator.
    directory: String,
    port: u16,
    /// Whether the port could not be read from the Supabase configuration.
    port_unreadable: bool,
}

pub(crate) fn init(mut session: CliSession) -> Result<(), CliDiagnostic> {
    let fs = &mut session.app.fs;
    let config = &mut PartialConfiguration::init();
    // Inside the `supabase` folder, the configuration goes into the project folder above it, so
    // that it applies to the whole project like the Supabase configuration does, and the editor
    // finds it when it opens the project.
    let project_directory = if is_inside_supabase_directory(&**fs) {
        Path::new("..")
    } else {
        Path::new("")
    };
    let (supabase, ambiguous) = match find_supabase_directories(&**fs, project_directory).as_slice()
    {
        [directory] => (
            detect_supabase_project(&**fs, project_directory, directory),
            Vec::new(),
        ),
        directories => (None, directories.to_vec()),
    };
    if let Some(supabase) = &supabase {
        let db = config.db.get_or_insert_with(Default::default);
        db.host = Some("127.0.0.1".to_string());
        db.port = Some(supabase.port);
        db.username = Some("postgres".to_string());
        db.password = Some("postgres".to_string());
        db.database = Some("postgres".to_string());
        // Statement execution stays off: enabling it is the user's call.
        config.migrations = Some(PartialMigrationsConfiguration {
            migrations_dir: Some(format!("{}/migrations", supabase.directory)),
            ..Default::default()
        });
    }
    create_config(fs, project_directory, config)?;
    let file_created = project_directory
        .join(ConfigName::pgls_jsonc())
        .display()
        .to_string();
    session.app.console.log(markup! {
"
Welcome to the Postgres Language Server! Let's get you started...

"<Info><Emphasis>"Files created "</Emphasis></Info>"

  "<Dim>"- "</Dim><Emphasis>{file_created}</Emphasis>"
    Your project configuration.
"
    });
    if let Some(supabase) = supabase {
        let config_path = format!("{}/{SUPABASE_CONFIG_FILE}", supabase.directory);
        if supabase.port_unreadable {
            session.app.console.log(markup! {
                <Warn>"Could not read the database port from "{config_path}", using the default port "{SUPABASE_DEFAULT_DB_PORT}"."</Warn>
            });
        }
        let port = supabase.port;
        let migrations = format!("{}/migrations", supabase.directory);
        session.app.console.log(markup! {
            <Info><Emphasis>"Supabase project detected"</Emphasis></Info>" in "{supabase.directory}": the database connection is set to the local Supabase database at 127.0.0.1:"{port}", and the migrations to "{migrations}".
"
        });
    } else if !ambiguous.is_empty() {
        let directories = ambiguous.join(", ");
        session.app.console.log(markup! {
            <Warn>"Found several Supabase projects ("{directories}"), so the database connection is not set. Run "<Emphasis>"init"</Emphasis>" in the folder of one of them, or set "<Emphasis>"db"</Emphasis>" and "<Emphasis>"migrations.migrationsDir"</Emphasis>" yourself."</Warn>
        });
    }
    Ok(())
}

/// Whether the working directory is the `supabase` folder of a project: it is named like it and
/// has a `config.toml` with a `[db]` section.
fn is_inside_supabase_directory(fs: &dyn FileSystem) -> bool {
    let is_named_supabase = fs
        .working_directory()
        .is_some_and(|directory| directory.file_name() == Some(OsStr::new(SUPABASE_DIRECTORY)));
    let mut contents = String::new();
    is_named_supabase
        && fs
            .open_with_options(
                Path::new(SUPABASE_CONFIG_FILE),
                OpenOptions::default().read(true),
            )
            .and_then(|mut file| file.read_to_string(&mut contents))
            .is_ok()
        && contents
            .parse::<toml::Table>()
            .is_ok_and(|config| config.get("db").is_some_and(toml::Value::is_table))
}

/// The `supabase` folders with a `config.toml`, relative to `project_directory`: the one in it,
/// or else, if it is the working directory, the ones in the folders one or two levels below it,
/// as in a monorepo (`supabase/`, `db/supabase/`, `apps/web/supabase/`).
fn find_supabase_directories(fs: &dyn FileSystem, project_directory: &Path) -> Vec<String> {
    let is_project = |directory: &Path| {
        fs.path_is_file(
            &directory
                .join(SUPABASE_DIRECTORY)
                .join(SUPABASE_CONFIG_FILE),
        )
    };
    if is_project(project_directory) {
        return vec![SUPABASE_DIRECTORY.to_string()];
    }
    if !project_directory.as_os_str().is_empty() {
        return Vec::new();
    }
    let Some(working_directory) = fs.working_directory() else {
        return Vec::new();
    };
    let mut found = Vec::new();
    for child in subdirectories(&working_directory) {
        if is_project(&working_directory.join(&child)) {
            found.push(format!("{child}/{SUPABASE_DIRECTORY}"));
            continue;
        }
        for grandchild in subdirectories(&working_directory.join(&child)) {
            if is_project(&working_directory.join(&child).join(&grandchild)) {
                found.push(format!("{child}/{grandchild}/{SUPABASE_DIRECTORY}"));
            }
        }
    }
    found
}

/// The names of the folders in `directory`, sorted, without hidden and skipped ones.
fn subdirectories(directory: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|file_type| file_type.is_dir()))
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| !name.starts_with('.') && !SKIPPED_DIRECTORIES.contains(&name.as_str()))
        .collect();
    names.sort();
    names
}

/// Reads the Supabase project in `directory`, relative to `project_directory`, which is relative
/// to the working directory.
fn detect_supabase_project(
    fs: &dyn FileSystem,
    project_directory: &Path,
    directory: &str,
) -> Option<SupabaseProject> {
    let path = project_directory.join(directory).join(SUPABASE_CONFIG_FILE);
    if !fs.path_is_file(&path) {
        return None;
    }
    let mut contents = String::new();
    let port = fs
        .open_with_options(&path, OpenOptions::default().read(true))
        .and_then(|mut file| file.read_to_string(&mut contents))
        .ok()
        .and_then(|_| supabase_db_port(&contents));
    Some(SupabaseProject {
        directory: directory.to_string(),
        port: port.unwrap_or(SUPABASE_DEFAULT_DB_PORT),
        port_unreadable: port.is_none(),
    })
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
