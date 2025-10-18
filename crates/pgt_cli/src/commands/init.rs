use crate::{CliDiagnostic, CliSession};
use pgt_configuration::PartialConfiguration;
use pgt_console::{ConsoleExt, markup};
use pgt_fs::ConfigName;
use pgt_workspace::configuration::create_config;

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
