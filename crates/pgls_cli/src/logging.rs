use std::fmt::{Display, Formatter};
use std::str::FromStr;
use tracing::Metadata;
use tracing::subscriber::Interest;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::{Context, Filter, SubscriberExt};
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{Layer, registry};

pub fn setup_cli_subscriber(level: LoggingLevel, kind: LoggingKind) {
    if level == LoggingLevel::None {
        return;
    }

    let format = tracing_subscriber::fmt::layer()
        .with_level(true)
        .with_target(false)
        .with_thread_names(true)
        .with_file(true)
        .with_ansi(true);
    match kind {
        LoggingKind::Pretty => {
            let format = format.pretty();
            registry()
                .with(format.with_filter(LoggingFilter::new(level)))
                .init()
        }
        LoggingKind::Compact => {
            let format = format.compact();
            registry()
                .with(format.with_filter(LoggingFilter::new(level)))
                .init()
        }
        LoggingKind::Json => {
            let format = format.json().flatten_event(true);

            registry()
                .with(format.with_filter(LoggingFilter::new(level)))
                .init()
        }
    };
}

/// The level of the logs, for the CLI (`--log-level`) and the language server (`__run_server
/// --log-level`). Both read `PGLS_LOG_LEVEL`, so they must accept the same values.
#[derive(Copy, Debug, Default, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum LoggingLevel {
    /// No logs should be shown
    #[default]
    None,
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LoggingLevel {
    fn to_filter_level(self) -> Option<LevelFilter> {
        match self {
            LoggingLevel::None => None,
            LoggingLevel::Trace => Some(LevelFilter::TRACE),
            LoggingLevel::Debug => Some(LevelFilter::DEBUG),
            LoggingLevel::Info => Some(LevelFilter::INFO),
            LoggingLevel::Warn => Some(LevelFilter::WARN),
            LoggingLevel::Error => Some(LevelFilter::ERROR),
        }
    }
}

impl FromStr for LoggingLevel {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            // The server used to take `off`.
            "none" | "off" => Ok(Self::None),
            "trace" => Ok(Self::Trace),
            "debug" => Ok(Self::Debug),
            "info" => Ok(Self::Info),
            "warn" => Ok(Self::Warn),
            "error" => Ok(Self::Error),
            _ => Err(format!(
                "value {s:?} is not a log level: use none, trace, debug, info, warn or error"
            )),
        }
    }
}

impl Display for LoggingLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LoggingLevel::None => write!(f, "none"),
            LoggingLevel::Trace => write!(f, "trace"),
            LoggingLevel::Debug => write!(f, "debug"),
            LoggingLevel::Info => write!(f, "info"),
            LoggingLevel::Warn => write!(f, "warn"),
            LoggingLevel::Error => write!(f, "error"),
        }
    }
}

/// Tracing filter of the CLI and the language server:
/// - spans and events of crates whose name starts with `pgls` at `level`
/// - spans and events of all other crates at level info or higher
/// - nothing at level `none`
pub(crate) struct LoggingFilter {
    level: LoggingLevel,
}

impl LoggingFilter {
    pub(crate) fn new(level: LoggingLevel) -> Self {
        Self { level }
    }

    fn is_enabled(&self, meta: &Metadata<'_>) -> bool {
        let Some(level) = self.level.to_filter_level() else {
            return false;
        };
        let filter = if meta.target().starts_with("pgls") {
            level
        } else {
            LevelFilter::INFO
        };

        meta.level() <= &filter
    }
}

impl<S> Filter<S> for LoggingFilter {
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
        Some(
            self.level
                .to_filter_level()
                .map_or(LevelFilter::OFF, |level| level.max(LevelFilter::INFO)),
        )
    }
}

/// The kind of logging
#[derive(Copy, Debug, Default, Clone, Eq, PartialEq)]
pub enum LoggingKind {
    /// A pretty log on multiple lines with nice colours
    #[default]
    Pretty,
    /// A more cluttered logging
    Compact,
    /// Logs are emitted in JSON format
    Json,
}

impl Display for LoggingKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LoggingKind::Pretty => write!(f, "pretty"),
            LoggingKind::Compact => write!(f, "compact"),
            LoggingKind::Json => write!(f, "json"),
        }
    }
}

impl FromStr for LoggingKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "compact" => Ok(Self::Compact),
            "pretty" => Ok(Self::Pretty),
            "json" => Ok(Self::Json),
            _ => Err("This log kind doesn't exist".to_string()),
        }
    }
}
