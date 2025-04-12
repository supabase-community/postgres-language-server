use std::sync::Arc;

use dashmap::DashMap;
use pgt_lexer::{SyntaxKind, WHITESPACE_TOKENS};

use super::statement_identifier::StatementId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatementAnnotations {
    ends_with_semicolon: bool,
}

pub struct AnnotationStore {
    db: DashMap<StatementId, Option<Arc<StatementAnnotations>>>,
}

impl AnnotationStore {
    pub fn new() -> AnnotationStore {
        AnnotationStore { db: DashMap::new() }
    }

    #[allow(unused)]
    pub fn get_annotations(
        &self,
        statement: &StatementId,
        content: &str,
    ) -> Option<Arc<StatementAnnotations>> {
        if let Some(existing) = self.db.get(statement).map(|x| x.clone()) {
            return existing;
        }

        // we swallow the error here because the lexing within the document would have already
        // thrown and we wont even get here if that happened.
        let annotations = pgt_lexer::lex(content).ok().map(|tokens| {
            let ends_with_semicolon = tokens
                .iter()
                .rev()
                .find(|token| !WHITESPACE_TOKENS.contains(&token.kind))
                .is_some_and(|token| token.kind == SyntaxKind::Ascii59);

            Arc::new(StatementAnnotations {
                ends_with_semicolon,
            })
        });

        self.db.insert(statement.clone(), None);
        annotations
    }

    pub fn clear_statement(&self, id: &StatementId) {
        self.db.remove(id);

        if let Some(child_id) = id.get_child_id() {
            self.db.remove(&child_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::workspace::StatementId;

    use super::AnnotationStore;

    #[test]
    fn annotates_correctly() {
        let store = AnnotationStore::new();

        let test_cases = [
            ("SELECT * FROM foo", false),
            ("SELECT * FROM foo;", true),
            ("SELECT * FROM foo ;", true),
            ("SELECT * FROM foo ; ", true),
            ("SELECT * FROM foo ;\n", true),
            ("SELECT * FROM foo\n", false),
        ];

        for (idx, (content, expected)) in test_cases.iter().enumerate() {
            let statement_id = StatementId::Root(idx.into());

            let annotations = store.get_annotations(&statement_id, content);

            assert!(annotations.is_some());
            assert_eq!(annotations.unwrap().ends_with_semicolon, *expected);
        }
    }
}
