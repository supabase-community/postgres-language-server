use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::str::FromStr;
use std::sync::RwLock;
use std::time::Duration;

use sqlx::{PgPool, Postgres, pool::PoolOptions, postgres::PgConnectOptions};

use crate::{
    WorkspaceError,
    diagnostics::DatabaseConnectionError,
    settings::{DEFAULT_DATABASE_PASSWORD, DatabaseSettings},
};

use super::backoff::Backoff;
use super::connection_key::ConnectionKey;
use super::last_used::LastUsed;
use super::pgpass::{PassfileTarget, password_from_passfile};

/// How long a pool, and each of its connections, may stay unused before it is closed. An open
/// connection holds a backend process on the server, and a project that isn't edited for a while
/// shouldn't keep one. Reconnecting takes a few milliseconds on the next use.
const POOL_IDLE_TIMEOUT: Duration = Duration::from_secs(5 * 60);

/// How often [`ConnectionManager::get_pools`] looks for idle pools at most.
const IDLE_SWEEP_INTERVAL: Duration = Duration::from_secs(60);

/// Fingerprints the connection parameters that [`ConnectionKey`] does not cover, so that a change
/// to one of them invalidates the cached pool instead of being silently ignored. Hashing keeps the
/// credentials out of the cache entries, and therefore out of anything that prints them.
fn settings_fingerprint(settings: &DatabaseSettings) -> u64 {
    let mut hasher = DefaultHasher::new();
    settings.connection_string.hash(&mut hasher);
    settings.password.hash(&mut hasher);
    settings.conn_timeout_secs.hash(&mut hasher);
    hasher.finish()
}

/// The configured password, else the one from the password file, else the default.
fn password(settings: &DatabaseSettings) -> String {
    settings
        .password
        .clone()
        .or_else(|| {
            password_from_passfile(&PassfileTarget {
                host: &settings.host,
                port: settings.port,
                database: &settings.database,
                username: &settings.username,
            })
        })
        .unwrap_or_else(|| DEFAULT_DATABASE_PASSWORD.to_string())
}

/// sqlx looks up the password file for a connection string too, but unlike libpq it does not
/// unescape the password and falls back to `~/.pgpass` when `PGPASSFILE` has no entry. Prefer
/// the libpq lookup when the connection string and `PGPASSWORD` have no password.
fn connection_string_options(uri: &str) -> Result<PgConnectOptions, sqlx::Error> {
    let options = PgConnectOptions::from_str(uri)?;
    let url = url::Url::parse(uri).map_err(|err| sqlx::Error::Configuration(err.into()))?;
    let has_password = url.password().is_some()
        || url.query_pairs().any(|(key, _)| key == "password")
        || std::env::var_os("PGPASSWORD").is_some_and(|password| !password.is_empty());
    if has_password {
        return Ok(options);
    }

    let password = password_from_passfile(&PassfileTarget {
        host: options.get_host(),
        port: options.get_port(),
        // Postgres connects to the database named like the user by default.
        database: options.get_database().unwrap_or(options.get_username()),
        username: options.get_username(),
    });
    Ok(match password {
        Some(password) => options.password(&password),
        None => options,
    })
}

/// The connection pools of one database.
#[derive(Clone)]
pub(crate) struct DatabasePools {
    /// The pool for the language server's own queries.
    pub(crate) pool: PgPool,
    /// The pool for the statements a user executes. Its connections never run the language
    /// server's own queries, so the session state that a statement leaves behind can't leak into
    /// them.
    pub(crate) statement_pool: PgPool,
}

impl DatabasePools {
    /// Closes both pools: they open no new connections, and their idle connections close once
    /// the last clone, e.g. of a statement execution that is still running, is dropped.
    fn close(&self) {
        // `close` marks a pool as closed before its future is polled; the future would only wait
        // for the connections that are in use.
        drop(self.pool.close());
        drop(self.statement_pool.close());
    }
}

