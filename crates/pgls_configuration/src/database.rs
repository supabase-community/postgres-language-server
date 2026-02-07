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
    #[partial(bpaf(env("DATABASE_URL"), long("connection-string")))]
    pub connection_string: Option<String>,

    /// The host of the database.
    /// Required if you want database-related features.
    /// All else falls back to sensible defaults.
    #[partial(bpaf(env("PGHOST"), long("host")))]
    pub host: String,

    /// The port of the database.
    #[partial(bpaf(env("PGPORT"), long("port")))]
    pub port: u16,

    /// The username to connect to the database.
    #[partial(bpaf(env("PGUSER"), long("username")))]
    pub username: String,

    /// The password to connect to the database.
    #[partial(bpaf(env("PGPASSWORD"), long("password")))]
    pub password: String,

    /// The name of the database.
    #[partial(bpaf(env("PGDATABASE"), long("database")))]
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    const ALL_VARS: &[&str] = &[
        "DATABASE_URL",
        "PGHOST",
        "PGPORT",
        "PGUSER",
        "PGPASSWORD",
        "PGDATABASE",
    ];

    fn clear_env_vars() {
        for var in ALL_VARS {
            unsafe {
                std::env::remove_var(var);
            }
        }
    }

    #[test]
    fn from_env_none_when_no_vars_set() {
        let _lock = ENV_MUTEX.lock().unwrap();
        clear_env_vars();

        assert!(PartialDatabaseConfiguration::from_env().is_none());
    }

    #[test]
    fn from_env_all_vars_set() {
        let _lock = ENV_MUTEX.lock().unwrap();
        clear_env_vars();

        unsafe {
            std::env::set_var("DATABASE_URL", "postgres://u:p@h:1234/d");
            std::env::set_var("PGHOST", "myhost");
            std::env::set_var("PGPORT", "5433");
            std::env::set_var("PGUSER", "myuser");
            std::env::set_var("PGPASSWORD", "mypass");
            std::env::set_var("PGDATABASE", "mydb");
        }

        let config = PartialDatabaseConfiguration::from_env().unwrap();
        assert_eq!(
            config.connection_string,
            Some("postgres://u:p@h:1234/d".to_string())
        );
        assert_eq!(config.host, Some("myhost".to_string()));
        assert_eq!(config.port, Some(5433));
        assert_eq!(config.username, Some("myuser".to_string()));
        assert_eq!(config.password, Some("mypass".to_string()));
        assert_eq!(config.database, Some("mydb".to_string()));

        clear_env_vars();
    }

    #[test]
    fn from_env_partial_vars() {
        let _lock = ENV_MUTEX.lock().unwrap();
        clear_env_vars();

        unsafe {
            std::env::set_var("PGHOST", "remotehost");
            std::env::set_var("PGDATABASE", "appdb");
        }

        let config = PartialDatabaseConfiguration::from_env().unwrap();
        assert_eq!(config.connection_string, None);
        assert_eq!(config.host, Some("remotehost".to_string()));
        assert_eq!(config.port, None);
        assert_eq!(config.username, None);
        assert_eq!(config.password, None);
        assert_eq!(config.database, Some("appdb".to_string()));

        clear_env_vars();
    }

    #[test]
    fn from_env_invalid_pgport_ignored() {
        let _lock = ENV_MUTEX.lock().unwrap();
        clear_env_vars();

        unsafe {
            std::env::set_var("PGHOST", "localhost");
            std::env::set_var("PGPORT", "not_a_number");
        }

        let config = PartialDatabaseConfiguration::from_env().unwrap();
        assert_eq!(config.host, Some("localhost".to_string()));
        assert_eq!(config.port, None);

        clear_env_vars();
    }

    #[test]
    fn from_env_only_invalid_pgport_returns_none() {
        let _lock = ENV_MUTEX.lock().unwrap();
        clear_env_vars();

        unsafe {
            std::env::set_var("PGPORT", "not_a_number");
        }

        // PGPORT is set but invalid — parse fails so it becomes None.
        // No other vars are set, so has_any is false.
        assert!(PartialDatabaseConfiguration::from_env().is_none());

        clear_env_vars();
    }
}
