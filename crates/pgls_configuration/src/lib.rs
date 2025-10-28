//! This module contains the configuration of `postgres-language-server.jsonc`
//!
//! The configuration is divided by "tool".

pub mod analyser;
pub mod database;
pub mod diagnostics;
pub mod files;
pub mod generated;
pub mod migrations;
pub mod plpgsql_check;
pub mod typecheck;
pub mod vcs;

pub use crate::diagnostics::ConfigurationDiagnostic;

use std::path::PathBuf;

pub use crate::generated::push_to_analyser_rules;
use crate::vcs::{PartialVcsConfiguration, VcsConfiguration, partial_vcs_configuration};
pub use analyser::{
    LinterConfiguration, PartialLinterConfiguration, RuleConfiguration, RuleFixConfiguration,
    RulePlainConfiguration, RuleSelector, RuleWithFixOptions, RuleWithOptions, Rules,
    partial_linter_configuration,
};
use biome_deserialize::StringSet;
use biome_deserialize_macros::{Merge, Partial};
use bpaf::Bpaf;
use database::{
    DatabaseConfiguration, PartialDatabaseConfiguration, partial_database_configuration,
};
use files::{FilesConfiguration, PartialFilesConfiguration, partial_files_configuration};
use migrations::{
    MigrationsConfiguration, PartialMigrationsConfiguration, partial_migrations_configuration,
};
use pgls_env::PGLS_WEBSITE;
use plpgsql_check::{
    PartialPlPgSqlCheckConfiguration, PlPgSqlCheckConfiguration,
    partial_pl_pg_sql_check_configuration,
};
use serde::{Deserialize, Serialize};
pub use typecheck::{
    PartialTypecheckConfiguration, TypecheckConfiguration, partial_typecheck_configuration,
};
use vcs::VcsClientKind;

pub use pgls_env::VERSION;

/// The configuration that is contained inside the configuration file.
#[derive(Clone, Debug, Default, Deserialize, Eq, Partial, PartialEq, Serialize)]
#[partial(derive(Bpaf, Clone, Eq, PartialEq, Merge))]
#[partial(cfg_attr(feature = "schema", derive(schemars::JsonSchema)))]
#[partial(serde(deny_unknown_fields, rename_all = "camelCase"))]
pub struct Configuration {
    /// A field for the [JSON schema](https://json-schema.org/) specification
    #[partial(serde(rename = "$schema"))]
    #[partial(bpaf(hide))]
    pub schema: String,

    /// A list of paths to other JSON files, used to extends the current configuration.
    #[partial(bpaf(hide))]
    pub extends: StringSet,

    /// The configuration of the VCS integration
    #[partial(type, bpaf(external(partial_vcs_configuration), optional, hide_usage))]
    pub vcs: VcsConfiguration,

    /// The configuration of the filesystem
    #[partial(
        type,
        bpaf(external(partial_files_configuration), optional, hide_usage)
    )]
    pub files: FilesConfiguration,

    /// Configure migrations
    #[partial(
        type,
        bpaf(external(partial_migrations_configuration), optional, hide_usage)
    )]
    pub migrations: MigrationsConfiguration,

    /// The configuration for the linter
    #[partial(type, bpaf(external(partial_linter_configuration), optional))]
    pub linter: LinterConfiguration,

    /// The configuration for type checking
    #[partial(type, bpaf(external(partial_typecheck_configuration), optional))]
    pub typecheck: TypecheckConfiguration,

    /// The configuration for type checking
    #[partial(type, bpaf(external(partial_pl_pg_sql_check_configuration), optional))]
    pub plpgsql_check: PlPgSqlCheckConfiguration,

    /// The configuration of the database connection
    #[partial(
        type,
        bpaf(external(partial_database_configuration), optional, hide_usage)
    )]
    pub db: DatabaseConfiguration,
}

impl PartialConfiguration {
    /// Returns the initial configuration.
    pub fn init() -> Self {
        Self {
            schema: Some(format!("{PGLS_WEBSITE}/schemas/{VERSION}/schema.json")),
            extends: Some(StringSet::default()),
            files: Some(PartialFilesConfiguration {
                ignore: Some(Default::default()),
                ..Default::default()
            }),
            migrations: None,
            vcs: Some(PartialVcsConfiguration {
                enabled: Some(false),
                client_kind: Some(VcsClientKind::Git),
                use_ignore_file: Some(false),
                ..Default::default()
            }),
            linter: Some(PartialLinterConfiguration {
                enabled: Some(true),
                rules: Some(Rules {
                    recommended: Some(true),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            typecheck: Some(PartialTypecheckConfiguration {
                ..Default::default()
            }),
            plpgsql_check: Some(PartialPlPgSqlCheckConfiguration {
                ..Default::default()
            }),
            db: Some(PartialDatabaseConfiguration {
                connection_string: None,
                host: Some("127.0.0.1".to_string()),
                port: Some(5432),
                username: Some("postgres".to_string()),
                password: Some("postgres".to_string()),
                database: Some("postgres".to_string()),
                allow_statement_executions_against: Default::default(),
                conn_timeout_secs: Some(10),
                disable_connection: Some(false),
            }),
        }
    }
}

pub struct ConfigurationPayload {
    /// The result of the deserialization
    pub deserialized: PartialConfiguration,
    /// The path of where the configuration file that was found. This contains the file name.
    pub configuration_file_path: PathBuf,
    /// The base path where the external configuration in a package should be resolved from
    pub external_resolution_base_path: PathBuf,
}

#[derive(Debug, Default, PartialEq, Clone)]
pub enum ConfigurationPathHint {
    /// The default mode, not having a configuration file is not an error.
    /// The path will be filled with the working directory if it is not filled at the time of usage.
    #[default]
    None,

    /// Very similar to [ConfigurationPathHint::None]. However, the path provided by this variant
    /// will be used as **working directory**, which means that all globs defined in the configuration
    /// will use **this path** as base path.
    FromWorkspace(PathBuf),

    /// The configuration path provided by the LSP, not having a configuration file is not an error.
    /// The path will always be a directory path.
    FromLsp(PathBuf),
    /// The configuration path provided by the user, not having a configuration file is an error.
    /// The path can either be a directory path or a file path.
    /// Throws any kind of I/O errors.
    FromUser(PathBuf),
}

impl ConfigurationPathHint {
    pub const fn is_from_user(&self) -> bool {
        matches!(self, Self::FromUser(_))
    }
    pub const fn is_from_lsp(&self) -> bool {
        matches!(self, Self::FromLsp(_))
    }
}
