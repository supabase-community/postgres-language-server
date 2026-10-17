use crate::{CliDiagnostic, CliSession};
use pgls_configuration::PartialConfiguration;
use pgls_console::{ConsoleExt, markup};
use pgls_fs::{ConfigName, FileSystem, OpenOptions};
use pgls_workspace::configuration::create_config;
use std::path::Path;

/// The configuration file of a project managed by the Supabase CLI.
const SUPABASE_CONFIG_PATH: &str = "supabase/config.toml";

/// The port of the local Supabase database if `[db] port` is not set.
const SUPABASE_DEFAULT_DB_PORT: u16 = 54322;

/// The local Supabase database, as reported by `supabase status`.
struct SupabaseProject {
    port: u16,
    /// Whether the port could not be read from the Supabase configuration.
    port_unreadable: bool,
}

pub(crate) fn init(mut session: CliSession) -> Result<(), CliDiagnostic> {
    let fs = &mut session.app.fs;
    let config = &mut PartialConfiguration::init();
    let supabase = detect_supabase_project(&**fs);
    if let Some(supabase) = &supabase {
        let db = config.db.get_or_insert_with(Default::default);
        db.host = Some("127.0.0.1".to_string());
        db.port = Some(supabase.port);
        db.username = Some("postgres".to_string());
        db.password = Some("postgres".to_string());
        db.database = Some("postgres".to_string());
    }
    create_config(fs, config)?;
    let file_created = ConfigName::pgls_jsonc();
    session.app.console.log(markup! {
"
Welcome to the Postgres Language Server! Let's get you started...

"<Info><Emphasis>"Files created "</Emphasis></Info>"

  "<Dim>"- "</Dim><Emphasis>{file_created}</Emphasis>"
    Your project configuration.
"
    });
    if let Some(supabase) = supabase {
        if supabase.port_unreadable {
            session.app.console.log(markup! {
                <Warn>"Could not read the database port from "{SUPABASE_CONFIG_PATH}", using the default port "{SUPABASE_DEFAULT_DB_PORT}"."</Warn>
            });
        }
        let port = supabase.port;
        session.app.console.log(markup! {
            <Info><Emphasis>"Supabase project detected"</Emphasis></Info>": the database connection is set to the local Supabase database at 127.0.0.1:"{port}".
"
        });
    }
    Ok(())
}

/// Detects a Supabase project by its `supabase/config.toml` in the working directory.
fn detect_supabase_project(fs: &dyn FileSystem) -> Option<SupabaseProject> {
    let path = Path::new(SUPABASE_CONFIG_PATH);
    if !fs.path_is_file(path) {
        return None;
    }
    let mut contents = String::new();
    let port = fs
        .open_with_options(path, OpenOptions::default().read(true))
        .and_then(|mut file| file.read_to_string(&mut contents))
        .ok()
        .and_then(|_| supabase_db_port(&contents));
    Some(SupabaseProject {
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
        Some(port) => port.as_integer().and_then(|port| u16::try_from(port).ok()),
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
    }
}
