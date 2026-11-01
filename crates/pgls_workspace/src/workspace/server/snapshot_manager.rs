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
use super::{async_helper::run_async, connection_key::ConnectionKey, last_used::LastUsed};
#[cfg(feature = "db")]
use std::time::Duration;

/// How long the snapshot of a connection may stay unused before it is dropped. A snapshot of a
/// large database takes megabytes (see the snapshot memory test), and one that nobody looks at
/// for that long can be loaded again, which takes well under a second for a local database.
#[cfg(feature = "db")]
const SNAPSHOT_IDLE_TIMEOUT: Duration = Duration::from_secs(30 * 60);

/// How often [`SnapshotManager::load`] looks for idle snapshots at most.
#[cfg(feature = "db")]
const IDLE_SWEEP_INTERVAL: Duration = Duration::from_secs(60);

/// The snapshot of a connection, once it loaded.
#[cfg(feature = "db")]
#[derive(Default)]
struct SnapshotSlot {
    snapshot: Mutex<Option<Arc<Snapshot>>>,
    last_used: LastUsed,
}

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
    db_schemas: RwLock<HashMap<ConnectionKey, Arc<SnapshotSlot>>>,

    /// When the snapshots were last checked for idle ones.
    #[cfg(feature = "db")]
    last_idle_sweep: LastUsed,

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
            #[cfg(feature = "db")]
            last_idle_sweep: LastUsed::now(),
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

        slot.last_used.touch();
        self.drop_idle_snapshots();

        // Callers for the same connection wait for one load instead of starting their own.
        let mut snapshot = slot.snapshot.lock().unwrap();
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

    /// Drops the snapshots of the connections for which `is_used` is false, e.g. because no
    /// configuration points at them any more.
    #[cfg(feature = "db")]
    pub(crate) fn retain(&self, is_used: impl Fn(&ConnectionKey) -> bool) {
        self.db_schemas.write().unwrap().retain(|key, _| {
            let used = is_used(key);
            if !used {
                tracing::debug!(
                    "Dropping the snapshot of {}:{}/{}, which no configuration uses",
                    key.host,
                    key.port,
                    key.database
                );
            }
            used
        });
    }

    /// The number of connections with a snapshot slot.
    #[cfg(all(test, feature = "db"))]
    pub(crate) fn connection_count(&self) -> usize {
        self.db_schemas.read().unwrap().len()
    }

    /// Drops the snapshots that weren't used for [`SNAPSHOT_IDLE_TIMEOUT`], looking at most once
    /// per [`IDLE_SWEEP_INTERVAL`]. They are loaded again on the next use.
    #[cfg(feature = "db")]
    fn drop_idle_snapshots(&self) {
        if self.last_idle_sweep.idle_for() < IDLE_SWEEP_INTERVAL {
            return;
        }
        self.last_idle_sweep.touch();
        let is_idle = |slot: &SnapshotSlot| slot.last_used.idle_for() > SNAPSHOT_IDLE_TIMEOUT;
        if !self
            .db_schemas
            .read()
            .unwrap()
            .values()
            .any(|slot| is_idle(slot))
        {
            return;
        }
        self.db_schemas.write().unwrap().retain(|key, slot| {
            let idle = is_idle(slot);
            if idle {
                tracing::debug!(
                    "Dropping the snapshot of {}:{}/{}, unused for {:?}",
                    key.host,
                    key.port,
                    key.database,
                    slot.last_used.idle_for()
                );
            }
            !idle
        });
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

#[cfg(all(test, feature = "db"))]
mod tests {
    use super::*;

    fn key(database: &str) -> ConnectionKey {
        ConnectionKey {
            host: "127.0.0.1".to_string(),
            port: 5432,
            username: "postgres".to_string(),
            database: database.to_string(),
        }
    }

    fn insert(manager: &SnapshotManager, database: &str) {
        let slot = SnapshotSlot::default();
        *slot.snapshot.lock().unwrap() = Some(Arc::new(Snapshot::default()));
        manager
            .db_schemas
            .write()
            .unwrap()
            .insert(key(database), Arc::new(slot));
    }

    fn databases(manager: &SnapshotManager) -> Vec<String> {
        let mut databases: Vec<_> = manager
            .db_schemas
            .read()
            .unwrap()
            .keys()
            .map(|key| key.database.clone())
            .collect();
        databases.sort();
        databases
    }

    #[test]
    fn unused_snapshots_are_dropped() {
        let manager = SnapshotManager::new();
        insert(&manager, "one");
        insert(&manager, "two");

        manager.retain(|key| key.database == "two");
        assert_eq!(databases(&manager), ["two"]);
    }

    #[test]
    fn idle_snapshots_are_dropped() {
        let manager = SnapshotManager::new();
        insert(&manager, "idle");
        insert(&manager, "busy");
        manager.db_schemas.read().unwrap()[&key("idle")]
            .last_used
            .backdate(SNAPSHOT_IDLE_TIMEOUT * 2);

        // The sweep runs at most once per interval.
        manager.drop_idle_snapshots();
        assert_eq!(databases(&manager), ["busy", "idle"]);

        manager.last_idle_sweep.backdate(IDLE_SWEEP_INTERVAL * 2);
        manager.drop_idle_snapshots();
        assert_eq!(databases(&manager), ["busy"]);
    }
}
