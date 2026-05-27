use std::collections::HashMap;
use std::str::FromStr;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use sqlx::{PgPool, Postgres, pool::PoolOptions, postgres::PgConnectOptions};

use crate::{WorkspaceError, settings::DatabaseSettings};

use super::connection_key::ConnectionKey;

const INITIAL_FAILURE_BACKOFF: Duration = Duration::from_secs(5);
const MAX_FAILURE_BACKOFF: Duration = Duration::from_secs(60);

/// Cached connection pool with last access time
struct CachedPool {
    pool: PgPool,
    last_accessed: Instant,
    idle_timeout: Duration,
}

struct CachedFailure {
    message: String,
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

        if self.connection_is_in_backoff(&key) {
            return None;
        }

        {
            if let Ok(pools) = self.pools.read()
                && let Some(cached_pool) = pools.get(&key)
            {
                return Some(cached_pool.pool.clone());
            }
        }

        let mut pools = self.pools.write().unwrap();

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
        let result = operation(&pool);
        self.record_result(&pool, &result);
        Some(result)
    }

    fn record_result<T>(&self, pool: &PgPool, result: &Result<T, WorkspaceError>) {
        match result {
            Ok(_) => self.clear_failure(&pool.into()),
            Err(err @ WorkspaceError::DatabaseConnectionError(_)) => {
                self.record_failure(pool, &err.to_string());
            }
            Err(_) => {}
        }
    }

    fn record_failure(&self, pool: &PgPool, error: &str) {
        let key: ConnectionKey = pool.into();
        let mut failures = self.failures.write().unwrap();
        let now = Instant::now();
        let attempts = failures.get(&key).map_or(1, |failure| failure.attempts + 1);
        let multiplier = 1u32
            .checked_shl(attempts.saturating_sub(1))
            .unwrap_or(u32::MAX);
        let backoff = INITIAL_FAILURE_BACKOFF
            .saturating_mul(multiplier)
            .min(MAX_FAILURE_BACKOFF);

        let was_cached = failures.contains_key(&key);
        failures.insert(
            key,
            CachedFailure {
                message: error.to_string(),
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

    fn connection_is_in_backoff(&self, key: &ConnectionKey) -> bool {
        let failures = self.failures.read().unwrap();
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
