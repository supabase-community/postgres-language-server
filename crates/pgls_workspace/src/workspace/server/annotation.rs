use std::sync::{Arc, Mutex};

use pgls_lexer::SyntaxKind;
use rustc_hash::FxHashMap;

use super::statement_identifier::StatementId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatementAnnotations {
    ends_with_semicolon: bool,
}

pub struct AnnotationStore {
    db: Mutex<FxHashMap<StatementId, Arc<StatementAnnotations>>>,
}

const WHITESPACE_TOKENS: [SyntaxKind; 6] = [
    SyntaxKind::SPACE,
    SyntaxKind::TAB,
    SyntaxKind::VERTICAL_TAB,
    SyntaxKind::FORM_FEED,
    SyntaxKind::LINE_ENDING,
    SyntaxKind::EOF,
];

impl AnnotationStore {
    pub fn new() -> AnnotationStore {
        AnnotationStore {
            db: Mutex::default(),
        }
    }

    /// Drops every cached annotation.
    pub fn clear(&self) {
        self.db.lock().unwrap().clear();
    }

    /// Drops the annotations of the statements for which `keep` returns `false`.
    pub fn retain(&self, keep: impl Fn(&StatementId) -> bool) {
        self.db.lock().unwrap().retain(|id, _| keep(id));
    }

    #[allow(unused)]
    pub fn get_annotations(
        &self,
        statement_id: &StatementId,
        content: &str,
    ) -> Arc<StatementAnnotations> {
        let mut cache = self.db.lock().unwrap();

        if let Some(existing) = cache.get(statement_id) {
            return existing.clone();
        }

        let lexed = pgls_lexer::lex(content);

        let ends_with_semicolon = (0..lexed.len())
            .rfind(|t| !WHITESPACE_TOKENS.contains(&lexed.kind(*t)))
            .map(|t| lexed.kind(t) == SyntaxKind::SEMICOLON)
            .unwrap_or(false);

        let annotations = Arc::new(StatementAnnotations {
            ends_with_semicolon,
        });

        cache.insert(statement_id.clone(), annotations.clone());

        annotations
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

        for (content, expected) in test_cases.iter() {
            let statement_id = StatementId::new(content);

            let annotations = store.get_annotations(&statement_id, content);

            assert_eq!(annotations.ends_with_semicolon, *expected);
        }
    }
}
