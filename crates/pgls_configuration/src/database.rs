use crate::StringSet;
use bpaf::Bpaf;
use pgls_configuration_macros::{Merge, Partial};
use serde::{Deserialize, Serialize};

/// The configuration of the database connection.
#[derive(Clone, Deserialize, Eq, Partial, PartialEq, Serialize)]
#[partial(derive(Bpaf, Clone, Eq, PartialEq, Merge))]
#[partial(cfg_attr(feature = "schema", derive(schemars::JsonSchema)))]
#[partial(serde(rename_all = "camelCase", default, deny_unknown_fields))]
pub struct DatabaseConfiguration {
    /// A connection string that encodes the full connection setup.
    /// When provided, it takes precedence over the individual fields.
    /// Can also be set via the `DATABASE_URL` environment variable.
    #[partial(bpaf(long("connection-string")))]
    pub connection_string: Option<String>,

    /// The host of the database.
    /// Required if you want database-related features.
    /// All else falls back to sensible defaults.
    /// Can also be set via the `PGHOST` environment variable.
    #[partial(bpaf(long("host")))]
    pub host: String,

    /// The port of the database.
    /// Can also be set via the `PGPORT` environment variable.
    #[partial(bpaf(long("port")))]
    pub port: u16,

    /// The username to connect to the database.
    /// Can also be set via the `PGUSER` environment variable.
    #[partial(bpaf(long("username")))]
    pub username: String,

    /// The password to connect to the database.
    /// Can also be set via the `PGPASSWORD` environment variable.
    #[partial(bpaf(long("password")))]
    pub password: String,

    /// The name of the database.
    /// Can also be set via the `PGDATABASE` environment variable.
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

impl std::fmt::Debug for DatabaseConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DatabaseConfiguration")
            .field(
                "connection_string",
                &self.connection_string.as_ref().map(|_| "[redacted]"),
            )
            .field("host", &self.host)
            .field("port", &self.port)
            .field("username", &self.username)
            .field("password", &"[redacted]")
            .field("database", &self.database)
            .field(
                "allow_statement_executions_against",
                &self.allow_statement_executions_against,
            )
            .field("conn_timeout_secs", &self.conn_timeout_secs)
            .field("disable_connection", &self.disable_connection)
            .finish()
    }
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

impl PartialDatabaseConfiguration {
    /// Creates a partial configuration from standard Postgres environment variables.
    ///
    /// Reads `DATABASE_URL`, `PGHOST`, `PGPORT`, `PGUSER`, `PGPASSWORD`, and `PGDATABASE`.
    /// Returns `None` if no relevant env vars are set.
    pub fn from_env() -> Option<Self> {
        let database_url = std::env::var("DATABASE_URL").ok();
        let pghost = std::env::var("PGHOST").ok();
        let pgport = std::env::var("PGPORT").ok().and_then(|p| p.parse().ok());
        let pguser = std::env::var("PGUSER").ok();
        let pgpassword = std::env::var("PGPASSWORD").ok();
        let pgdatabase = std::env::var("PGDATABASE").ok();

        let has_any = database_url.is_some()
            || pghost.is_some()
            || pgport.is_some()
            || pguser.is_some()
            || pgpassword.is_some()
            || pgdatabase.is_some();

        if !has_any {
            return None;
        }

        Some(Self {
            connection_string: database_url,
            host: pghost,
            port: pgport,
            username: pguser,
            password: pgpassword,
            database: pgdatabase,
            ..Default::default()
        })
    }
}