/// Cached connection pool with last access time
struct CachedPool {
    pools: DatabasePools,
    settings_fingerprint: u64,
    last_used: LastUsed,
}

struct CachedFailure {
    error: DatabaseConnectionError,
    message: String,
    settings_fingerprint: u64,
    backoff: Backoff,
}

pub struct ConnectionManager {
    pools: RwLock<HashMap<ConnectionKey, CachedPool>>,
    failures: RwLock<HashMap<ConnectionKey, CachedFailure>>,
    /// When the pools were last checked for idle ones.
    last_idle_sweep: LastUsed,
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            pools: RwLock::new(HashMap::new()),
            failures: RwLock::new(HashMap::new()),
            last_idle_sweep: LastUsed::now(),
        }
    }

    /// Closes and drops the pools and drops the failures of the connections for which `is_used`
    /// is false, e.g. because no configuration points at them any more. That includes the pool
    /// for executed statements.
    pub(crate) fn retain(&self, is_used: impl Fn(&ConnectionKey) -> bool) {
        self.pools.write().unwrap().retain(|key, cached| {
            let used = is_used(key);
            if !used {
                cached.pools.close();
                tracing::debug!(
                    "Closing the pool of {}:{}/{}, which no configuration uses",
                    key.host,
                    key.port,
                    key.database
                );
            }
            used
        });
        self.failures.write().unwrap().retain(|key, _| is_used(key));
    }

    /// The number of cached pools.
    #[cfg(test)]
    pub(crate) fn pool_count(&self) -> usize {
        self.pools.read().unwrap().len()
    }

    /// The cached pools of every database.
    #[cfg(test)]
    pub(crate) fn cached_pools(&self) -> Vec<DatabasePools> {
        self.pools
            .read()
            .unwrap()
            .values()
            .map(|cached| cached.pools.clone())
            .collect()
    }

    /// Closes the pools that weren't used for [`POOL_IDLE_TIMEOUT`], looking at most once per
    /// [`IDLE_SWEEP_INTERVAL`].
    fn close_idle_pools(&self) {
        if self.last_idle_sweep.idle_for() < IDLE_SWEEP_INTERVAL {
            return;
        }
        self.last_idle_sweep.touch();
        let has_idle = self
            .pools
            .read()
            .unwrap()
            .values()
            .any(|pool| pool.last_used.idle_for() > POOL_IDLE_TIMEOUT);
        if !has_idle {
            return;
        }
        self.pools.write().unwrap().retain(|key, pool| {
            let idle_for = pool.last_used.idle_for();
            let idle = idle_for > POOL_IDLE_TIMEOUT;
            if idle {
                pool.pools.close();
                tracing::debug!(
                    "Closing the pool of {}:{}/{}, unused for {idle_for:?}",
                    key.host,
                    key.port,
                    key.database
                );
            }
            !idle
        });
    }

    #[cfg(test)]
    pub(crate) fn get_pool(&self, settings: &DatabaseSettings) -> Option<PgPool> {
        self.get_pools(settings).map(|pools| pools.pool)
    }

    /// Get the connection pools for the given database settings.
    /// If pools already exist for these settings, they will be returned.
    /// If not, new pools will be created if connections are enabled.
    /// Will also clean up idle connections that haven't been accessed for a while.
    pub(crate) fn get_pools(&self, settings: &DatabaseSettings) -> Option<DatabasePools> {
        let key = ConnectionKey::from(settings);

        if !settings.enable_connection {
            tracing::info!("Database connection disabled.");
            return None;
        }

        let settings_fingerprint = settings_fingerprint(settings);
        if self.connection_is_in_backoff(&key, settings_fingerprint) {
            return None;
        }

        self.close_idle_pools();
        {
            if let Ok(pools) = self.pools.read()
                && let Some(cached_pool) = pools.get(&key)
                && cached_pool.settings_fingerprint == settings_fingerprint
            {
                cached_pool.last_used.touch();
                return Some(cached_pool.pools.clone());
            }
        }

        let mut pools = self.pools.write().unwrap();

        // Drop a pool created with stale connection parameters.
        if pools
            .get(&key)
            .is_some_and(|pool| pool.settings_fingerprint != settings_fingerprint)
        {
            pools.remove(&key);
        }

        // Double-check after acquiring write lock
        if let Some(cached_pool) = pools.get(&key) {
            cached_pool.last_used.touch();
            return Some(cached_pool.pools.clone());
        }

        // Create a new pool
        let config = if let Some(uri) = settings.connection_string.as_ref() {
            match connection_string_options(uri) {
                Ok(options) => options,
                Err(err) => {
                    tracing::error!("Failed to parse database connection URI: {err}");
                    return None;
                }
            }
        } else {
            PgConnectOptions::new_without_pgpass()
                .host(&settings.host)
                .port(settings.port)
                .username(&settings.username)
                .password(&password(settings))
                .database(&settings.database)
        };

        let timeout = settings.conn_timeout_secs;

        let pool_options = || {
            PoolOptions::<Postgres>::new()
                .acquire_timeout(timeout)
                .acquire_slow_threshold(Duration::from_secs(2))
                .idle_timeout(POOL_IDLE_TIMEOUT)
        };
        let database_pools = DatabasePools {
            pool: pool_options().connect_lazy_with(config.clone()),
            // Without cached prepared statements, `discard all` can reset the whole session after
            // a statement.
            statement_pool: pool_options().connect_lazy_with(config.statement_cache_capacity(0)),
        };

        let cached_pool = CachedPool {
            pools: database_pools.clone(),
            settings_fingerprint,
            last_used: LastUsed::now(),
        };

        pools.insert(key, cached_pool);

        Some(database_pools)
    }

    /// Like [`Self::with_pool`], but returns the error of a connection that failed recently
    /// instead of skipping it during the backoff.
    pub(crate) fn try_with_pool<T>(
        &self,
        settings: &DatabaseSettings,
        operation: impl FnOnce(&PgPool) -> Result<T, WorkspaceError>,
    ) -> Option<Result<T, WorkspaceError>> {
        if settings.enable_connection
            && let Some(error) = self.failure_in_backoff(
                &ConnectionKey::from(settings),
                settings_fingerprint(settings),
            )
        {
            return Some(Err(WorkspaceError::DatabaseConnectionError(error)));
        }

        self.with_pool(settings, operation)
    }

    pub(crate) fn with_pool<T>(
        &self,
        settings: &DatabaseSettings,
        operation: impl FnOnce(&PgPool) -> Result<T, WorkspaceError>,
    ) -> Option<Result<T, WorkspaceError>> {
        self.with_pools(settings, |pools| operation(&pools.pool))
    }

    /// Like [`Self::with_pool`], but passes the pool for executed statements as well.
    pub(crate) fn with_pools<T>(
        &self,
        settings: &DatabaseSettings,
        operation: impl FnOnce(&DatabasePools) -> Result<T, WorkspaceError>,
    ) -> Option<Result<T, WorkspaceError>> {
        let pools = self.get_pools(settings)?;
        let key = ConnectionKey::from(settings);
        let result = operation(&pools);
        self.record_result(&key, settings_fingerprint(settings), &result);
        Some(result)
    }

    fn record_result<T>(
        &self,
        key: &ConnectionKey,
        settings_fingerprint: u64,
        result: &Result<T, WorkspaceError>,
    ) {
        match result {
            Ok(_) => self.clear_failure(key),
            Err(err @ WorkspaceError::DatabaseConnectionError(error))
                if error.connection_failure =>
            {
                self.record_failure(key, settings_fingerprint, error, &err.to_string());
            }
            // A query that fails, e.g. on a division by zero, says nothing about the connection.
            Err(_) => {}
        }
    }

    fn record_failure(
        &self,
        key: &ConnectionKey,
        settings_fingerprint: u64,
        error: &DatabaseConnectionError,
        message: &str,
    ) {
        let mut failures = self.failures.write().unwrap();
        let backoff = Backoff::after_failure(failures.get(key).map(|failure| &failure.backoff));
        let delay = backoff.delay();

        let was_cached = failures.contains_key(key);
        failures.insert(
            key.clone(),
            CachedFailure {
                error: error.clone(),
                message: message.to_string(),
                settings_fingerprint,
                backoff,
            },
        );

        if was_cached {
            tracing::debug!(
                "Database connection failed again. Retrying after {delay:?}: {message}"
            );
        } else {
            tracing::warn!(
                "Database connection failed. Skipping database-backed features for {delay:?}: {message}"
            );
        }
    }

    fn connection_is_in_backoff(&self, key: &ConnectionKey, settings_fingerprint: u64) -> bool {
        self.failure_in_backoff(key, settings_fingerprint).is_some()
    }

    /// The failure that holds back the next connection attempt, if any.
    fn failure_in_backoff(
        &self,
        key: &ConnectionKey,
        settings_fingerprint: u64,
    ) -> Option<DatabaseConnectionError> {
        let mut failures = self.failures.write().unwrap();
        if failures
            .get(key)
            .is_some_and(|failure| failure.settings_fingerprint != settings_fingerprint)
        {
            failures.remove(key);
            return None;
        }
        let failure = failures.get(key)?;

        if failure.backoff.is_active() {
            tracing::debug!(
                "Skipping database connection retry during backoff until {:?}: {}",
                failure.backoff.next_retry_at(),
                failure.message
            );
            return Some(failure.error.clone());
        }

        None
    }

    fn clear_failure(&self, key: &ConnectionKey) {
        self.failures.write().unwrap().remove(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings() -> DatabaseSettings {
        DatabaseSettings {
            enable_connection: true,
            ..DatabaseSettings::default()
        }
    }

    #[tokio::test]
    async fn same_settings_reuse_cached_pool() {
        let manager = ConnectionManager::new();
        let settings = settings();
        let first = manager.get_pool(&settings).unwrap();

        first.close().await;
        let second = manager.get_pool(&settings).unwrap();

        assert!(second.is_closed());
        assert_eq!(manager.pools.read().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn password_change_replaces_cached_pool_with_new_settings() {
        let manager = ConnectionManager::new();
        let mut settings = settings();
        let first_fingerprint = settings_fingerprint(&settings);
        let first = manager.get_pool(&settings).unwrap();
        first.close().await;

        settings.password = Some("changed-password".to_string());
        let changed_fingerprint = settings_fingerprint(&settings);
        let pool = manager.get_pool(&settings).unwrap();

        assert!(!pool.is_closed());
        let pools = manager.pools.read().unwrap();
        let cached = pools.get(&ConnectionKey::from(&settings)).unwrap();
        assert_ne!(first_fingerprint, changed_fingerprint);
        assert_eq!(cached.settings_fingerprint, changed_fingerprint);
        assert_eq!(
            cached.pools.pool.connect_options().get_host(),
            pool.connect_options().get_host()
        );
        assert_eq!(pools.len(), 1);
    }

    #[test]
    fn password_change_clears_failure_backoff() {
        let manager = ConnectionManager::new();
        let mut settings = settings();
        let key = ConnectionKey::from(&settings);
        let old_fingerprint = settings_fingerprint(&settings);
        manager.failures.write().unwrap().insert(
            key.clone(),
            CachedFailure {
                error: authentication_failed(),
                message: "authentication failed".to_string(),
                settings_fingerprint: old_fingerprint,
                backoff: Backoff::after_failure(None),
            },
        );

        settings.password = Some("corrected-password".to_string());
        let new_fingerprint = settings_fingerprint(&settings);
        assert!(!manager.connection_is_in_backoff(&key, new_fingerprint));
        assert!(!manager.failures.read().unwrap().contains_key(&key));
        assert!(manager.get_pool(&settings).is_some());
    }

    #[tokio::test]
    async fn unused_connections_are_dropped() {
        let manager = ConnectionManager::new();
        let used = settings();
        let unused = DatabaseSettings {
            database: "other".to_string(),
            ..settings()
        };
        manager.get_pool(&used).unwrap();
        let unused_pools = manager.get_pools(&unused).unwrap();
        manager.record_failure(
            &ConnectionKey::from(&unused),
            settings_fingerprint(&unused),
            &authentication_failed(),
            "authentication failed",
        );

        let used_key = ConnectionKey::from(&used);
        manager.retain(|key| key == &used_key);

        let pools = manager.pools.read().unwrap();
        assert_eq!(pools.keys().collect::<Vec<_>>(), [&used_key]);
        assert!(manager.failures.read().unwrap().is_empty());
        // Both pools of the database are closed, also the one for executed statements, even
        // though a clone of them is still alive.
        assert!(unused_pools.pool.is_closed());
        assert!(unused_pools.statement_pool.is_closed());
        let used_pools = &pools[&used_key].pools;
        assert!(!used_pools.pool.is_closed());
        assert!(!used_pools.statement_pool.is_closed());
    }

    #[tokio::test]
    async fn idle_pools_are_closed() {
        let manager = ConnectionManager::new();
        let idle = DatabaseSettings {
            database: "idle".to_string(),
            ..settings()
        };
        let busy = settings();
        let idle_pools = manager.get_pools(&idle).unwrap();
        manager.get_pool(&busy).unwrap();
        manager.pools.read().unwrap()[&ConnectionKey::from(&idle)]
            .last_used
            .backdate(POOL_IDLE_TIMEOUT * 2);

        // The sweep runs at most once per interval.
        manager.get_pool(&busy).unwrap();
        assert_eq!(manager.pools.read().unwrap().len(), 2);

        manager.last_idle_sweep.backdate(IDLE_SWEEP_INTERVAL * 2);
        manager.get_pool(&busy).unwrap();
        let pools = manager.pools.read().unwrap();
        assert_eq!(
            pools.keys().collect::<Vec<_>>(),
            [&ConnectionKey::from(&busy)]
        );
        assert!(idle_pools.pool.is_closed());
        assert!(idle_pools.statement_pool.is_closed());
    }

    fn authentication_failed() -> DatabaseConnectionError {
        DatabaseConnectionError {
            message: "password authentication failed".to_string(),
            code: Some("28P01".to_string()),
            connection_failure: true,
        }
    }

    #[test]
    fn try_with_pool_reports_the_failure_during_backoff() {
        let manager = ConnectionManager::new();
        let settings = settings();
        let key = ConnectionKey::from(&settings);
        manager.record_failure(
            &key,
            settings_fingerprint(&settings),
            &authentication_failed(),
            "authentication failed",
        );

        assert!(manager.with_pool(&settings, |_| Ok(())).is_none());
        let result = manager.try_with_pool(&settings, |_| Ok(()));
        assert!(matches!(
            result,
            Some(Err(WorkspaceError::DatabaseConnectionError(error)))
                if error.message == "password authentication failed"
        ));
    }

    #[test]
    fn only_connection_failures_start_the_backoff() {
        let manager = ConnectionManager::new();
        let settings = settings();

        let query_error = manager.with_pool(&settings, |_| {
            Err::<(), _>(WorkspaceError::from(sqlx::Error::ColumnNotFound(
                "id".to_string(),
            )))
        });
        assert!(matches!(query_error, Some(Err(_))));
        assert!(manager.with_pool(&settings, |_| Ok(())).is_some());

        let connection_failure = manager.with_pool(&settings, |_| {
            Err::<(), _>(WorkspaceError::from(sqlx::Error::PoolTimedOut))
        });
        assert!(matches!(connection_failure, Some(Err(_))));
        assert!(manager.with_pool(&settings, |_| Ok(())).is_none());
    }
}
