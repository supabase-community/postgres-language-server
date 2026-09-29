//! Session state of a file: search path, transactions, locks, and timeouts.

/// Session state as seen by a statement.
#[derive(Debug, Clone, Default)]
pub struct Session {
    search_path: Vec<String>,
}

impl Session {
    pub fn new(search_path: Vec<String>) -> Self {
        Self { search_path }
    }

    /// The explicit search path, without the implicit `pg_catalog` and `pg_temp`.
    pub fn search_path(&self) -> &[String] {
        &self.search_path
    }

    /// Applies the effects of `stmt` on the session.
    pub fn apply(&mut self, stmt: &pgls_query::NodeEnum) {
        let _ = stmt;
    }
}

pub fn is_vacuum_full(_stmt: &pgls_query::protobuf::VacuumStmt) -> bool {
    false
}

pub fn is_reindex_concurrent(_stmt: &pgls_query::protobuf::ReindexStmt) -> bool {
    false
}
