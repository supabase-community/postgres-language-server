use crate::{CliDiagnostic, CliSession};
use pgls_configuration::PartialConfiguration;
use pgls_console::{ConsoleExt, markup};
use pgls_fs::ConfigName;
use pgls_workspace::configuration::create_config;

pub(crate) fn init(mut session: CliSession) -> Result<(), CliDiagnostic> {
    let fs = &mut session.app.fs;
    let config = &mut PartialConfiguration::init();
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
    Ok(())
}
