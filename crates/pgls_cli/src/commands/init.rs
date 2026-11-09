use crate::{CliDiagnostic, CliSession};
use pgls_configuration::PartialConfiguration;
use pgls_configuration::migrations::{PartialMigrationsConfiguration, SectionMarkers};
use pgls_console::{ConsoleExt, markup};
use pgls_fs::ConfigName;
use pgls_migrations::{
    Detected, ToolId,
    detect::{configuration_directory, find_projects},
};
use pgls_workspace::configuration::create_config;

pub(crate) fn init(mut session: CliSession) -> Result<(), CliDiagnostic> {
    let fs = &mut session.app.fs;
    let config = &mut PartialConfiguration::init();
    let project_directory = configuration_directory(&**fs);
    let mut projects = find_projects(&**fs, project_directory);
    let (project, ambiguous) = if projects.len() == 1 {
        (projects.pop(), Vec::new())
    } else {
        (None, projects)
    };
    if let Some(project) = &project {
        configure(project, config);
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
    if let Some(project) = project {
        if let Some(warning) = &project.warning {
            session.app.console.log(markup! {
                <Warn>{warning}</Warn>
            });
        }
        let tool = project.tool.tool().name();
        let directory = &project.directory;
        let settings = configured_settings(&project);
        session.app.console.log(markup! {
            <Info><Emphasis>{tool}" project detected"</Emphasis></Info>" in "{directory}": "{settings}".
"
        });
    } else if !ambiguous.is_empty() {
        let first = ambiguous[0].tool;
        let projects = if ambiguous.iter().all(|project| project.tool == first) {
            let first = first.tool().name();
            let directories: Vec<&str> = ambiguous
                .iter()
                .map(|project| project.directory.as_str())
                .collect();
            format!("several {first} projects ({})", directories.join(", "))
        } else {
            let directories: Vec<String> = ambiguous
                .iter()
                .map(|project| format!("{}: {}", project.directory, project.tool.tool().name()))
                .collect();
            format!("several migration projects ({})", directories.join(", "))
        };
        session.app.console.log(markup! {
            <Warn>"Found "{projects}", so the database connection is not set. Run "<Emphasis>"init"</Emphasis>" in the folder of one of them, or set "<Emphasis>"db"</Emphasis>" and "<Emphasis>"migrations.migrationsDir"</Emphasis>" yourself."</Warn>
        });
    }
    Ok(())
}

/// The section markers of the tool's migration files, if it has any.
fn section_markers(tool: ToolId) -> Option<SectionMarkers> {
    match tool {
        ToolId::Plain | ToolId::Supabase => None,
        ToolId::Dbmate => Some(SectionMarkers::Dbmate),
        ToolId::Goose => Some(SectionMarkers::Goose),
    }
}

/// Points the database connection at the local database of the project, and the migrations at
/// the migrations of the project.
fn configure(project: &Detected, config: &mut PartialConfiguration) {
    if let Some(database) = &project.database {
        let db = config.db.get_or_insert_with(Default::default);
        db.host = Some(database.host.to_string());
        db.port = Some(database.port);
        db.username = Some(database.username.to_string());
        db.password = Some(database.password.to_string());
        db.database = Some(database.database.to_string());
        // Statement execution stays off: enabling it is the user's call.
    }
    let section_markers = section_markers(project.tool);
    if project.migrations_dir.is_some() || section_markers.is_some() {
        config.migrations = Some(PartialMigrationsConfiguration {
            migrations_dir: project.migrations_dir.clone(),
            section_markers,
            ..Default::default()
        });
    }
}

/// The settings that `init` took from the project, like "the database connection is set to the
/// local Supabase database at 127.0.0.1:54322, and the migrations to supabase/migrations".
fn configured_settings(project: &Detected) -> String {
    let mut settings = Vec::new();
    if let Some(database) = &project.database {
        settings.push((
            "the database connection",
            "is",
            format!(
                "the local {} database at {}:{}",
                project.tool.tool().name(),
                database.host,
                database.port
            ),
        ));
    }
    if let Some(migrations_dir) = &project.migrations_dir {
        settings.push(("the migrations", "are", migrations_dir.clone()));
    }
    if section_markers(project.tool).is_some() {
        settings.push((
            "the section markers",
            "are",
            project.tool.tool().name().to_string(),
        ));
    }
    let mut text = String::new();
    for (index, (setting, verb, value)) in settings.iter().enumerate() {
        if index == 0 {
            text.push_str(&format!("{setting} {verb} set to {value}"));
        } else if index + 1 == settings.len() {
            text.push_str(&format!(", and {setting} to {value}"));
        } else {
            text.push_str(&format!(", {setting} to {value}"));
        }
    }
    text
}
