//! A simple implementation of feature flags.

use pgt_console::fmt::{Display, Formatter};
use pgt_console::{DebugDisplay, KeyValuePair, markup};
use std::env;
use std::ops::Deref;
use std::sync::{LazyLock, OnceLock};

/// Returns `true` if this is an unstable build of Postgres Tools
pub fn is_unstable() -> bool {
    PGLS_VERSION.deref().is_none()
}

/// The internal version of Postgres Tools. This is usually supplied during the CI build
pub static PGLS_VERSION: LazyLock<Option<&str>> = LazyLock::new(|| option_env!("PGLS_VERSION"));

pub struct PgLSEnv {
    pub pgls_log_path: PgLSEnvVariable,
    pub pgt_log_path: PgLSEnvVariable,
    pub pgls_log_prefix: PgLSEnvVariable,
    pub pgt_log_prefix: PgLSEnvVariable,
    pub pgls_config_path: PgLSEnvVariable,
    pub pgt_config_path: PgLSEnvVariable,
}

pub static PGLS_ENV: OnceLock<PgLSEnv> = OnceLock::new();

impl PgLSEnv {
    fn new() -> Self {
        Self {
            pgls_log_path: PgLSEnvVariable::new(
                "PGLS_LOG_PATH",
                "The directory where the Daemon logs will be saved.",
            ),
            pgt_log_path: PgLSEnvVariable::new(
                "PGT_LOG_PATH",
                "The directory where the Daemon logs will be saved.",
            ),
            pgls_log_prefix: PgLSEnvVariable::new(
                "PGLS_LOG_PREFIX_NAME",
                "A prefix that's added to the name of the log. Default: `server.log.`",
            ),
            pgt_log_prefix: PgLSEnvVariable::new(
                "PGT_LOG_PREFIX_NAME",
                "A prefix that's added to the name of the log. Default: `server.log.`",
            ),
            pgls_config_path: PgLSEnvVariable::new(
                "PGLS_CONFIG_PATH",
                "A path to the configuration file",
            ),
            pgt_config_path: PgLSEnvVariable::new(
                "PGT_CONFIG_PATH",
                "A path to the configuration file",
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
    PGLS_ENV.get_or_init(PgLSEnv::new)
}

impl Display for PgLSEnv {
    fn fmt(&self, fmt: &mut Formatter) -> std::io::Result<()> {
        match self.pgt_log_path.value() {
            None => {
                KeyValuePair(self.pgt_log_path.name, markup! { <Dim>"unset"</Dim> }).fmt(fmt)?;
            }
            Some(value) => {
                KeyValuePair(self.pgt_log_path.name, markup! {{DebugDisplay(value)}}).fmt(fmt)?;
            }
        };
        match self.pgls_log_path.value() {
            None => {
                KeyValuePair(self.pgls_log_path.name, markup! { <Dim>"unset"</Dim> }).fmt(fmt)?;
            }
            Some(value) => {
                KeyValuePair(self.pgls_log_path.name, markup! {{DebugDisplay(value)}}).fmt(fmt)?;
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
        match self.pgls_log_prefix.value() {
            None => {
                KeyValuePair(self.pgls_log_prefix.name, markup! { <Dim>"unset"</Dim> }).fmt(fmt)?;
            }
            Some(value) => {
                KeyValuePair(self.pgls_log_prefix.name, markup! {{DebugDisplay(value)}})
                    .fmt(fmt)?;
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

        Ok(())
    }
}
