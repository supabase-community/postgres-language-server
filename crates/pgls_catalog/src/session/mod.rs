//! Session state tracked while analysing a file: the settings that name resolution depends on,
//! the transaction depth, and the lock and timeout state that the safety rules check.

mod lock_level;
mod locks;
mod settings;
#[cfg(test)]
mod tests;
mod transaction;

use pgls_query::{NodeEnum, protobuf};

use crate::Catalog;

pub use lock_level::{LockMode, TableLock, alter_table_cmd_lock, alter_table_lock, relation_locks};
use locks::Locks;
use settings::Settings;
pub use transaction::{NonTransactional, ends_transaction_block, prevents_transaction_block};

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
    /// Tables (schema, name) that an `ALTER TABLE` changed, also in earlier transactions.
    altered_tables: Vec<(String, String)>,
    /// Whether a `BEGIN` opened a transaction block that hasn't ended yet. Postgres doesn't nest
    /// transaction blocks: a `BEGIN` in one only warns, and savepoints don't end it.
    explicit_block: bool,
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
    /// Whether a `BEGIN` opened a transaction block that hasn't ended yet.
    pub fn in_explicit_transaction(&self) -> bool {
        self.explicit_block
    }

    /// Whether the statement opens a transaction block that a later statement has to end: a
    /// `BEGIN` outside of one, or `COMMIT AND CHAIN` or `ROLLBACK AND CHAIN` in one.
    pub fn opens_transaction_block(&self, stmt: &NodeEnum) -> bool {
        use protobuf::TransactionStmtKind as Kind;
        let NodeEnum::TransactionStmt(stmt) = stmt else {
            return false;
        };
        match stmt.kind() {
            Kind::TransStmtBegin | Kind::TransStmtStart => !self.explicit_block,
            Kind::TransStmtCommit | Kind::TransStmtRollback => stmt.chain && self.explicit_block,
            _ => false,
        }
    }

    pub fn transaction_mode(&self) -> TransactionMode {
        self.transaction_mode
    }

    /// Whether the next statement runs in a transaction block, or `None` if that depends on the
    /// unknown migration tool.
    pub fn in_transaction(&self) -> Option<bool> {
        match self.transaction_mode {
            _ if self.explicit_block || self.implicit_block => Some(true),
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
    /// Returns true if an earlier `ALTER TABLE` changed the table. `catalog` resolves its name,
    /// like in [`Session::has_created_relation`].
    pub fn has_altered_table(&self, relation: &protobuf::RangeVar, catalog: &Catalog) -> bool {
        let (schema, name) = catalog.relation_name_of(relation, self.search_path());
        self.altered_tables
            .iter()
            .any(|(s, n)| *s == schema && *n == name)
    }
    /// Returns true if the transaction is currently holding an ACCESS EXCLUSIVE lock
    pub fn is_holding_access_exclusive(&self) -> bool {
        self.locks.holding_access_exclusive
    }

    /// Returns true if the statement takes a dangerous lock on an existing object.
    ///
    /// Covers: AlterTableStmt and IndexStmt with a lock that blocks writes (see
    /// [`Session::table_locks`]), DropStmt (table/index), TruncateStmt, VacuumStmt,
    /// non-concurrent ReindexStmt, RenameStmt (with relation), and non-concurrent
    /// RefreshMatViewStmt.
    pub fn is_dangerous_lock_stmt(&self, stmt: &NodeEnum, catalog: &Catalog) -> bool {
        self.locks
            .is_dangerous_lock_stmt(stmt, catalog, self.search_path())
    }

    /// The locks that an `ALTER TABLE` or `CREATE INDEX` takes on tables that this transaction
    /// didn't create, see [`relation_locks`]. A table created in the transaction is invisible to
    /// other sessions, so its locks block no one.
    pub fn table_locks(&self, stmt: &NodeEnum, catalog: &Catalog) -> Vec<TableLock> {
        self.locks.table_locks(stmt, catalog, self.search_path())
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
            NodeEnum::AlterTableStmt(alter) => {
                if let Some(relation) = &alter.relation {
                    self.altered_tables
                        .push(catalog.relation_name_of(relation, self.search_path()));
                }
                self.locks.apply(stmt, catalog, self.settings.search_path());
            }
            _ => self.locks.apply(stmt, catalog, self.settings.search_path()),
        }
        // Outside a transaction block, the statement commits on its own.
        if self.in_transaction() == Some(false) && !matches!(stmt, NodeEnum::TransactionStmt(_)) {
            self.settings.commit();
            self.locks.release();
        }
    }

    /// Port of what `BEGIN`, `COMMIT`, `ROLLBACK` and `PREPARE TRANSACTION` do to the transaction
    /// block, see [`BeginTransactionBlock`] and [`ends_transaction_block`].
    ///
    /// [`BeginTransactionBlock`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/access/transam/xact.c#L3924
    fn apply_transaction(&mut self, stmt: &protobuf::TransactionStmt) {
        use protobuf::TransactionStmtKind as Kind;
        match stmt.kind() {
            // In a transaction block, `BEGIN` only warns.
            Kind::TransStmtBegin | Kind::TransStmtStart => {
                if !self.explicit_block {
                    self.settings.begin();
                    self.explicit_block = true;
                }
            }
            // A prepared transaction keeps its settings like a committed one, see
            // [`PrepareTransaction`](https://github.com/postgres/postgres/blob/REL_18_6/src/backend/access/transam/xact.c#L2760).
            Kind::TransStmtCommit | Kind::TransStmtPrepare => {
                self.settings.commit();
                self.locks.reset();
                self.end_block(stmt.chain);
            }
            Kind::TransStmtRollback => {
                self.settings.rollback();
                self.locks.reset();
                self.end_block(stmt.chain);
            }
            // Savepoints don't open or end the block, and Postgres rejects them outside of one.
            _ => {}
        }
    }

    /// Ends the transaction block. `AND CHAIN` opens the next one right away; outside of a
    /// transaction block, Postgres rejects it.
    fn end_block(&mut self, chain: bool) {
        if chain && self.explicit_block {
            self.settings.begin();
        } else {
            self.explicit_block = false;
        }
    }
}
