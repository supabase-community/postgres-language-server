use insta::assert_snapshot;
use pgls_schema_cache::SchemaCache;
use pgls_test_utils::QueryWithCursorPosition;
use pgls_text_size::TextRange;
use regex::Regex;
use sqlx::{Executor, PgPool};
use std::{collections::HashMap, fmt::Write, sync::OnceLock};
use unindent::unindent;

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
    #[allow(unused)]
    LabelAndDesc(String, String),
    LabelNotExists(String),
    #[allow(unused)]
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
    #[allow(unused)]
    WithoutCompletions(String),
}

static COMMENT_RE: OnceLock<Regex> = OnceLock::new();

fn comment_regex() -> &'static Regex {
    COMMENT_RE.get_or_init(|| Regex::new(r"<\d+>").unwrap())
}

pub(crate) struct TestCompletionsCase {
    tokens_to_type: Vec<ChunkToType>,
    surrounding_statement: String,
    comments: std::collections::HashMap<usize, String>,
    comment_position: usize,
}

impl TestCompletionsCase {
    pub(crate) fn new() -> Self {
        Self {
            tokens_to_type: Vec::new(),
            surrounding_statement: String::new(),
            comments: HashMap::new(),
            comment_position: 0,
        }
    }

    pub(crate) fn inside_static_statement(mut self, it: &str) -> Self {
        assert!(it.contains("<sql>"));
        self.surrounding_statement = unindent(it);
        self
    }

    pub(crate) fn type_sql(mut self, it: &str) -> Self {
        self.tokens_to_type
            .push(ChunkToType::WithCompletions(it.trim().to_string()));
        self
    }

    #[allow(unused)]
    pub(crate) fn type_without_completions(mut self, it: &str) -> Self {
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

    async fn generate_snapshot(
        &self,
        schema_cache: &SchemaCache,
        parser: &mut tree_sitter::Parser,
    ) -> String {
        let mut stmt_parts = self.surrounding_statement.split("<sql>");
        let mut pre_sql = stmt_parts.next().unwrap().to_string();
        let post_sql = stmt_parts.next().unwrap_or("").to_string();

        let mut snapshot_result = String::new();

        for chunk in &self.tokens_to_type {
            match chunk {
                ChunkToType::WithCompletions(sql) => {
                    let whitespace_count = sql.chars().filter(|c| c.is_ascii_whitespace()).count();
                    let whitespace_split = sql.split_ascii_whitespace().enumerate();

                    let mut should_close_with_paren = false;

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

                            let part_without_comments = comment_regex().replace_all(og_part, "");

                            let starts_with_paren = part_without_comments.starts_with('(');
                            let ends_with_paren = part_without_comments.ends_with(')');

                            let part_without_parens = if starts_with_paren || ends_with_paren {
                                // we only want to sanitize when the token either starts or ends; that helps
                                // catch end tokens like `('something');`
                                part_without_comments.replace(['(', ')'], "")
                            } else {
                                part_without_comments.to_string()
                            };

                            let is_inside_quotes = part_without_parens.starts_with('"')
                                && part_without_parens.ends_with('"');

                            let part_without_quotes = part_without_parens.replace('"', "");

                            if !pre_sql.is_empty() {
                                let query = format!(
                                    "{}{}{}{}{}",
                                    pre_sql,
                                    if dot_idx <= dot_count { "" } else { "." },
                                    QueryWithCursorPosition::cursor_marker(),
                                    if should_close_with_paren { ")" } else { "" },
                                    post_sql,
                                );

                                self.completions_snapshot(
                                    query.into(),
                                    &mut snapshot_result,
                                    schema_cache,
                                    parser,
                                    comment,
                                )
                                .await;
                            }

                            // try `<pre_sql> (|` and `<pre_sql> (|)`
                            if starts_with_paren {
                                let query1 = format!(
                                    "{}{}({}{}",
                                    pre_sql,
                                    if dot_idx <= dot_count { "" } else { "." },
                                    QueryWithCursorPosition::cursor_marker(),
                                    post_sql,
                                );

                                self.completions_snapshot(
                                    query1.into(),
                                    &mut snapshot_result,
                                    schema_cache,
                                    parser,
                                    comment,
                                )
                                .await;

                                let query2 = format!(
                                    "{}{}({}){}",
                                    pre_sql,
                                    if dot_idx <= dot_count { "" } else { "." },
                                    QueryWithCursorPosition::cursor_marker(),
                                    post_sql,
                                );

                                self.completions_snapshot(
                                    query2.into(),
                                    &mut snapshot_result,
                                    schema_cache,
                                    parser,
                                    comment,
                                )
                                .await;

                                pre_sql.push('(');
                                should_close_with_paren = true;
                            }

                            // try `<pre_sql> "|` and `<pre_sql> "|"`
                            if is_inside_quotes {
                                let query1 = format!(
                                    "{}{}\"{}{}{}",
                                    pre_sql,
                                    if dot_idx <= dot_count { "" } else { "." },
                                    QueryWithCursorPosition::cursor_marker(),
                                    if should_close_with_paren { ")" } else { "" },
                                    post_sql,
                                );

                                self.completions_snapshot(
                                    query1.into(),
                                    &mut snapshot_result,
                                    schema_cache,
                                    parser,
                                    comment,
                                )
                                .await;

                                let query2 = format!(
                                    "{}{}\"{}\"{}{}",
                                    pre_sql,
                                    if dot_idx <= dot_count { "" } else { "." },
                                    QueryWithCursorPosition::cursor_marker(),
                                    if should_close_with_paren { ")" } else { "" },
                                    post_sql,
                                );

                                self.completions_snapshot(
                                    query2.into(),
                                    &mut snapshot_result,
                                    schema_cache,
                                    parser,
                                    comment,
                                )
                                .await;
                            }

                            if part_without_quotes.len() > 1 {
                                let first_token = &part_without_quotes[..1];

                                let query = format!(
                                    "{}{}{}{}{}{}",
                                    pre_sql,
                                    if is_inside_quotes {
                                        format!(r#""{first_token}"#)
                                    } else {
                                        first_token.to_string()
                                    },
                                    QueryWithCursorPosition::cursor_marker(),
                                    if is_inside_quotes { r#"""# } else { "" },
                                    if should_close_with_paren { ")" } else { "" },
                                    post_sql,
                                );

                                self.completions_snapshot(
                                    query.into(),
                                    &mut snapshot_result,
                                    schema_cache,
                                    parser,
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
                                    pre_sql,
                                    if is_inside_quotes {
                                        format!(r#""{}""#, part_without_quotes.as_str())
                                    } else {
                                        part_without_quotes.clone()
                                    },
                                    QueryWithCursorPosition::cursor_marker(),
                                    post_sql,
                                );

                                self.completions_snapshot(
                                    query.into(),
                                    &mut snapshot_result,
                                    schema_cache,
                                    parser,
                                    None,
                                )
                                .await;
                            }

                            pre_sql.push_str(&part_without_parens);

                            if dot_idx < dot_count {
                                pre_sql.push('.');
                            }

                            if ends_with_paren {
                                should_close_with_paren = false;
                                pre_sql.push(')');
                            }
                        }

                        if whitespace_idx < whitespace_count {
                            // note: we're sanitizing the white_space of typed SQL to simple spaces.
                            pre_sql.push(' ');
                        }
                    }

                    pre_sql.push('\n');
                }

                ChunkToType::WithoutCompletions(sql) => {
                    pre_sql.push_str(sql.as_str());
                    pre_sql.push('\n');
                }
            }
        }

        snapshot_result
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
            sql.push('|');
        }
        writeln!(writer, "{sql}").unwrap();

        if let Some(c) = comment {
            writeln!(writer, "**{c}**").unwrap();
        }

        if !items.is_empty() {
            writeln!(writer).unwrap();
            writeln!(writer, "Results:").unwrap();

            let max_idx = std::cmp::min(items.len(), 5);
            for item in &items[..max_idx] {
                write!(
                    writer,
                    "{}",
                    item.completion_text
                        .as_ref()
                        .filter(|c| !c.is_snippet)
                        .map(|c| c.text.as_str())
                        .unwrap_or(item.label.as_str())
                )
                .unwrap();

                write!(writer, " - ").unwrap();

                match item.kind {
                    CompletionItemKind::Schema
                    | CompletionItemKind::Role
                    | CompletionItemKind::Keyword => {}
                    _ => {
                        write!(writer, "{}.", item.description).unwrap();
                    }
                }

                write!(writer, "{} ({})", item.label, item.kind).unwrap();

                writeln!(writer).unwrap();
            }

            writeln!(writer).unwrap();

            writeln!(writer, "--------------").unwrap();
            writeln!(writer).unwrap();
        }
    }
}

pub(crate) struct TestCompletionsSuite<'a> {
    pool: &'a PgPool,
    setup: Option<&'a str>,
    cases: Vec<TestCompletionsCase>,
}

