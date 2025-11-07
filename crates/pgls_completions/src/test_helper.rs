use pgls_schema_cache::SchemaCache;
use pgls_test_utils::QueryWithCursorPosition;
use pgls_text_size::TextRange;
use regex::Regex;
use sqlx::{Executor, PgPool};
use std::{collections::HashMap, fmt::Write, sync::OnceLock};

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

enum ChunkToType {
    WithCompletions(String),
    WithoutCompletions(String),
}

static COMMENT_RE: OnceLock<Regex> = OnceLock::new();

fn comment_regex() -> &'static Regex {
    COMMENT_RE.get_or_init(|| Regex::new(r"<\d+>").unwrap())
}

pub(crate) struct TestCompletionsBuilder<'a> {
    prefixes: Vec<String>,
    tokens_to_type: Vec<ChunkToType>,
    appendices: Vec<String>,
    comments: std::collections::HashMap<usize, String>,
    pool: &'a PgPool,
    setup: Option<&'a str>,
    comment_position: usize,
}

impl<'a> TestCompletionsBuilder<'a> {
    pub(crate) fn new(pool: &'a PgPool, setup: Option<&'a str>) -> Self {
        Self {
            prefixes: Vec::new(),
            tokens_to_type: Vec::new(),
            appendices: Vec::new(),
            comments: HashMap::new(),
            pool,
            setup,
            comment_position: 0,
        }
    }

    pub(crate) fn prefix_static(mut self, it: &str) -> Self {
        self.prefixes.push(it.trim().to_string());
        self
    }

    pub(crate) fn append_static(mut self, it: &str) -> Self {
        self.appendices.push(it.trim().to_string());
        self
    }

    pub(crate) fn type_sql(mut self, it: &str) -> Self {
        assert_eq!(
            self.appendices.len(),
            0,
            "Make sure to call appendices LAST."
        );
        self.tokens_to_type
            .push(ChunkToType::WithCompletions(it.trim().to_string()));
        self
    }

    pub(crate) fn type_without_completions(mut self, it: &str) -> Self {
        assert_eq!(
            self.appendices.len(),
            0,
            "Make sure to call appendices LAST."
        );
        self.tokens_to_type
            .push(ChunkToType::WithoutCompletions(it.trim().to_string()));
        self
    }

    pub(crate) fn comment(mut self, comment: &str) -> Self {
        self.comment_position += 1;
        self.comments
            .insert(self.comment_position, comment.to_string());
        self
    }

