use std::sync::Arc;

use dashmap::DashMap;
use pgt_query_ext::diagnostics::*;

use super::statement_identifier::StatementId;

pub struct PgQueryStore {
    db: DashMap<StatementId, Arc<Result<pgt_query_ext::NodeEnum, SyntaxDiagnostic>>>,
}

impl PgQueryStore {
    pub fn new() -> PgQueryStore {
        PgQueryStore { db: DashMap::new() }
    }

    pub fn load_parse(
        &self,
        statement: &StatementId,
        content: &str,
    ) -> Arc<Result<pgt_query_ext::NodeEnum, SyntaxDiagnostic>> {
        if let Some(existing) = self.db.get(statement).map(|x| x.clone()) {
            return existing;
        }

        let r = Arc::new(pgt_query_ext::parse(content).map_err(SyntaxDiagnostic::from));
        self.db.insert(statement.clone(), r.clone());
        r
    }

    pub fn clear_statement(&self, id: &StatementId) {
        self.db.remove(id);

        if let Some(child_id) = id.as_child() {
            self.db.remove(&child_id);
        }
    }
}
