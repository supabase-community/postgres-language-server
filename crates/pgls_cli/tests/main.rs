mod commands;

use bpaf::ParseFailure;
use pgls_cli::{CliDiagnostic, CliSession, pgt_command};
use pgls_console::{Console, ConsoleExt, markup};
use pgls_fs::FileSystem;
use pgls_workspace::{App, DynRef};

/// Create an [App] instance using the provided [FileSystem] and [Console]
/// instance, and using an in-process "remote" instance of the workspace
pub(crate) fn run_cli<'app>(
    fs: DynRef<'app, dyn FileSystem>,
    console: &'app mut dyn Console,
    args: bpaf::Args,
) -> Result<(), CliDiagnostic> {
    use pgls_cli::SocketTransport;
    use pgls_lsp::ServerFactory;
    use pgls_workspace::{WorkspaceRef, workspace};
    use tokio::{
        io::{duplex, split},
        runtime::Runtime,
    };

    let factory = ServerFactory::default();
    let connection = factory.create(None);

    let runtime = Runtime::new().expect("failed to create runtime");

    let (client, server) = duplex(4096);
    let (stdin, stdout) = split(server);
    runtime.spawn(connection.accept(stdin, stdout));

    let (client_read, client_write) = split(client);
    let transport = SocketTransport::open(runtime, client_read, client_write);

    let workspace = workspace::client(transport).unwrap();
    let app = App::new(fs, console, WorkspaceRef::Owned(workspace));

    let mut session = CliSession { app };
    let command = pgt_command().run_inner(args);
    match command {
        Ok(command) => session.run(command),
        Err(failure) => {
            if let ParseFailure::Stdout(help, _) = &failure {
                let console = &mut session.app.console;
                console.log(markup! {{help.to_string()}});
                Ok(())
            } else {
                Err(CliDiagnostic::parse_error_bpaf(failure))
            }
        }
    }
}
