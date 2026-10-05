use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::str::FromStr;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use sqlx::{PgPool, Postgres, pool::PoolOptions, postgres::PgConnectOptions};

use crate::{
    WorkspaceError,
    diagnostics::DatabaseConnectionError,
    settings::{DEFAULT_DATABASE_PASSWORD, DatabaseSettings},
};

use super::connection_key::ConnectionKey;
use super::pgpass::{PassfileTarget, password_from_passfile};

const INITIAL_FAILURE_BACKOFF: Duration = Duration::from_secs(5);
const MAX_FAILURE_BACKOFF: Duration = Duration::from_secs(60);

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

/// Cached connection pool with last access time
struct CachedPool {
    pool: PgPool,
    settings_fingerprint: u64,
    last_accessed: Instant,
    idle_timeout: Duration,
}

struct CachedFailure {
    error: DatabaseConnectionError,
    message: String,
    settings_fingerprint: u64,
    attempts: u32,
    next_retry_at: Instant,
}

#[derive(Default)]
pub struct ConnectionManager {
    pools: RwLock<HashMap<ConnectionKey, CachedPool>>,
    failures: RwLock<HashMap<ConnectionKey, CachedFailure>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            pools: RwLock::new(HashMap::new()),
            failures: RwLock::new(HashMap::new()),
        }
    }

    /// Get a connection pool for the given database settings.
    /// If a pool already exists for these settings, it will be returned.
    /// If not, a new pool will be created if connections are enabled.
    /// Will also clean up idle connections that haven't been accessed for a while.
    pub(crate) fn get_pool(&self, settings: &DatabaseSettings) -> Option<PgPool> {
        let key = ConnectionKey::from(settings);

        if !settings.enable_connection {
            tracing::info!("Database connection disabled.");
            return None;
        }

        let settings_fingerprint = settings_fingerprint(settings);
        if self.connection_is_in_backoff(&key, settings_fingerprint) {
            return None;
        }

        {
            if let Ok(pools) = self.pools.read()
                && let Some(cached_pool) = pools.get(&key)
                && cached_pool.settings_fingerprint == settings_fingerprint
            {
                return Some(cached_pool.pool.clone());
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
        if let Some(cached_pool) = pools.get_mut(&key) {
            cached_pool.last_accessed = Instant::now();
            return Some(cached_pool.pool.clone());
        }

        // Clean up idle connections before creating new ones to avoid unbounded growth
        let now = Instant::now();
        pools.retain(|k, cached_pool| {
            let idle_duration = now.duration_since(cached_pool.last_accessed);
            if idle_duration > cached_pool.idle_timeout && k != &key {
                tracing::debug!(
                    "Removing idle database connection (idle for {:?})",
                    idle_duration
                );
                false
            } else {
                true
            }
        });

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

        let pool = PoolOptions::<Postgres>::new()
            .acquire_timeout(timeout)
            .acquire_slow_threshold(Duration::from_secs(2))
            .connect_lazy_with(config);

        let cached_pool = CachedPool {
            pool: pool.clone(),
            settings_fingerprint,
            last_accessed: Instant::now(),
            // TODO: add this to the db settings, for now default to five minutes
            idle_timeout: Duration::from_secs(60 * 5),
        };

        pools.insert(key, cached_pool);

        Some(pool)
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
        let pool = self.get_pool(settings)?;
        let key = ConnectionKey::from(settings);
        let result = operation(&pool);
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
            Err(err @ WorkspaceError::DatabaseConnectionError(error)) => {
                self.record_failure(key, settings_fingerprint, error, &err.to_string());
            }
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
        let now = Instant::now();
        let attempts = failures.get(key).map_or(1, |failure| failure.attempts + 1);
        let multiplier = 1u32
            .checked_shl(attempts.saturating_sub(1))
            .unwrap_or(u32::MAX);
        let backoff = INITIAL_FAILURE_BACKOFF
            .saturating_mul(multiplier)
            .min(MAX_FAILURE_BACKOFF);

        let was_cached = failures.contains_key(key);
        failures.insert(
            key.clone(),
            CachedFailure {
                error: error.clone(),
                message: message.to_string(),
                settings_fingerprint,
                attempts,
                next_retry_at: now + backoff,
            },
        );

        if was_cached {
            tracing::debug!(
                "Database connection failed again. Retrying after {:?}: {message}",
                backoff
            );
        } else {
            tracing::warn!(
                "Database connection failed. Skipping database-backed features for {:?}: {message}",
                backoff
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

        let now = Instant::now();
        if now < failure.next_retry_at {
            tracing::debug!(
                "Skipping database connection retry during backoff until {:?}: {}",
                failure.next_retry_at,
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
            cached.pool.connect_options().get_host(),
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
                attempts: 1,
                next_retry_at: Instant::now() + Duration::from_secs(60),
            },
        );

        settings.password = Some("corrected-password".to_string());
        let new_fingerprint = settings_fingerprint(&settings);
        assert!(!manager.connection_is_in_backoff(&key, new_fingerprint));
        assert!(!manager.failures.read().unwrap().contains_key(&key));
        assert!(manager.get_pool(&settings).is_some());
    }

    fn authentication_failed() -> DatabaseConnectionError {
        DatabaseConnectionError {
            message: "password authentication failed".to_string(),
            code: Some("28P01".to_string()),
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
}
