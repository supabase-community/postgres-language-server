use crate::{
    CliDiagnostic, CliSession, LoggingLevel,
    commands::LogOptions,
    logging::LoggingFilter,
    open_transport,
    service::{self, ensure_daemon, open_socket, run_daemon},
};
use pgls_console::{ConsoleExt, markup};
use pgls_lsp::ServerFactory;
use pgls_workspace::{TransportError, WorkspaceError, workspace::WorkspaceClient};
use std::path::PathBuf;
use tokio::io;
use tokio::runtime::Runtime;
use tracing::{Instrument, debug_span};
use tracing_appender::rolling::Rotation;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{prelude::*, registry};
use tracing_tree::{HierarchicalLayer, time::UtcDateTime};

pub(crate) fn start(
    mut session: CliSession,
    config_path: Option<PathBuf>,
    log_options: &LogOptions,
) -> Result<(), CliDiagnostic> {
    let rt = crate::runtime()?;
    let did_spawn = rt.block_on(ensure_daemon(false, config_path, Some(log_options)))?;

    if did_spawn {
        session.console().log(markup! {
            "The server was successfully started"
        });
    } else {
        session.console().log(markup! {
            "The server was already running"
        });
    }

    Ok(())
}

pub(crate) fn stop(mut session: CliSession) -> Result<(), CliDiagnostic> {
    let rt = crate::runtime()?;

    match open_transport(rt)? {
        Some(transport) => {
            let client = WorkspaceClient::new(transport)?;
            match client.shutdown() {
                // The `ChannelClosed` error is expected since the server can
                // shutdown before sending a response
                Ok(()) | Err(WorkspaceError::TransportError(TransportError::ChannelClosed)) => {}
                Err(err) => return Err(CliDiagnostic::from(err)),
            };

            session.console().log(markup! {
                "The server was successfully stopped"
            });
        }
        _ => {
            session.console().log(markup! {
                "The server was not running"
            });
        }
    }

    Ok(())
}

pub(crate) fn run_server(
    stop_on_disconnect: bool,
    config_path: Option<PathBuf>,
    log_options: &LogOptions,
    log_level: LoggingLevel,
    log_kind: &str,
) -> Result<(), CliDiagnostic> {
    setup_tracing_subscriber(log_options, log_level, log_kind);

    let rt = crate::runtime()?;
    let factory = ServerFactory::new(stop_on_disconnect);
    let cancellation = factory.cancellation();
    let span = debug_span!("Running Server", pid = std::process::id());

    rt.block_on(async move {
        tokio::select! {
            res = run_daemon(factory, config_path).instrument(span) => {
                match res {
                    Ok(never) => match never {},
                    Err(err) => Err(err.into()),
                }
            }
            _ = cancellation.notified() => {
                tracing::info!("Received shutdown signal");
                Ok(())
            }
        }
    })
}

pub(crate) fn print_socket() -> Result<(), CliDiagnostic> {
    let rt = crate::runtime()?;
    rt.block_on(service::print_socket())?;
    Ok(())
}

pub(crate) fn lsp_proxy(
    config_path: Option<PathBuf>,
    log_options: &LogOptions,
) -> Result<(), CliDiagnostic> {
    let rt = crate::runtime()?;
    rt.block_on(start_lsp_proxy(&rt, config_path, log_options))?;

    Ok(())
}

/// Start a proxy process.
/// Receives a process via `stdin` and then copy the content to the LSP socket.
/// Copy to the process on `stdout` when the LSP responds to a message
async fn start_lsp_proxy(
    rt: &Runtime,
    config_path: Option<PathBuf>,
    log_options: &LogOptions,
) -> Result<(), CliDiagnostic> {
    ensure_daemon(true, config_path, Some(log_options)).await?;

    match open_socket().await? {
        Some((mut owned_read_half, mut owned_write_half)) => {
            // forward stdin to socket
            let mut stdin = io::stdin();
            let input_handle = rt.spawn(async move {
                loop {
                    match io::copy(&mut stdin, &mut owned_write_half).await {
                        Ok(b) => {
                            if b == 0 {
                                return Ok(());
                            }
                        }
                        Err(err) => return Err(err),
                    };
                }
            });

            // receive socket response to stdout
            let mut stdout = io::stdout();
            let out_put_handle = rt.spawn(async move {
                loop {
                    match io::copy(&mut owned_read_half, &mut stdout).await {
                        Ok(b) => {
                            if b == 0 {
                                return Ok(());
                            }
                        }
                        Err(err) => return Err(err),
                    };
                }
            });

            let _ = input_handle.await;
            let _ = out_put_handle.await;
            Ok(())
        }
        None => Ok(()),
    }
}

/// Set up the [tracing]-based logging system for the server
/// The events received by the subscriber are filtered at the `info` level,
/// then printed using the [HierarchicalLayer] layer, and the resulting text
/// is written to log files rotated on a hourly basis (in
/// `pgls-logs/server.log.yyyy-MM-dd-HH` files inside the system temporary
/// directory)
fn setup_tracing_subscriber(log_options: &LogOptions, log_level: LoggingLevel, log_kind: &str) {
    if log_level == LoggingLevel::None {
        return;
    }

    let appender_builder = tracing_appender::rolling::RollingFileAppender::builder();

    // Without a log file the server runs without logs, rather than not at all.
    let file_appender = match appender_builder
        .filename_prefix(&log_options.log_prefix_name)
        .max_log_files(7)
        .rotation(Rotation::HOURLY)
        .build(&log_options.log_path)
    {
        Ok(file_appender) => file_appender,
        Err(error) => {
            eprintln!("Failed to start the logger for the daemon: {error}");
            return;
        }
    };

    let filter = LoggingFilter::new(log_level);

    match log_kind {
        "bunyan" => {
            registry()
                .with(JsonStorageLayer)
                .with(
                    BunyanFormattingLayer::new("pgls_logs".into(), file_appender)
                        .with_filter(filter),
                )
                .init();
        }

        _ => registry()
            .with(
                HierarchicalLayer::default()
                    .with_indent_lines(true)
                    .with_indent_amount(2)
                    .with_bracketed_fields(true)
                    .with_targets(true)
                    .with_ansi(false)
                    .with_timer(UtcDateTime {
                        higher_precision: false,
                    })
                    .with_writer(file_appender)
                    .with_filter(filter),
            )
            .init(),
    }
}