impl<'a> TestCompletionsSuite<'a> {
    pub(crate) fn new(pool: &'a PgPool, setup: Option<&'a str>) -> Self {
        Self {
            pool,
            setup,
            cases: vec![],
        }
    }

    pub(crate) fn with_case(mut self, case: TestCompletionsCase) -> Self {
        self.cases.push(case);
        self
    }

    pub(crate) async fn snapshot(self, snapshot_name: &str) {
        assert!(!self.cases.is_empty(), "Needs at least one Snapshot case.");

        let mut final_snapshot = String::new();

        if let Some(setup) = self.setup {
            self.pool.execute(setup).await.expect("Problem with Setup");
            writeln!(final_snapshot, "***Setup***").unwrap();
            writeln!(final_snapshot).unwrap();
            write!(final_snapshot, "{}", unindent(setup)).unwrap();
            writeln!(final_snapshot).unwrap();
            writeln!(final_snapshot).unwrap();
            writeln!(final_snapshot, "--------------").unwrap();
            writeln!(final_snapshot).unwrap();
        }

        let cache = SchemaCache::load(self.pool)
            .await
            .expect("Problem loading SchemaCache");

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
            .expect("Problem with TreeSitter Grammar");

        let has_more_than_one_case = self.cases.len() > 1;

        for (idx, additional) in self.cases.iter().enumerate() {
            if idx > 0 {
                writeln!(final_snapshot).unwrap();
                writeln!(final_snapshot).unwrap();
                writeln!(final_snapshot).unwrap();

                writeln!(final_snapshot).unwrap();
            }

            if has_more_than_one_case {
                writeln!(final_snapshot, "***Case {}:***", idx + 1).unwrap();
                writeln!(final_snapshot).unwrap();
            }

            let snap = additional.generate_snapshot(&cache, &mut parser).await;

            write!(final_snapshot, "{snap}").unwrap();
        }

        assert_snapshot!(snapshot_name, final_snapshot)
    }
}
