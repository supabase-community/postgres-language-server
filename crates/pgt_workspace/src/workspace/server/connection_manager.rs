use std::time::{Duration, Instant};

use dashmap::DashMap;
use sqlx::{pool::PoolOptions, postgres::PgConnectOptions, PgPool, Postgres};

use crate::settings::DatabaseSettings;

/// A unique identifier for database connection settings
#[derive(Clone, PartialEq, Eq, Hash)]
struct ConnectionKey {
    host: String,
    port: u16,
    username: String,
    password: String,
    database: String,
}

impl From<&DatabaseSettings> for ConnectionKey {
    fn from(settings: &DatabaseSettings) -> Self {
        Self {
            host: settings.host.clone(),
            port: settings.port,
            username: settings.username.clone(),
            password: settings.password.clone(),
            database: settings.database.clone(),
        }
    }
}

/// Cached connection pool with last access time
struct CachedPool {
    pool: PgPool,
    last_accessed: Instant,
    idle_timeout: Duration,
}

#[derive(Default)]
pub struct ConnectionManager {
    pools: DashMap<ConnectionKey, CachedPool>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            pools: DashMap::new(),
        }
    }

    /// Get a connection pool for the given database settings.
    /// If a pool already exists for these settings, it will be returned.
    /// If not, a new pool will be created if connections are enabled.
    pub(crate) fn get_pool(&self, settings: &DatabaseSettings) -> Option<PgPool> {
        let key = ConnectionKey::from(settings);

        // Cleanup idle connections first
        self.cleanup_idle_pools(&key);

        if !settings.enable_connection {
            tracing::info!("Database connection disabled.");
            return None;
        }

        // If we have a cached pool, update its last_accessed time and return it
        if let Some(mut cached_pool) = self.pools.get_mut(&key) {
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
            // TODO: add this to the db settings, for now default to one minute
            idle_timeout: Duration::from_secs(60),
        };

        self.pools.insert(key, cached_pool);

        Some(pool)
    }

    /// Remove pools that haven't been accessed for longer than the idle timeout
    fn cleanup_idle_pools(&self, ignore_key: &ConnectionKey) {
        let now = Instant::now();

        // Use retain to keep only non-idle connections
        self.pools.retain(|key, cached_pool| {
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
