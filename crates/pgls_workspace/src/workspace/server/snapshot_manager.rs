use std::sync::{Arc, RwLock};

#[cfg(feature = "db")]
use std::sync::Mutex;

use pgls_catalog::Snapshot;

use crate::WorkspaceError;

#[cfg(feature = "db")]
use sqlx::PgPool;
#[cfg(feature = "db")]
use std::collections::HashMap;

#[cfg(feature = "db")]
use super::{async_helper::run_async, connection_key::ConnectionKey};

/// The snapshot of a connection, once it loaded.
#[cfg(feature = "db")]
type SnapshotSlot = Arc<Mutex<Option<Arc<Snapshot>>>>;

/// Manages database snapshot storage and retrieval.
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
/// - `clear_all_connections()` - Clear the schemas of all connections
pub struct SnapshotManager {
    /// Connection-based snapshots (db mode only). Each connection has its own lock, held while
    /// its snapshot loads, so that a slow or unreachable database doesn't block the others.
    #[cfg(feature = "db")]
    db_schemas: RwLock<HashMap<ConnectionKey, SnapshotSlot>>,

    /// JSON-loaded schema (available in both modes)
    schema: RwLock<Option<Arc<Snapshot>>>,
}

impl Default for SnapshotManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SnapshotManager {
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
        let schema: Snapshot = serde_json::from_str(json)
            .map_err(|e| WorkspaceError::runtime(&format!("Invalid schema JSON: {e}")))?;
        *self.schema.write().unwrap() = Some(Arc::new(schema));
        Ok(())
    }

    /// Get the current schema if available.
    pub fn get(&self) -> Option<Arc<Snapshot>> {
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
    pub fn load(&self, pool: &PgPool) -> Result<Arc<Snapshot>, WorkspaceError> {
        let key: ConnectionKey = pool.into();

        // The map is only locked to find the connection's slot, never during network I/O.
        let existing = self.db_schemas.read().unwrap().get(&key).cloned();
        let slot = match existing {
            Some(slot) => slot,
            None => Arc::clone(self.db_schemas.write().unwrap().entry(key).or_default()),
        };

        // Callers for the same connection wait for one load instead of starting their own.
        let mut snapshot = slot.lock().unwrap();
        if let Some(cache) = snapshot.as_ref() {
            return Ok(Arc::clone(cache));
        }

        // Load the snapshot from the database. A failure leaves the slot empty for a retry.
        let pool_clone = pool.clone();
        let loaded = Arc::new(run_async(
            async move { Snapshot::load(&pool_clone).await },
        )??);
        *snapshot = Some(Arc::clone(&loaded));
        Ok(loaded)
    }

    /// Clear all connection-based snapshots.
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
