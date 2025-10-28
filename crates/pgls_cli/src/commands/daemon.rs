use crate::{
    CliDiagnostic, CliSession, open_transport,
    service::{self, ensure_daemon, open_socket, run_daemon},
};
use pgls_console::{ConsoleExt, markup};
use pgls_lsp::ServerFactory;
use pgls_workspace::{TransportError, WorkspaceError, workspace::WorkspaceClient};
use std::path::PathBuf;
use tokio::io;
use tokio::runtime::Runtime;
use tracing::subscriber::Interest;
use tracing::{Instrument, Metadata, debug_span, metadata::LevelFilter};
use tracing_appender::rolling::Rotation;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{
    layer::{Context, Filter},
    prelude::*,
    registry,
};
use tracing_tree::{HierarchicalLayer, time::UtcDateTime};

pub(crate) fn start(
    session: CliSession,
    config_path: Option<PathBuf>,
    log_path: Option<PathBuf>,
    log_file_name_prefix: Option<String>,
) -> Result<(), CliDiagnostic> {
    let rt = Runtime::new()?;
    let did_spawn = rt.block_on(ensure_daemon(
        false,
        config_path,
        log_path,
        log_file_name_prefix,
    ))?;

    if did_spawn {
        session.app.console.log(markup! {
            "The server was successfully started"
        });
    } else {
        session.app.console.log(markup! {
            "The server was already running"
        });
    }

    Ok(())
}

pub(crate) fn stop(session: CliSession) -> Result<(), CliDiagnostic> {
    let rt = Runtime::new()?;

    match open_transport(rt)? {
        Some(transport) => {
            let client = WorkspaceClient::new(transport)?;
            match client.shutdown() {
                // The `ChannelClosed` error is expected since the server can
                // shutdown before sending a response
                Ok(()) | Err(WorkspaceError::TransportError(TransportError::ChannelClosed)) => {}
                Err(err) => return Err(CliDiagnostic::from(err)),
            };

            session.app.console.log(markup! {
                "The server was successfully stopped"
            });
        }
        _ => {
            session.app.console.log(markup! {
                "The server was not running"
            });
        }
    }

    Ok(())
}

pub(crate) fn run_server(
    stop_on_disconnect: bool,
    config_path: Option<PathBuf>,
    log_path: Option<PathBuf>,
    log_file_name_prefix: Option<String>,
    log_level: Option<String>,
    log_kind: Option<String>,
) -> Result<(), CliDiagnostic> {
    setup_tracing_subscriber(log_path, log_file_name_prefix, log_level, log_kind);

    let rt = Runtime::new()?;
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
    let rt = Runtime::new()?;
    rt.block_on(service::print_socket())?;
    Ok(())
}

pub(crate) fn lsp_proxy(
    config_path: Option<PathBuf>,
    log_path: Option<PathBuf>,
    log_file_name_prefix: Option<String>,
) -> Result<(), CliDiagnostic> {
    let rt = Runtime::new()?;
    rt.block_on(start_lsp_proxy(
        &rt,
        config_path,
        log_path,
        log_file_name_prefix,
    ))?;

    Ok(())
}

/// Start a proxy process.
/// Receives a process via `stdin` and then copy the content to the LSP socket.
/// Copy to the process on `stdout` when the LSP responds to a message
async fn start_lsp_proxy(
    rt: &Runtime,
    config_path: Option<PathBuf>,
    log_path: Option<PathBuf>,
    log_file_name_prefix: Option<String>,
) -> Result<(), CliDiagnostic> {
    ensure_daemon(true, config_path, log_path, log_file_name_prefix).await?;

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
/// `pgt-logs/server.log.yyyy-MM-dd-HH` files inside the system temporary
/// directory)
fn setup_tracing_subscriber(
    log_path: Option<PathBuf>,
    log_file_name_prefix: Option<String>,
    log_level: Option<String>,
    log_kind: Option<String>,
) {
    let pgls_log_path = log_path.unwrap_or(pgls_fs::ensure_cache_dir().join("pgt-logs"));

    let appender_builder = tracing_appender::rolling::RollingFileAppender::builder();

    let file_appender = appender_builder
        .filename_prefix(log_file_name_prefix.unwrap_or(String::from("server.log")))
        .max_log_files(7)
        .rotation(Rotation::HOURLY)
        .build(pgls_log_path)
        .expect("Failed to start the logger for the daemon.");

    let filter = PgtLoggingFilter::from(log_level);

    let log_kind = log_kind.unwrap_or("hierarchical".into());

    match log_kind.as_str() {
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

pub fn default_pgls_log_path() -> PathBuf {
    let env = pgls_env::pgls_env();
    match env
        .pgls_log_path
        .value()
        .or_else(|| env.pgls_log_path.value())
    {
        Some(directory) => PathBuf::from(directory),
        None => pgls_fs::ensure_cache_dir().join("pgt-logs"),
    }
}

/// Tracing Filter with two rules:
/// For all crates starting with pgt*, use `PGT_LOG_LEVEL` or CLI option or "info" as default
/// For all other crates, use "info"
struct PgtLoggingFilter(LevelFilter);

impl From<Option<String>> for PgtLoggingFilter {
    fn from(value: Option<String>) -> Self {
        Self(
            value
                .map(|lv_filter| match lv_filter.as_str() {
                    "trace" => LevelFilter::TRACE,
                    "debug" => LevelFilter::DEBUG,
                    "info" => LevelFilter::INFO,
                    "warn" => LevelFilter::WARN,
                    "error" => LevelFilter::ERROR,
                    "off" => LevelFilter::OFF,

                    _ => LevelFilter::INFO,
                })
                .unwrap_or(LevelFilter::INFO),
        )
    }
}

impl PgtLoggingFilter {
    fn is_enabled(&self, meta: &Metadata<'_>) -> bool {
        let filter = if meta.target().starts_with("pgt") {
            self.0
        } else {
            LevelFilter::INFO
        };

        meta.level() <= &filter
    }
}

impl<S> Filter<S> for PgtLoggingFilter {
    fn enabled(&self, meta: &Metadata<'_>, _cx: &Context<'_, S>) -> bool {
        self.is_enabled(meta)
    }

    fn callsite_enabled(&self, meta: &'static Metadata<'static>) -> Interest {
        if self.is_enabled(meta) {
            Interest::always()
        } else {
            Interest::never()
        }
    }

    fn max_level_hint(&self) -> Option<LevelFilter> {
        Some(self.0)
    }
}
