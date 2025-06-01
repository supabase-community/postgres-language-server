use std::sync::Arc;

use dashmap::DashMap;
use pgt_schema_cache::SchemaCache;
use sqlx::PgPool;

use crate::WorkspaceError;

use super::{async_helper::run_async, connection_key::ConnectionKey};

#[derive(Default)]
pub struct SchemaCacheManager {
    schemas: DashMap<ConnectionKey, Arc<SchemaCache>>,
}

impl SchemaCacheManager {
    pub fn new() -> Self {
        Self {
            schemas: DashMap::new(),
        }
    }

    pub fn load(&self, pool: PgPool) -> Result<Arc<SchemaCache>, WorkspaceError> {
        let key: ConnectionKey = (&pool).into();

        if let Some(cache) = self.schemas.get(&key) {
            return Ok(Arc::clone(&*cache));
        }

        let schema_cache = self
            .schemas
            .entry(key)
            .or_try_insert_with::<WorkspaceError>(|| {
                // This closure will only be called once per key if multiple threads
                // try to access the same key simultaneously
                let pool_clone = pool.clone();
                let schema_cache =
                    Arc::new(run_async(
                        async move { SchemaCache::load(&pool_clone).await },
                    )??);

                Ok(schema_cache)
            })?;

        Ok(Arc::clone(&schema_cache))
    }
}
