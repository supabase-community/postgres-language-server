use pgls_console::fmt::Formatter;
use pgls_console::{ConsoleExt, fmt, markup};
use pgls_workspace::workspace::ServerInfo;

use crate::{CliDiagnostic, CliSession, VERSION};

/// Handle of the `version` command. Prints a more in detail version.
pub(crate) fn version(mut session: CliSession) -> Result<(), CliDiagnostic> {
    {
        let console = session.console();
        console.log(markup! {
            "CLI:        "{VERSION}
        });
    }

    let server_info = session.workspace().server_info().cloned();
    let console = session.console();

    match server_info {
        None => {
            console.log(markup! {
                "Server:     "<Dim>"not connected"</Dim>
            });
        }
        Some(info) => {
            console.log(markup! {
"Server:
  Name:     "{info.name}"
  Version:  "{DisplayServerVersion(&info)}
            });
        }
    };

    Ok(())
}

pub(super) struct DisplayServerVersion<'a>(pub &'a ServerInfo);

impl fmt::Display for DisplayServerVersion<'_> {
    fn fmt(&self, fmt: &mut Formatter) -> std::io::Result<()> {
        match &self.0.version {
            None => markup!(<Dim>"-"</Dim>).fmt(fmt),
            Some(version) => {
                write!(fmt, "{version}")
            }
        }
    }
}
