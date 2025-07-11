use std::sync::{Arc, Mutex};

use dashmap::DashMap;

use super::statement_identifier::StatementId;

pub struct TreeSitterStore {
    db: DashMap<StatementId, Arc<tree_sitter::Tree>>,
    parser: Mutex<tree_sitter::Parser>,
}

impl TreeSitterStore {
    pub fn new() -> TreeSitterStore {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(tree_sitter_sql::language())
            .expect("Error loading sql language");

        TreeSitterStore {
            db: DashMap::new(),
            parser: Mutex::new(parser),
        }
    }

    pub fn get_or_cache_tree(&self, statement: &StatementId) -> Arc<tree_sitter::Tree> {
        if let Some(existing) = self.db.get(statement).map(|x| x.clone()) {
            return existing;
        }

        let mut parser = self.parser.lock().expect("Failed to lock parser");
        let tree = Arc::new(parser.parse(statement.content(), None).unwrap());
        self.db.insert(statement.clone(), tree.clone());

        tree
    }
}
