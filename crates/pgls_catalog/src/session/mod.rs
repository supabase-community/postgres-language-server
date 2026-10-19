//! Session state tracked while analysing a file: the settings that name resolution depends on,
//! the transaction depth, and the lock and timeout state that the safety rules check.

mod locks;
mod settings;
#[cfg(test)]
mod tests;

use pgls_query::{NodeEnum, protobuf};

use locks::Locks;
pub use locks::{is_reindex_concurrent, is_vacuum_full};
use settings::Settings;

/// Whether the migration tool runs the statements outside `BEGIN` and `COMMIT` in a
/// transaction.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TransactionMode {
    /// Not known. Migration tools usually run each migration in a transaction.
    #[default]
    Unknown,
    /// The migration tool runs the statements in a transaction.
    Transaction,
    /// Each statement outside `BEGIN` and `COMMIT` runs in a transaction of its own.
    Autocommit,
}

/// The state of the session as we analyse the statements of a file.
#[derive(Debug, Default, Clone)]
pub struct Session {
    /// `search_path` and `role`.
    settings: Settings,
    /// Lock, timeout, and constraint state of the current transaction.
    locks: Locks,
    /// Transaction nesting depth (0 = not in explicit transaction)
    transaction_depth: usize,
    transaction_mode: TransactionMode,
}

impl Session {
    /// Create a session with its initial search path.
    pub fn new(search_path: Vec<String>) -> Self {
        Self {
            settings: Settings::new(search_path),
            ..Self::default()
        }
    }

    /// Sets whether the migration tool runs the statements in a transaction.
    pub fn with_transaction_mode(mut self, transaction_mode: TransactionMode) -> Self {
        self.transaction_mode = transaction_mode;
        self
    }

    /// Returns the current search path.
    pub fn search_path(&self) -> &[String] {
        self.settings.search_path()
    }
    /// Returns the current role, if one has been set.
    pub fn role(&self) -> Option<&str> {
        self.settings.role()
    }
    /// Returns whether `CREATE FUNCTION` validates function bodies (`check_function_bodies`).
    pub fn check_function_bodies(&self) -> bool {
        self.settings.check_function_bodies()
    }
    /// Returns the transaction nesting depth (0 = not in an explicit transaction).
    pub fn transaction_depth(&self) -> usize {
        self.transaction_depth
    }

    pub fn transaction_mode(&self) -> TransactionMode {
        self.transaction_mode
    }

    /// Whether the next statement runs in a transaction block, or `None` if that depends on the
    /// unknown migration tool.
    pub fn in_transaction(&self) -> Option<bool> {
        match self.transaction_mode {
            _ if self.transaction_depth > 0 => Some(true),
            TransactionMode::Unknown => None,
            TransactionMode::Transaction => Some(true),
            TransactionMode::Autocommit => Some(false),
        }
    }

    /// Returns true if a lock timeout has been set in this transaction
    pub fn has_lock_timeout(&self) -> bool {
        self.locks.lock_timeout_set
    }
    /// Returns true if a statement timeout has been set in this transaction
    pub fn has_statement_timeout(&self) -> bool {
        self.locks.statement_timeout_set
    }
    /// Returns true if an idle-in-transaction timeout has been set in this transaction
    pub fn has_idle_in_transaction_timeout(&self) -> bool {
        self.locks.idle_in_transaction_timeout_set
    }
    /// Returns true if a constraint with the given name on the given table was added with NOT VALID
    pub fn has_not_valid_constraint(&self, schema: &str, table: &str, name: &str) -> bool {
        self.locks
            .has_not_valid_constraint(schema, table, name, self.search_path())
    }
    /// Returns the tables currently holding ACCESS EXCLUSIVE locks
    pub fn access_exclusive_tables(&self) -> &[(String, String)] {
        &self.locks.access_exclusive_tables
    }
    /// Returns true if an object with the given schema and name was created in this transaction
    pub fn has_created_object(&self, schema: &str, name: &str) -> bool {
        self.locks
            .has_created_object(schema, name, self.search_path())
    }
    /// Returns true if the transaction is currently holding an ACCESS EXCLUSIVE lock
    pub fn is_holding_access_exclusive(&self) -> bool {
        self.locks.holding_access_exclusive
    }

    /// Returns true if the statement takes a dangerous lock on an existing object.
    ///
    /// Covers: AlterTableStmt, non-concurrent IndexStmt, DropStmt (table/index),
    /// TruncateStmt, VacuumStmt, non-concurrent ReindexStmt, RenameStmt (with relation),
    /// and non-concurrent RefreshMatViewStmt.
    pub fn is_dangerous_lock_stmt(&self, stmt: &NodeEnum) -> bool {
        self.locks.is_dangerous_lock_stmt(stmt, self.search_path())
    }

    /// Returns the existing table targeted by an ALTER TABLE command that takes ACCESS EXCLUSIVE.
    pub fn access_exclusive_table_for_alter(
        &self,
        stmt: &protobuf::AlterTableStmt,
    ) -> Option<(String, String)> {
        self.locks
            .access_exclusive_table_for_alter(stmt, self.search_path())
    }

    /// Update session state based on a statement.
    pub fn apply(&mut self, stmt: &NodeEnum) {
        match stmt {
            NodeEnum::TransactionStmt(stmt) => self.apply_transaction(stmt),
            NodeEnum::VariableSetStmt(stmt) => {
                self.locks.apply_variable_set(stmt);
                self.settings.apply_variable_set(stmt);
            }
            _ => self.locks.apply(stmt, self.settings.search_path()),
        }
        // Outside a transaction block, the statement commits on its own.
        if self.in_transaction() == Some(false) && !matches!(stmt, NodeEnum::TransactionStmt(_)) {
            self.settings.commit();
            self.locks.release();
        }
    }

    fn apply_transaction(&mut self, stmt: &protobuf::TransactionStmt) {
        use protobuf::TransactionStmtKind as Kind;
        match stmt.kind() {
            Kind::TransStmtBegin | Kind::TransStmtStart => {
                if self.transaction_depth == 0 {
                    self.settings.begin();
                }
                self.transaction_depth += 1;
            }
            Kind::TransStmtCommit => {
                self.transaction_depth = self.transaction_depth.saturating_sub(1);
                self.settings.commit();
                self.locks.reset();
            }
            Kind::TransStmtRollback => {
                self.transaction_depth = self.transaction_depth.saturating_sub(1);
                self.settings.rollback();
                self.locks.reset();
            }
            Kind::TransStmtSavepoint => self.transaction_depth += 1,
            Kind::TransStmtRelease | Kind::TransStmtRollbackTo => {
                self.transaction_depth = self.transaction_depth.saturating_sub(1)
            }
            _ => {}
        }
    }
}
