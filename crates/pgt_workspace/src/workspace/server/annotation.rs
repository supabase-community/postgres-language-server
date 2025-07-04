use std::sync::Arc;

use dashmap::DashMap;
use pgt_lexer_new::SyntaxKind;

use super::statement_identifier::StatementId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatementAnnotations {
    ends_with_semicolon: bool,
}

pub struct AnnotationStore {
    db: DashMap<StatementId, Arc<StatementAnnotations>>,
}

impl AnnotationStore {
    pub fn new() -> AnnotationStore {
        AnnotationStore { db: DashMap::new() }
    }

    #[allow(unused)]
    pub fn get_annotations(
        &self,
        statement_id: &StatementId,
        content: &str,
    ) -> Arc<StatementAnnotations> {
        if let Some(existing) = self.db.get(statement_id).map(|x| x.clone()) {
            return existing;
        }

        let lexed = pgt_lexer_new::lex(content);

        let mut ends_with_semicolon = false;

        // Iterate through tokens in reverse to find the last non-whitespace token
        for idx in (0..lexed.len()).rev() {
            let kind = lexed.kind(idx);
            if !matches!(
                kind,
                SyntaxKind::SPACE
                    | SyntaxKind::TAB
                    | SyntaxKind::VERTICAL_TAB
                    | SyntaxKind::FORM_FEED
                    | SyntaxKind::LINE_ENDING
                    | SyntaxKind::EOF
            ) {
                ends_with_semicolon = kind == SyntaxKind::SEMICOLON;
                break;
            }
        }

        let annotations = Arc::new(StatementAnnotations {
            ends_with_semicolon,
        });

        self.db.insert(statement_id.clone(), annotations.clone());

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

            assert_eq!(annotations.ends_with_semicolon, *expected);
        }
    }
}
