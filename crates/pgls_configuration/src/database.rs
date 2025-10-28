use biome_deserialize::StringSet;
use biome_deserialize_macros::{Merge, Partial};
use bpaf::Bpaf;
use serde::{Deserialize, Serialize};

/// The configuration of the database connection.
#[derive(Clone, Debug, Deserialize, Eq, Partial, PartialEq, Serialize)]
#[partial(derive(Bpaf, Clone, Eq, PartialEq, Merge))]
#[partial(cfg_attr(feature = "schema", derive(schemars::JsonSchema)))]
#[partial(serde(rename_all = "camelCase", default, deny_unknown_fields))]
pub struct DatabaseConfiguration {
    /// A connection string that encodes the full connection setup.
    /// When provided, it takes precedence over the individual fields.
    #[partial(bpaf(long("connection-string")))]
    pub connection_string: Option<String>,

    /// The host of the database.
    /// Required if you want database-related features.
    /// All else falls back to sensible defaults.
    #[partial(bpaf(long("host")))]
    pub host: String,

    /// The port of the database.
    #[partial(bpaf(long("port")))]
    pub port: u16,

    /// The username to connect to the database.
    #[partial(bpaf(long("username")))]
    pub username: String,

    /// The password to connect to the database.
    #[partial(bpaf(long("password")))]
    pub password: String,

    /// The name of the database.
    #[partial(bpaf(long("database")))]
    pub database: String,

    #[partial(bpaf(long("allow_statement_executions_against")))]
    pub allow_statement_executions_against: StringSet,

    /// The connection timeout in seconds.
    #[partial(bpaf(long("conn_timeout_secs"), fallback(Some(10)), debug_fallback))]
    pub conn_timeout_secs: u16,

    /// Actively disable all database-related features.
    #[partial(bpaf(long("disable-db"), switch, fallback(Some(false))))]
    #[partial(cfg_attr(feature = "schema", schemars(skip)))]
    pub disable_connection: bool,
}

impl Default for DatabaseConfiguration {
    fn default() -> Self {
        Self {
            connection_string: None,
            disable_connection: false,
            host: "127.0.0.1".to_string(),
            port: 5432,
            username: "postgres".to_string(),
            password: "postgres".to_string(),
            database: "postgres".to_string(),
            allow_statement_executions_against: Default::default(),
            conn_timeout_secs: 10,
        }
    }
}
