use std::sync::{Arc, RwLock};

use pgls_schema_cache::SchemaCache;

use crate::WorkspaceError;

#[cfg(feature = "db")]
use sqlx::PgPool;
#[cfg(feature = "db")]
use std::collections::HashMap;

#[cfg(feature = "db")]
use super::{async_helper::run_async, connection_key::ConnectionKey};

/// Manages schema cache storage and retrieval.
///
/// In db mode: supports loading from database connections and/or JSON.
/// In no-db mode: only supports loading from JSON.
///
/// Common API (both modes):
/// - `set()` - Set schema from JSON string
/// - `get()` - Get the current schema
/// - `clear()` - Clear the current schema
///
/// DB-only API:
/// - `load()` - Load schema from database connection
/// - `clear_connection()` - Clear schema for specific connection
pub struct SchemaCacheManager {
    /// Connection-based schema caches (db mode only)
    #[cfg(feature = "db")]
    db_schemas: RwLock<HashMap<ConnectionKey, Arc<SchemaCache>>>,

    /// JSON-loaded schema (available in both modes)
    schema: RwLock<Option<Arc<SchemaCache>>>,
}

impl Default for SchemaCacheManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SchemaCacheManager {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "db")]
            db_schemas: RwLock::new(HashMap::new()),
            schema: RwLock::new(None),
        }
    }

    // ==================== Common API (both modes) ====================

    /// Set schema from JSON string.
    pub fn set(&self, json: &str) -> Result<(), WorkspaceError> {
        let schema: SchemaCache = serde_json::from_str(json)
            .map_err(|e| WorkspaceError::runtime(&format!("Invalid schema JSON: {e}")))?;
        *self.schema.write().unwrap() = Some(Arc::new(schema));
        Ok(())
    }

    /// Get the current schema if available.
    pub fn get(&self) -> Option<Arc<SchemaCache>> {
        self.schema.read().unwrap().clone()
    }

    /// Clear the current schema.
    pub fn clear(&self) {
        *self.schema.write().unwrap() = None;
    }

    // ==================== DB-only API ====================

    /// Load schema from a database connection.
    /// Returns cached schema if available, otherwise loads from database.
    #[cfg(feature = "db")]
    pub fn load(&self, pool: PgPool) -> Result<Arc<SchemaCache>, WorkspaceError> {
        let key: ConnectionKey = (&pool).into();

        // Try read lock first for cache hit
        if let Ok(schemas) = self.db_schemas.read() {
            if let Some(cache) = schemas.get(&key) {
                return Ok(Arc::clone(cache));
            }
        }

        // Cache miss - need write lock to insert
        let mut schemas = self.db_schemas.write().unwrap();

        // Double-check after acquiring write lock
        if let Some(cache) = schemas.get(&key) {
            return Ok(Arc::clone(cache));
        }

        // Load schema cache from database
        let pool_clone = pool.clone();
        let schema_cache = Arc::new(run_async(
            async move { SchemaCache::load(&pool_clone).await },
        )??);

        schemas.insert(key, schema_cache.clone());
        Ok(schema_cache)
    }

    /// Clear the schema cache for a specific connection.
    #[cfg(feature = "db")]
    pub fn clear_connection(&self, pool: &PgPool) {
        let key: ConnectionKey = pool.into();
        let mut schemas = self.db_schemas.write().unwrap();
        schemas.remove(&key);
    }

    /// Clear all connection-based schema caches.
    #[cfg(feature = "db")]
    pub fn clear_all_connections(&self) {
        let mut schemas = self.db_schemas.write().unwrap();
        schemas.clear();
    }

    /// Clear everything (both JSON schema and all db connections in db mode).
    pub fn clear_all(&self) {
        self.clear();
        #[cfg(feature = "db")]
        self.clear_all_connections();
    }
}