    pub(crate) async fn snapshot(self) {
        if let Some(setup) = self.setup {
            self.pool.execute(setup).await.expect("Invalid Setup!");
        }

        let schema_cache = SchemaCache::load(self.pool)
            .await
            .expect("Failed to load Schema Cache");

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
            .expect("Error loading sql language");

        let mut joined_prefix = self.prefixes.join("\n");
        if joined_prefix.len() > 0 {
            joined_prefix.push_str("\n");
        };

        let mut joined_appendix = String::new();
        if self.appendices.len() > 0 {
            joined_appendix.push_str("\n");
        }
        joined_appendix.push_str(self.appendices.join("\n").as_str());

        let mut snapshot_result = String::new();

        for chunk in &self.tokens_to_type {
            match chunk {
                ChunkToType::WithCompletions(sql) => {
                    let whitespace_count = sql.chars().filter(|c| c.is_ascii_whitespace()).count();
                    let whitespace_split = sql.split_ascii_whitespace().enumerate();

                    for (whitespace_idx, token) in whitespace_split {
                        let dot_count = token.chars().filter(|c| *c == '.').count();
                        let dotted_split = token.split(".").enumerate();

                        for (dot_idx, og_part) in dotted_split {
                            let comment_indicator = comment_regex().find(og_part);
                            let comment = comment_indicator.and_then(|n| {
                                let num = n.as_str().replace("<", "").replace(">", "");
                                let num: usize = num
                                    .parse()
                                    .expect("Regex should only find matches with numbers");
                                self.comments.get(&num).map(|s| s.as_str())
                            });

                            let part = comment_regex().replace_all(og_part, "");

                            if joined_prefix.len() > 0 {
                                let query = format!(
                                    "{}{}{}{}",
                                    joined_prefix,
                                    if dot_idx <= dot_count { "" } else { "." },
                                    QueryWithCursorPosition::cursor_marker(),
                                    joined_appendix,
                                );

                                self.completions_snapshot(
                                    query.into(),
                                    &mut snapshot_result,
                                    &schema_cache,
                                    &mut parser,
                                    comment,
                                )
                                .await;
                            }

                            if part.len() > 1 {
                                let query = format!(
                                    "{}{}{}{}",
                                    joined_prefix,
                                    &part[..1],
                                    QueryWithCursorPosition::cursor_marker(),
                                    joined_appendix,
                                );

                                self.completions_snapshot(
                                    query.into(),
                                    &mut snapshot_result,
                                    &schema_cache,
                                    &mut parser,
                                    if comment_indicator
                                        .is_some_and(|txt| og_part.starts_with(txt.as_str()))
                                    {
                                        None
                                    } else {
                                        comment
                                    },
                                )
                                .await;
                            };

                            if whitespace_idx == whitespace_count && dot_idx == dot_count {
                                let query = format!(
                                    "{}{} {}{}",
                                    joined_prefix,
                                    part,
                                    QueryWithCursorPosition::cursor_marker(),
                                    joined_appendix,
                                );

                                self.completions_snapshot(
                                    query.into(),
                                    &mut snapshot_result,
                                    &schema_cache,
                                    &mut parser,
                                    None,
                                )
                                .await;
                            }

                            joined_prefix.push_str(&part);

                            if dot_idx < dot_count {
                                joined_prefix.push_str(".");
                            }
                        }

                        if whitespace_idx < whitespace_count {
                            // note: we're sanitizing the white_space of typed SQL to simple spaces.
                            joined_prefix.push_str(" ");
                        }
                    }

                    joined_prefix.push_str("\n");
                }

                ChunkToType::WithoutCompletions(sql) => {
                    joined_prefix.push_str(sql.as_str());
                    joined_prefix.push_str("\n");
                }
            }
        }

        insta::assert_snapshot!(snapshot_result);
    }

    async fn completions_snapshot(
        &self,
        query: QueryWithCursorPosition,
        writer: &mut String,
        schema: &SchemaCache,
        parser: &mut tree_sitter::Parser,
        comment: Option<&str>,
    ) {
        let (pos, mut sql) = query.get_text_and_position();
        if sql.len() == 0 {
            return;
        }

        println!("'{sql}', {pos}");

        let tree = parser.parse(&sql, None).expect("Invalid TS Tree!");

        let params = CompletionParams {
            text: sql.clone(),
            position: (pos as u32).into(),
            schema,
            tree: &tree,
        };

        let items = complete(params);

        if pos < sql.len() {
            sql.replace_range(pos..pos, "|");
        } else {
            let diff = pos - sql.len();

            sql.push_str(&" ".repeat(diff));
            sql.push_str("|");
        }
        writeln!(writer, "{sql}").unwrap();
        writeln!(writer).unwrap();

        if let Some(c) = comment {
            writeln!(writer, "**{}**", c).unwrap();
            writeln!(writer).unwrap();
        }

        if items.len() == 0 {
            writeln!(writer, "No Results").unwrap();
        } else {
            writeln!(writer, "Results:").unwrap();

            let max_idx = std::cmp::min(items.len(), 5);
            for item in &items[..max_idx] {
                write!(
                    writer,
                    "{}",
                    item.completion_text
                        .as_ref()
                        .map(|c| c.text.as_str())
                        .unwrap_or(item.label.as_str())
                )
                .unwrap();

                write!(writer, " - ").unwrap();

                match item.kind {
                    CompletionItemKind::Schema | CompletionItemKind::Role => {}
                    _ => {
                        write!(writer, "{}.", item.description).unwrap();
                    }
                }

                write!(writer, "{} ({})", item.label, item.kind).unwrap();

                writeln!(writer).unwrap();
            }
        }

        writeln!(writer).unwrap();

        writeln!(writer, "--------------").unwrap();
        writeln!(writer).unwrap();
    }
}
