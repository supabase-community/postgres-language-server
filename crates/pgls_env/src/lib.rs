//! Environment variables and configuration constants for Postgres Language Server.
//!
//! This module provides:
//! - The environment variables that Postgres Language Server reads, in [`VARIABLES`]
//! - Static constants like version information and website URL
//! - Helper functions for checking build status

use std::env;

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

/// An environment variable that Postgres Language Server reads.
#[derive(Debug)]
pub struct EnvVariable {
    /// The name of the environment variable
    name: &'static str,
    /// The description of the variable, for the documentation.
    description: &'static str,
    /// The name from before the rename to Postgres Language Server, which is still read when the
    /// variable isn't set.
    deprecated_alias: Option<&'static str>,
}

impl EnvVariable {
    /// The value of the variable, or else of its deprecated alias.
    pub fn value(&self) -> Option<String> {
        env::var(self.name)
            .ok()
            .or_else(|| self.deprecated_alias.and_then(|alias| env::var(alias).ok()))
    }

    /// It returns the description of the variable
    pub fn description(&self) -> &'static str {
        self.description
    }

    /// It returns the name of the variable.
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// The name from before the rename to Postgres Language Server, if the variable had one.
    pub fn deprecated_alias(&self) -> Option<&'static str> {
        self.deprecated_alias
    }
}

pub static PGLS_LOG_PATH: EnvVariable = EnvVariable {
    name: "PGLS_LOG_PATH",
    description: "The directory where the Daemon logs will be saved.",
    deprecated_alias: Some("PGT_LOG_PATH"),
};

pub static PGLS_LOG_LEVEL: EnvVariable = EnvVariable {
    name: "PGLS_LOG_LEVEL",
    description: "Allows to change the log level: `none`, `trace`, `debug`, `info`, `warn` or `error`. Default is debug for the language server and none for the other commands. This will only affect \"pgls*\" crates. All others are logged with info level.",
    deprecated_alias: Some("PGT_LOG_LEVEL"),
};

pub static PGLS_LOG_KIND: EnvVariable = EnvVariable {
    name: "PGLS_LOG_KIND",
    description: "The format of the logs of the language server: `hierarchical` or `bunyan`. Default: `hierarchical`.",
    deprecated_alias: Some("PGT_LOG_KIND"),
};

pub static PGLS_LOG_PREFIX_NAME: EnvVariable = EnvVariable {
    name: "PGLS_LOG_PREFIX_NAME",
    description: "A prefix that's added to the name of the log. Default: `server.log.`",
    deprecated_alias: Some("PGT_LOG_PREFIX_NAME"),
};

pub static PGLS_CONFIG_PATH: EnvVariable = EnvVariable {
    name: "PGLS_CONFIG_PATH",
    description: "A path to the configuration file, or to a folder with one, for the language server (`lsp-proxy`, `start`). `check` and `format` take `--config-path` instead.",
    deprecated_alias: Some("PGT_CONFIG_PATH"),
};

pub static DATABASE_URL: EnvVariable = EnvVariable {
    name: "DATABASE_URL",
    description: "A connection string that encodes the full database connection setup.",
    deprecated_alias: None,
};

pub static PGHOST: EnvVariable = EnvVariable {
    name: "PGHOST",
    description: "The host of the database server.",
    deprecated_alias: None,
};

pub static PGPORT: EnvVariable = EnvVariable {
    name: "PGPORT",
    description: "The port of the database server.",
    deprecated_alias: None,
};

pub static PGUSER: EnvVariable = EnvVariable {
    name: "PGUSER",
    description: "The username to connect to the database.",
    deprecated_alias: None,
};

pub static PGPASSWORD: EnvVariable = EnvVariable {
    name: "PGPASSWORD",
    description: "The password to connect to the database.",
    deprecated_alias: None,
};

pub static PGPASSFILE: EnvVariable = EnvVariable {
    name: "PGPASSFILE",
    description: "The password file to look up the password in when none is configured. Default: `~/.pgpass`.",
    deprecated_alias: None,
};

pub static PGDATABASE: EnvVariable = EnvVariable {
    name: "PGDATABASE",
    description: "The name of the database to connect to.",
    deprecated_alias: None,
};

/// Every environment variable that Postgres Language Server reads, in the order of the
/// documentation.
pub static VARIABLES: &[&EnvVariable] = &[
    &PGLS_LOG_PATH,
    &PGLS_LOG_LEVEL,
    &PGLS_LOG_KIND,
    &PGLS_LOG_PREFIX_NAME,
    &PGLS_CONFIG_PATH,
    &DATABASE_URL,
    &PGHOST,
    &PGPORT,
    &PGUSER,
    &PGPASSWORD,
    &PGPASSFILE,
    &PGDATABASE,
];
