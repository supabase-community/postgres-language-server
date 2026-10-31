//! This is the main binary

use pgls_cli::{
    CliDiagnostic, CliSession, PgLSCommand, open_transport, pg_l_s_command, setup_panic_handler,
    to_color_mode,
};
use pgls_console::{ConsoleExt, EnvConsole, markup};
use pgls_diagnostics::{Diagnostic, PrintDiagnostic, set_bottom_frame};
use pgls_workspace::workspace;
use std::process::{ExitCode, Termination};
use tokio::runtime::Runtime;

#[cfg(target_os = "windows")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(all(
    any(target_os = "macos", target_os = "linux"),
    not(target_env = "musl")
))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

// Jemallocator does not work on aarch64 with musl, so we'll use the system allocator instead
#[cfg(all(target_env = "musl", target_os = "linux", target_arch = "aarch64"))]
#[global_allocator]
static GLOBAL: std::alloc::System = std::alloc::System;

fn main() -> ExitCode {
    setup_panic_handler();
    set_bottom_frame(main as *const () as usize);

    let mut console = EnvConsole::default();
    let command = pg_l_s_command().fallback_to_usage().run();

    console.set_color(to_color_mode(command.get_color()));

    let is_verbose = command.is_verbose();
    // The command runs on a thread with the stack of the workers: some commands, such as `check`
    // with `--stdin-file-path` and `dblint`, analyse SQL on this thread, and the main thread has
    // only 1 MiB on Windows and 8 MiB elsewhere.
    let result = std::thread::scope(|scope| {
        std::thread::Builder::new()
            .name("pgls::main".to_string())
            .stack_size(pgls_workspace::WORKER_STACK_SIZE_BYTES)
            .spawn_scoped(scope, || run_workspace(&mut console, command))
            .map_err(CliDiagnostic::io_error)?
            .join()
            .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
    });
    match result {
        Err(termination) => {
            if termination.tags().is_verbose() && is_verbose {
                console.error(markup! {{PrintDiagnostic::verbose(&termination)}})
            } else {
                console.error(markup! {{PrintDiagnostic::simple(&termination)}})
            }
            termination.report()
        }
        Ok(_) => ExitCode::SUCCESS,
    }
}

fn run_workspace(console: &mut EnvConsole, command: PgLSCommand) -> Result<(), CliDiagnostic> {
    // If the `--use-server` CLI flag is set, try to open a connection to an
    // existing server socket
    let workspace = if command.should_use_server() {
        let runtime = Runtime::new()?;
        match open_transport(runtime)? {
            Some(transport) => workspace::client(transport)?,
            None => return Err(CliDiagnostic::server_not_running()),
        }
    } else {
        workspace::server()
    };

    let session = CliSession::new(&*workspace, console)?;
    session.run(command)
}
