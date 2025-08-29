use pgt_schema_cache::SchemaCache;
use pgt_test_utils::QueryWithCursorPosition;
use sqlx::{Executor, PgPool};

use crate::{CompletionItem, CompletionItemKind, CompletionParams, complete};

pub(crate) async fn get_test_deps(
    setup: Option<&str>,
    input: QueryWithCursorPosition,
    test_db: &PgPool,
) -> (tree_sitter::Tree, pgt_schema_cache::SchemaCache) {
    if let Some(setup) = setup {
        test_db
            .execute(setup)
            .await
            .expect("Failed to execute setup query");
    }

    let schema_cache = SchemaCache::load(test_db, vec![])
        .await
        .expect("Failed to load Schema Cache");

    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(tree_sitter_sql::language())
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
) -> (tree_sitter::Tree, pgt_schema_cache::SchemaCache) {
    let pool = sqlx::PgPool::connect(conn_str)
        .await
        .expect("Unable to connect to database.");

    let schema_cache = SchemaCache::load(&pool, vec![])
        .await
        .expect("Failed to load Schema Cache");

    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(tree_sitter_sql::language())
        .expect("Error loading sql language");

    let tree = parser.parse(input.to_string(), None).unwrap();

    (tree, schema_cache)
}

pub(crate) fn get_test_params<'a>(
    tree: &'a tree_sitter::Tree,
    schema_cache: &'a pgt_schema_cache::SchemaCache,
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
                    "Expected label {} not to exist, but found it",
                    label
                );
            }
            CompletionAssertion::KindNotExists(kind) => {
                assert_ne!(
                    &item.kind, kind,
                    "Expected kind {:?} not to exist, but found it",
                    kind
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
