use pgls_schema_cache::SchemaCache;
use pgls_test_utils::QueryWithCursorPosition;
use pgls_text_size::TextRange;
use sqlx::{Executor, PgPool};

use crate::{CompletionItem, CompletionItemKind, CompletionParams, complete};

pub(crate) async fn get_test_deps(
    setup: Option<&str>,
    input: QueryWithCursorPosition,
    test_db: &PgPool,
) -> (tree_sitter::Tree, pgls_schema_cache::SchemaCache) {
    if let Some(setup) = setup {
        test_db
            .execute(setup)
            .await
            .expect("Failed to execute setup query");
    }

    let schema_cache = SchemaCache::load(test_db)
        .await
        .expect("Failed to load Schema Cache");

    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
        .expect("Error loading sql language");

    let tree = parser.parse(input.to_string(), None).unwrap();

    (tree, schema_cache)
}

/// Careful: This will connect against the passed database.
/// Use this only to debug issues. Do not commit to version control.
#[allow(dead_code)]
pub(crate) async fn test_against_connection_string(
    conn_str: &str,
    input: QueryWithCursorPosition,
) -> (tree_sitter::Tree, pgls_schema_cache::SchemaCache) {
    let pool = sqlx::PgPool::connect(conn_str)
        .await
        .expect("Unable to connect to database.");

    let schema_cache = SchemaCache::load(&pool)
        .await
        .expect("Failed to load Schema Cache");

    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
        .expect("Error loading sql language");

    let tree = parser.parse(input.to_string(), None).unwrap();

    (tree, schema_cache)
}

pub(crate) fn get_test_params<'a>(
    tree: &'a tree_sitter::Tree,
    schema_cache: &'a pgls_schema_cache::SchemaCache,
    sql: QueryWithCursorPosition,
) -> CompletionParams<'a> {
    let (position, text) = sql.get_text_and_position();

    CompletionParams {
        position: (position as u32).into(),
        schema: schema_cache,
        tree,
        text,
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum CompletionAssertion {
    Label(String),
    LabelAndKind(String, CompletionItemKind),
    LabelAndDesc(String, String),
    LabelNotExists(String),
    KindNotExists(CompletionItemKind),
    CompletionTextAndRange(String, TextRange),
}

impl CompletionAssertion {
    fn assert(&self, item: &CompletionItem) {
        match self {
            CompletionAssertion::Label(label) => {
                assert_eq!(
                    &item.label, label,
                    "Expected label to be {}, but got {}",
                    label, &item.label
                );
            }
            CompletionAssertion::LabelAndKind(label, kind) => {
                assert_eq!(
                    &item.label, label,
                    "Expected label to be {}, but got {}",
                    label, &item.label
                );
                assert_eq!(
                    &item.kind, kind,
                    "Expected kind to be {:?}, but got {:?}",
                    kind, &item.kind
                );
            }
            CompletionAssertion::LabelNotExists(label) => {
                assert_ne!(
                    &item.label, label,
                    "Expected label {label} not to exist, but found it"
                );
            }
            CompletionAssertion::KindNotExists(kind) => {
                assert_ne!(
                    &item.kind, kind,
                    "Expected kind {kind:?} not to exist, but found it"
                );
            }
            CompletionAssertion::LabelAndDesc(label, desc) => {
                assert_eq!(
                    &item.label, label,
                    "Expected label to be {}, but got {}",
                    label, &item.label
                );
                assert_eq!(
                    &item.description, desc,
                    "Expected desc to be {}, but got {}",
                    desc, &item.description
                );
            }
            CompletionAssertion::CompletionTextAndRange(txt, text_range) => {
                assert_eq!(
                    item.completion_text.as_ref().map(|t| t.text.as_str()),
                    Some(txt.as_str()),
                    "Expected completion text to be {}, but got {}",
                    txt,
                    item.completion_text
                        .as_ref()
                        .map(|t| t.text.clone())
                        .unwrap_or("None".to_string())
                );

                assert_eq!(
                    item.completion_text.as_ref().map(|t| &t.range),
                    Some(text_range),
                    "Expected range to be {:?}, but got {:?}",
                    text_range,
                    item.completion_text
                        .as_ref()
                        .map(|t| format!("{:?}", &t.range))
                        .unwrap_or("None".to_string())
                );
            }
        }
    }
}

pub(crate) async fn assert_complete_results(
    query: &str,
    assertions: Vec<CompletionAssertion>,
    setup: Option<&str>,
    pool: &PgPool,
) {
    let (tree, cache) = get_test_deps(setup, query.into(), pool).await;
    let params = get_test_params(&tree, &cache, query.into());
    let items = complete(params);

    let (not_existing, existing): (Vec<CompletionAssertion>, Vec<CompletionAssertion>) =
        assertions.into_iter().partition(|a| match a {
            CompletionAssertion::LabelNotExists(_) | CompletionAssertion::KindNotExists(_) => true,
            CompletionAssertion::Label(_)
            | CompletionAssertion::LabelAndKind(_, _)
            | CompletionAssertion::CompletionTextAndRange(_, _)
            | CompletionAssertion::LabelAndDesc(_, _) => false,
        });

    assert!(
        items.len() >= existing.len(),
        "Not enough items returned. Expected at least {} items, but got {}",
        existing.len(),
        items.len()
    );

    for item in &items {
        for assertion in &not_existing {
            assertion.assert(item);
        }
    }

    existing
        .into_iter()
        .zip(items.into_iter())
        .for_each(|(assertion, result)| {
            assertion.assert(&result);
        });
}

pub(crate) async fn assert_no_complete_results(query: &str, setup: Option<&str>, pool: &PgPool) {
    let (tree, cache) = get_test_deps(setup, query.into(), pool).await;
    let params = get_test_params(&tree, &cache, query.into());
    let items = complete(params);

    assert_eq!(items.len(), 0)
}
