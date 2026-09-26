use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::str::FromStr;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use sqlx::{PgPool, Postgres, pool::PoolOptions, postgres::PgConnectOptions};

use crate::{WorkspaceError, settings::DatabaseSettings};

use super::connection_key::ConnectionKey;

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

/// Cached connection pool with last access time
struct CachedPool {
    pool: PgPool,
    settings_fingerprint: u64,
    last_accessed: Instant,
    idle_timeout: Duration,
}

struct CachedFailure {
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
            match PgConnectOptions::from_str(uri) {
                Ok(options) => options,
                Err(err) => {
                    tracing::error!("Failed to parse database connection URI: {err}");
                    return None;
                }
            }
        } else {
            PgConnectOptions::new()
                .host(&settings.host)
                .port(settings.port)
                .username(&settings.username)
                .password(&settings.password)
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
            Err(err @ WorkspaceError::DatabaseConnectionError(_)) => {
                self.record_failure(key, settings_fingerprint, &err.to_string());
            }
            Err(_) => {}
        }
    }

    fn record_failure(&self, key: &ConnectionKey, settings_fingerprint: u64, error: &str) {
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
                message: error.to_string(),
                settings_fingerprint,
                attempts,
                next_retry_at: now + backoff,
            },
        );

        if was_cached {
            tracing::debug!(
                "Database connection failed again. Retrying after {:?}: {error}",
                backoff
            );
        } else {
            tracing::warn!(
                "Database connection failed. Skipping database-backed features for {:?}: {error}",
                backoff
            );
        }
    }

    fn connection_is_in_backoff(&self, key: &ConnectionKey, settings_fingerprint: u64) -> bool {
        let mut failures = self.failures.write().unwrap();
        if failures
            .get(key)
            .is_some_and(|failure| failure.settings_fingerprint != settings_fingerprint)
        {
            failures.remove(key);
            return false;
        }
        let Some(failure) = failures.get(key) else {
            return false;
        };

        let now = Instant::now();
        if now < failure.next_retry_at {
            tracing::debug!(
                "Skipping database connection retry during backoff until {:?}: {}",
                failure.next_retry_at,
                failure.message
            );
            return true;
        }

        false
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

        settings.password = "changed-password".to_string();
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
                message: "authentication failed".to_string(),
                settings_fingerprint: old_fingerprint,
                attempts: 1,
                next_retry_at: Instant::now() + Duration::from_secs(60),
            },
        );

        settings.password = "corrected-password".to_string();
        let new_fingerprint = settings_fingerprint(&settings);
        assert!(!manager.connection_is_in_backoff(&key, new_fingerprint));
        assert!(!manager.failures.read().unwrap().contains_key(&key));
        assert!(manager.get_pool(&settings).is_some());
    }
}
