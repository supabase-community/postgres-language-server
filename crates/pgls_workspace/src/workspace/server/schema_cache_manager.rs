use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use pgls_schema_cache::SchemaCache;
use sqlx::PgPool;

use crate::WorkspaceError;

use super::{async_helper::run_async, connection_key::ConnectionKey};

#[derive(Default)]
pub struct SchemaCacheManager {
    schemas: RwLock<HashMap<ConnectionKey, Arc<SchemaCache>>>,
}

impl SchemaCacheManager {
    pub fn new() -> Self {
        Self {
            schemas: RwLock::new(HashMap::new()),
        }
    }

    pub fn load(&self, pool: PgPool) -> Result<Arc<SchemaCache>, WorkspaceError> {
        let key: ConnectionKey = (&pool).into();
        // Try read lock first for cache hit
        if let Ok(schemas) = self.schemas.read() {
            if let Some(cache) = schemas.get(&key) {
                return Ok(Arc::clone(cache));
            }
        }

        // Cache miss - need write lock to insert
        let mut schemas = self.schemas.write().unwrap();

        // Double-check after acquiring write lock
        if let Some(cache) = schemas.get(&key) {
            return Ok(Arc::clone(cache));
        }

        // Load schema cache
        let pool_clone = pool.clone();
        let schema_cache = Arc::new(run_async(
            async move { SchemaCache::load(&pool_clone).await },
        )??);

        schemas.insert(key, schema_cache.clone());
        Ok(schema_cache)
    }
}
