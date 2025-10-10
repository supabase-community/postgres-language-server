//! Environment variables and configuration constants for Postgres Tools.
//!
//! This module provides:
//! - Environment variable definitions for runtime configuration
//! - Static constants like version information and website URL
//! - Helper functions for checking build status

use pgt_console::fmt::{Display, Formatter};
use pgt_console::{DebugDisplay, KeyValuePair, markup};
use std::env;
use std::sync::{LazyLock, OnceLock};

/// Returns `true` if this is an unstable build of Postgres Tools
pub fn is_unstable() -> bool {
    VERSION == "0.0.0"
}

/// The internal version of Postgres Tools. This is usually supplied during the CI build
pub static PGT_VERSION: LazyLock<Option<&str>> = LazyLock::new(|| option_env!("PGT_VERSION"));

/// The version of Postgres Tools with fallback logic
pub const VERSION: &str = match option_env!("PGT_VERSION") {
    Some(version) => version,
    None => match option_env!("CARGO_PKG_VERSION") {
        Some(pkg_version) => pkg_version,
        None => "0.0.0",
    },
};

pub static PGT_WEBSITE: &str = "https://pgtools.dev";

pub struct PgTEnv {
    pub pgt_log_path: PgTEnvVariable,
    pub pgt_log_prefix: PgTEnvVariable,
    pub pgt_config_path: PgTEnvVariable,
}

pub static PGT_ENV: OnceLock<PgTEnv> = OnceLock::new();

impl PgTEnv {
    fn new() -> Self {
        Self {
            pgt_log_path: PgTEnvVariable::new(
                "PGT_LOG_PATH",
                "The directory where the Daemon logs will be saved.",
            ),
            pgt_log_prefix: PgTEnvVariable::new(
                "PGT_LOG_PREFIX_NAME",
                "A prefix that's added to the name of the log. Default: `server.log.`",
            ),
            pgt_config_path: PgTEnvVariable::new(
                "PGT_CONFIG_PATH",
                "A path to the configuration file",
            ),
        }
    }
}

pub struct PgTEnvVariable {
    /// The name of the environment variable
    name: &'static str,
    /// The description of the variable.
    // This field will be used in the website to automate its generation
    description: &'static str,
}

impl PgTEnvVariable {
    fn new(name: &'static str, description: &'static str) -> Self {
        Self { name, description }
    }

    /// It attempts to read the value of the variable
    pub fn value(&self) -> Option<String> {
        env::var(self.name).ok()
    }

    /// It returns the description of the variable
    pub fn description(&self) -> &'static str {
        self.description
    }

    /// It returns the name of the variable.
    pub fn name(&self) -> &'static str {
        self.name
    }
}

pub fn pgt_env() -> &'static PgTEnv {
    PGT_ENV.get_or_init(PgTEnv::new)
}

impl Display for PgTEnv {
    fn fmt(&self, fmt: &mut Formatter) -> std::io::Result<()> {
        match self.pgt_log_path.value() {
            None => {
                KeyValuePair(self.pgt_log_path.name, markup! { <Dim>"unset"</Dim> }).fmt(fmt)?;
            }
            Some(value) => {
                KeyValuePair(self.pgt_log_path.name, markup! {{DebugDisplay(value)}}).fmt(fmt)?;
            }
        };
        match self.pgt_log_prefix.value() {
            None => {
                KeyValuePair(self.pgt_log_prefix.name, markup! { <Dim>"unset"</Dim> }).fmt(fmt)?;
            }
            Some(value) => {
                KeyValuePair(self.pgt_log_prefix.name, markup! {{DebugDisplay(value)}}).fmt(fmt)?;
            }
        };

        match self.pgt_config_path.value() {
            None => {
                KeyValuePair(self.pgt_config_path.name, markup! { <Dim>"unset"</Dim> }).fmt(fmt)?;
            }
            Some(value) => {
                KeyValuePair(self.pgt_config_path.name, markup! {{DebugDisplay(value)}})
                    .fmt(fmt)?;
            }
        };

        Ok(())
    }
}
