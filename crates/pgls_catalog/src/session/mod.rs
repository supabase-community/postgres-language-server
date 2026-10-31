//! Session state tracked while analysing a file: the settings that name resolution depends on,
//! the transaction depth, and the lock and timeout state that the safety rules check.

mod locks;
mod settings;
#[cfg(test)]
mod tests;

use pgls_query::{NodeEnum, protobuf};

use crate::Catalog;

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
    /// Whether the statements run in an implicit transaction block, see
    /// [`Session::begin_implicit_block`].
    implicit_block: bool,
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
            _ if self.transaction_depth > 0 || self.implicit_block => Some(true),
            TransactionMode::Unknown => None,
            TransactionMode::Transaction => Some(true),
            TransactionMode::Autocommit => Some(false),
        }
    }

    /// Starts a query with more than one statement that the migration tool sends to Postgres at
    /// once, like a dbmate section with `transaction:false`. Postgres runs such a query in an
    /// implicit transaction block: until the query ends, every statement runs in a transaction
    /// block, also after a `COMMIT` in the query. Port of the implicit block in
    /// [`exec_simple_query`].
    ///
    /// [`exec_simple_query`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/tcop/postgres.c#L1090
    pub fn begin_implicit_block(&mut self) {
        self.implicit_block = true;
    }

    /// Whether the statements run in an implicit transaction block, see
    /// [`Session::begin_implicit_block`].
    pub fn in_implicit_block(&self) -> bool {
        self.implicit_block
    }

    /// Ends the query started with [`Session::begin_implicit_block`]. Outside of an explicit
    /// transaction, the implicit block commits.
    pub fn end_implicit_block(&mut self) {
        if !std::mem::take(&mut self.implicit_block) || self.in_transaction() != Some(false) {
            return;
        }
        self.settings.commit();
        self.locks.release();
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
    /// Returns true if a constraint with the given name on the given table was added with NOT VALID.
    /// `catalog` resolves the name of the table.
    pub fn has_not_valid_constraint(
        &self,
        table: &protobuf::RangeVar,
        name: &str,
        catalog: &Catalog,
    ) -> bool {
        self.locks
            .has_not_valid_constraint(table, name, catalog, self.search_path())
    }
    /// Returns the tables currently holding ACCESS EXCLUSIVE locks
    pub fn access_exclusive_tables(&self) -> &[(String, String)] {
        &self.locks.access_exclusive_tables
    }
    /// Returns true if the relation was created in this transaction. `catalog` resolves its
    /// name, like Postgres: an unqualified name is the first relation with that name on the
    /// search path.
    pub fn has_created_relation(&self, relation: &protobuf::RangeVar, catalog: &Catalog) -> bool {
        self.locks
            .has_created_relation(relation, catalog, self.search_path())
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
    pub fn is_dangerous_lock_stmt(&self, stmt: &NodeEnum, catalog: &Catalog) -> bool {
        self.locks
            .is_dangerous_lock_stmt(stmt, catalog, self.search_path())
    }

    /// Returns the existing table targeted by an ALTER TABLE command that takes ACCESS EXCLUSIVE.
    pub fn access_exclusive_table_for_alter(
        &self,
        stmt: &protobuf::AlterTableStmt,
        catalog: &Catalog,
    ) -> Option<(String, String)> {
        self.locks
            .access_exclusive_table_for_alter(stmt, catalog, self.search_path())
    }

    /// Update session state based on a statement. `catalog` is the catalog after the statement,
    /// see [`Catalog::apply`].
    pub fn apply(&mut self, stmt: &NodeEnum, catalog: &Catalog) {
        match stmt {
            NodeEnum::TransactionStmt(stmt) => self.apply_transaction(stmt),
            NodeEnum::VariableSetStmt(stmt) => {
                self.locks.apply_variable_set(stmt);
                self.settings.apply_variable_set(stmt);
            }
            _ => self.locks.apply(stmt, catalog, self.settings.search_path()),
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
