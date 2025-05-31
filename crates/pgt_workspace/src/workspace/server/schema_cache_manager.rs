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

        let schema_cache = Arc::new(run_async(async move { SchemaCache::load(&pool).await })??);

        self.schemas.insert(key, Arc::clone(&schema_cache));

        Ok(schema_cache)
    }
}
