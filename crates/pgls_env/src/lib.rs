//! Environment variables and configuration constants for Postgres Language Server.
//!
//! This module provides:
//! - Environment variable definitions for runtime configuration
//! - Static constants like version information and website URL
//! - Helper functions for checking build status

use pgls_console::fmt::{Display, Formatter};
use pgls_console::{DebugDisplay, KeyValuePair, markup};
use std::env;
use std::sync::OnceLock;

/// Returns `true` if this is an unstable build of Postgres Language Server
pub fn is_unstable() -> bool {
    VERSION == "0.0.0"
}

/// The version of Postgres Language Server. This is usually supplied during the CI build.
pub const VERSION: &str = match option_env!("PGLS_VERSION") {
    Some(version) => version,
    None => match option_env!("PGT_VERSION") {
        Some(version) => version,
        None => match option_env!("CARGO_PKG_VERSION") {
            Some(pkg_version) => pkg_version,
            None => "0.0.0",
        },
    },
};

pub static PGLS_WEBSITE: &str = "https://pg-language-server.com";

pub struct PgLSEnv {
    pub pgls_log_path: PgLSEnvVariable,
    pub pgls_log_level: PgLSEnvVariable,
    pub pgls_log_prefix: PgLSEnvVariable,
    pub pgls_config_path: PgLSEnvVariable,

    // DEPRECATED - kept for backward compatibility
    pub pgt_log_path: PgLSEnvVariable,
    pub pgt_log_level: PgLSEnvVariable,
    pub pgt_log_prefix: PgLSEnvVariable,
    pub pgt_config_path: PgLSEnvVariable,
}

pub static PGT_ENV: OnceLock<PgLSEnv> = OnceLock::new();

impl PgLSEnv {
    fn new() -> Self {
        Self {
            pgls_log_path: PgLSEnvVariable::new(
                "PGLS_LOG_PATH",
                "The directory where the Daemon logs will be saved.",
            ),
            pgls_log_level: PgLSEnvVariable::new(
                "PGLS_LOG_LEVEL",
                "Allows to change the log level. Default is debug. This will only affect \"pgls*\" crates. All others are logged with info level.",
            ),
            pgls_log_prefix: PgLSEnvVariable::new(
                "PGLS_LOG_PREFIX_NAME",
                "A prefix that's added to the name of the log. Default: `server.log.`",
            ),
            pgls_config_path: PgLSEnvVariable::new(
                "PGLS_CONFIG_PATH",
                "A path to the configuration file",
            ),

            pgt_log_path: PgLSEnvVariable::new(
                "PGT_LOG_PATH",
                "The directory where the Daemon logs will be saved. Deprecated, use PGLS_LOG_PATH instead.",
            ),
            pgt_log_level: PgLSEnvVariable::new(
                "PGT_LOG_LEVEL",
                "Allows to change the log level. Default is debug. This will only affect \"pgls*\" crates. All others are logged with info level. Deprecated, use PGLS_LOG_LEVEL instead.",
            ),
            pgt_log_prefix: PgLSEnvVariable::new(
                "PGT_LOG_PREFIX_NAME",
                "A prefix that's added to the name of the log. Default: `server.log`. Deprecated, use PGLS_LOG_PREFIX_NAME instead.",
            ),
            pgt_config_path: PgLSEnvVariable::new(
                "PGT_CONFIG_PATH",
                "A path to the configuration file. Deprecated, use PGLS_CONFIG_PATH instead.",
            ),
        }
    }
}

pub struct PgLSEnvVariable {
    /// The name of the environment variable
    name: &'static str,
    /// The description of the variable.
    // This field will be used in the website to automate its generation
    description: &'static str,
}

impl PgLSEnvVariable {
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

pub fn pgls_env() -> &'static PgLSEnv {
    PGT_ENV.get_or_init(PgLSEnv::new)
}

impl Display for PgLSEnv {
    fn fmt(&self, fmt: &mut Formatter) -> std::io::Result<()> {
        match self.pgls_log_path.value() {
            None => {
                KeyValuePair(self.pgls_log_path.name, markup! { <Dim>"unset"</Dim> }).fmt(fmt)?;
            }
            Some(value) => {
                KeyValuePair(self.pgls_log_path.name, markup! {{DebugDisplay(value)}}).fmt(fmt)?;
            }
        };
        match self.pgls_log_level.value() {
            None => {
                KeyValuePair(self.pgls_log_level.name, markup! { <Dim>"unset"</Dim> }).fmt(fmt)?;
            }
            Some(value) => {
                KeyValuePair(self.pgls_log_level.name, markup! {{DebugDisplay(value)}}).fmt(fmt)?;
            }
        };
        match self.pgls_log_prefix.value() {
            None => {
                KeyValuePair(self.pgls_log_prefix.name, markup! { <Dim>"unset"</Dim> }).fmt(fmt)?;
            }
            Some(value) => {
                KeyValuePair(self.pgls_log_prefix.name, markup! {{DebugDisplay(value)}})
                    .fmt(fmt)?;
            }
        };

        match self.pgls_config_path.value() {
            None => {
                KeyValuePair(self.pgls_config_path.name, markup! { <Dim>"unset"</Dim> })
                    .fmt(fmt)?;
            }
            Some(value) => {
                KeyValuePair(self.pgls_config_path.name, markup! {{DebugDisplay(value)}})
                    .fmt(fmt)?;
            }
        };

        match self.pgt_log_path.value() {
            None => {
                KeyValuePair(self.pgt_log_path.name, markup! { <Dim>"unset"</Dim> }).fmt(fmt)?;
            }
            Some(value) => {
                KeyValuePair(self.pgt_log_path.name, markup! {{DebugDisplay(value)}}).fmt(fmt)?;
            }
        };
        match self.pgt_log_level.value() {
            None => {
                KeyValuePair(self.pgt_log_level.name, markup! { <Dim>"unset"</Dim> }).fmt(fmt)?;
            }
            Some(value) => {
                KeyValuePair(self.pgt_log_level.name, markup! {{DebugDisplay(value)}}).fmt(fmt)?;
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
