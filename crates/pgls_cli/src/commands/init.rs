use crate::{CliDiagnostic, CliSession};
use pgls_configuration::PartialConfiguration;
use pgls_console::{ConsoleExt, markup};
use pgls_fs::ConfigName;
use pgls_workspace::configuration::create_config;
use pgls_workspace::migrations::supabase::{
    SUPABASE_CONFIG_FILE, SUPABASE_DEFAULT_DB_PORT, detect_supabase_project,
    find_supabase_directories, is_inside_supabase_directory,
};
use std::path::Path;

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
        supabase.configure(config);
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
        let migrations = supabase.migrations_dir();
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
