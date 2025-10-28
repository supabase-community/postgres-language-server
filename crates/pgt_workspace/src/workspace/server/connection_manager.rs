use std::collections::HashMap;
use std::str::FromStr;
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

        if !settings.enable_connection {
            tracing::info!("Database connection disabled.");
            return None;
        }

        {
            if let Ok(pools) = self.pools.read() {
                if let Some(cached_pool) = pools.get(&key) {
                    return Some(cached_pool.pool.clone());
                }
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
}
