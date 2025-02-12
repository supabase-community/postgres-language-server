use crate::{CliDiagnostic, CliSession};
use pglt_configuration::PartialConfiguration;
use pglt_console::{markup, ConsoleExt};
use pglt_fs::ConfigName;
use pglt_workspace::configuration::create_config;

pub(crate) fn init(mut session: CliSession) -> Result<(), CliDiagnostic> {
    let fs = &mut session.app.fs;
    create_config(fs, PartialConfiguration::init())?;
    let file_created = ConfigName::pglsp_toml();
    session.app.console.log(markup! {
"
Welcome to the Postgres Language Server! Let's get you started...

"<Info><Emphasis>"Files created "</Emphasis></Info>"

  "<Dim>"- "</Dim><Emphasis>{file_created}</Emphasis>"
    Your project configuration.
"
    });
    Ok(())
}
