use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use sqlx::{PgPool, Postgres, pool::PoolOptions, postgres::PgConnectOptions};

use crate::settings::DatabaseSettings;

use super::connection_key::ConnectionKey;

/// Cached connection pool with last access time
struct CachedPool {
    pool: PgPool,
    last_accessed: Instant,
    idle_timeout: Duration,
}

#[derive(Default)]
pub struct ConnectionManager {
    pools: RwLock<HashMap<ConnectionKey, CachedPool>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            pools: RwLock::new(HashMap::new()),
        }
    }

    /// Get a connection pool for the given database settings.
    /// If a pool already exists for these settings, it will be returned.
    /// If not, a new pool will be created if connections are enabled.
    /// Will also clean up idle connections that haven't been accessed for a while.
    pub(crate) fn get_pool(&self, settings: &DatabaseSettings) -> Option<PgPool> {
        let key = ConnectionKey::from(settings);

        // Cleanup idle connections first
        self.cleanup_idle_pools(&key);

        if !settings.enable_connection {
            tracing::info!("Database connection disabled.");
            return None;
        }

        // Try read lock first for cache hit
        if let Ok(pools) = self.pools.read() {
            if let Some(cached_pool) = pools.get(&key) {
                // Can't update last_accessed with read lock, but that's okay for occasional misses
                return Some(cached_pool.pool.clone());
            }
        }

        // Cache miss or need to update timestamp - use write lock
        let mut pools = self.pools.write().unwrap();

        // Double-check after acquiring write lock
        if let Some(cached_pool) = pools.get_mut(&key) {
            cached_pool.last_accessed = Instant::now();
            return Some(cached_pool.pool.clone());
        }

        // Create a new pool
        let config = PgConnectOptions::new()
            .host(&settings.host)
            .port(settings.port)
            .username(&settings.username)
            .password(&settings.password)
            .database(&settings.database);

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

    /// Remove pools that haven't been accessed for longer than the idle timeout
    fn cleanup_idle_pools(&self, ignore_key: &ConnectionKey) {
        let now = Instant::now();

        let mut pools = self.pools.write().unwrap();

        // Use retain to keep only non-idle connections
        pools.retain(|key, cached_pool| {
            let idle_duration = now.duration_since(cached_pool.last_accessed);
            if idle_duration > cached_pool.idle_timeout && key != ignore_key {
                tracing::debug!(
                    "Removing idle database connection (idle for {:?})",
                    idle_duration
                );
                false
            } else {
                true
            }
        });
    }
}
