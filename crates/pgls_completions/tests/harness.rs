//! The completion harness: requests completions at cut points of Postgres' regression SQL and
//! checks every answer, without a database.
//!
//! The catalog is that of a fresh database of the version (`catalog.json.gz` of
//! `pgls_postgres_regress`) with the DDL of the file before the statement applied, as the
//! workspace's walk applies it. A cut point is a token of an accepted statement and a cursor
//! offset in it: at the token's start, after its first character, in its middle, and at its end.
//! The text up to the cursor is the statement, like the user is typing it at the end of the
//! file. Every `CUT_POINT_STRIDE`th cut point is requested, and for each request:
//!
//! - completing must not panic;
//! - the range of an item's text lies in the statement, starts on a character boundary, and ends
//!   at the cursor;
//! - inserting an item keeps the statement a viable prefix: Postgres' parser either accepts it or
//!   fails at its end. The parser fails at the first token that can't continue any statement, so
//!   an item that fails earlier can't be valid there. A snippet counts up to its first tab stop.
//!
//! The summary of the failures, grouped by kind with the shortest examples, is compared with
//! `tests/snapshots/harness__regress_<major>.snap`, so a change to the completions shows up as a
//! changed summary.
//!
//! The run takes about two minutes in release: a request takes 17 ms at the median and up
//! to half a second, most of it in `complete`. It is ignored in the regular debug run; CI runs it
//! with
//!
//! ```sh
//! cargo test -p pgls_completions --release --test harness -- --ignored
//! ```
//!
//! Prefix it with `INSTA_UPDATE=always` to accept a changed summary.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "a test fails on unexpected data; clippy exempts `#[test]` functions, not their helpers"
)]

use std::{
    collections::BTreeMap,
    fmt::Write,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::Arc,
};

use pgls_catalog::{CatalogBase, Snapshot};
use pgls_completions::{CompletionItem, CompletionParams, complete};
use pgls_postgres_regress::{File, Verdict, Version};
use pgls_text_size::{TextRange, TextSize};
use pgls_walk::{AnalysableStatement, ResolveOptions, WalkParams, walk_statements};

/// The version whose regression SQL the harness runs over.
const MAJOR: u32 = 18;

/// Every how many cut points the harness requests completions: about 3,000 requests of about
/// 1.6 million cut points. The cut points are in file order, so the sample is the same on every
/// run.
const CUT_POINT_STRIDE: usize = 500;

/// How many examples the summary shows per group of failures, the shortest first.
const EXAMPLES_MAX: usize = 3;

/// How much of the text before the cursor an example shows.
const EXAMPLE_PREFIX_CHARS_MAX: usize = 60;

/// Tokens appended to a text that the parser rejects before its end, before the harness decides
/// that the text isn't a viable prefix. Postgres' lexer ([`base_yylex`]) looks at the token after
/// `NOT`, `NULLS`, `WITH`, `FORMAT` and a few more, and makes them different tokens, so that such
/// a keyword at the end can fail although a statement continues it.
///
/// [`base_yylex`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parser.c#L111
const FOLLOWERS: &[&str] = &[
    " x",
    " 1",
    " (",
    " between",
    " in",
    " like",
    " first",
    " time",
    " ordinality",
    " json",
];

#[test]
#[ignore = "takes minutes in debug; CI runs it in release"]
fn regress() {
    let version = pgls_postgres_regress::version(MAJOR).expect("the fixtures of the version");
    let mut harness = Harness::new(&version);
    for file in version.files() {
        harness.run(&file);
    }
    insta::assert_snapshot!(format!("regress_{MAJOR}"), harness.summary.report());
}

struct Harness {
    catalog: Arc<CatalogBase>,
    parser: tree_sitter::Parser,
    /// The cut points seen so far, sampled or not.
    cut_points: usize,
    summary: Summary,
}

/// The statement at a cut point, up to the cursor.
struct Cut<'a> {
    file: &'a File,
    /// The text of the statement up to the cursor.
    text: &'a str,
    /// The start of the statement in the file.
    stmt_start: TextSize,
    /// The start of the token the cursor is in or at.
    token_start: usize,
}

impl Harness {
    fn new(version: &Version) -> Self {
        let snapshot: Snapshot = serde_json::from_str(&version.catalog_json())
            .unwrap_or_else(|e| panic!("parse the catalog of {}: {e}", version.major));
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
            .expect("the SQL grammar");
        Self {
            catalog: Arc::new(CatalogBase::new(Arc::new(snapshot))),
            parser,
            cut_points: 0,
            summary: Summary::default(),
        }
    }

    /// Requests completions at the sampled cut points of the accepted statements of `file`, each
    /// with the catalog after the statements before it.
    fn run(&mut self, file: &File) {
        let stmts: Vec<AnalysableStatement> = file
            .statements
            .iter()
            .filter(|statement| statement.verdict == Verdict::Accepted)
            .filter_map(|statement| {
                let root = pgls_query::parse(file.sql(statement)).ok()?.into_root()?;
                Some(AnalysableStatement::new(root, statement.range))
            })
            .collect();
        let statement_ranges: Vec<TextRange> = stmts.iter().map(|stmt| stmt.range).collect();
        let search_path = ["public".to_owned()];
        walk_statements(
            WalkParams {
                stmts: &stmts,
                unparsable: &[],
                statement_ranges: &statement_ranges,
                catalog_base: Some(Arc::clone(&self.catalog)),
                search_path: &search_path,
                sections: &[],
                end: TextSize::from(u32::MAX),
                best_effort: true,
                from: None,
                checkpoints: &[],
                resolve: ResolveOptions::default(),
                should_stop: &|| false,
            },
            |stmt, file_context| {
                if stmt.function.is_some() {
                    return;
                }
                let sql = &file.source[stmt.range];
                let cursors = self.sampled_cursors(sql);
                if cursors.is_empty() {
                    return;
                }
                let snapshot = file_context
                    .catalog()
                    .snapshot()
                    .expect("a catalog with a database snapshot");
                for (token, cursor) in cursors {
                    let Some(text) = sql.get(..cursor) else {
                        continue;
                    };
                    let viable = Viability::of(sql.get(..token.end).unwrap_or_default());
                    self.request(
                        &Cut {
                            file,
                            text,
                            stmt_start: stmt.range.start(),
                            token_start: token.start,
                        },
                        &snapshot,
                        viable,
                    );
                }
            },
        );
    }

    /// The tokens of `sql` and the cursors in them of the sampled cut points.
    fn sampled_cursors(&mut self, sql: &str) -> Vec<(std::ops::Range<usize>, usize)> {
        let Ok(scanned) = pgls_query::scan(sql) else {
            return Vec::new();
        };
        let mut cursors = Vec::new();
        for token in scanned.tokens {
            let Some(range) = token.range() else {
                continue;
            };
            let (start, end) = (usize::from(range.start()), usize::from(range.end()));
            let len = end.saturating_sub(start);
            let mut offsets = vec![0, 1, len / 2, len];
            offsets.retain(|offset| *offset <= len);
            offsets.sort_unstable();
            offsets.dedup();
            for offset in offsets {
                let cursor = start + offset;
                if !sql.is_char_boundary(cursor) {
                    continue;
                }
                self.cut_points += 1;
                if self.cut_points.is_multiple_of(CUT_POINT_STRIDE) {
                    cursors.push((start..end, cursor));
                }
            }
        }
        cursors
    }

    /// Requests completions at the end of `cut.text` and checks them. `viable` is whether the
    /// statement up to the end of the token is a viable prefix; only then must the items keep it
    /// one.
    fn request(&mut self, cut: &Cut, snapshot: &Snapshot, viable: Viability) {
        self.summary.requests += 1;
        let Some(tree) = self.parser.parse(cut.text, None) else {
            return;
        };
        let items = catch_unwind(AssertUnwindSafe(|| {
            complete(CompletionParams {
                position: TextSize::new(u32::try_from(cut.text.len()).unwrap_or(u32::MAX)),
                schema: snapshot,
                text: cut.text.to_owned(),
                tree: &tree,
                function: None,
            })
        }));
        let items = match items {
            Ok(items) => items,
            Err(panic) => {
                let message = panic
                    .downcast_ref::<String>()
                    .map(String::as_str)
                    .or_else(|| panic.downcast_ref::<&str>().copied())
                    .unwrap_or("a panic without a message");
                self.summary.fail(format!("panic: {message}"), cut, None);
                return;
            }
        };
        self.summary.items += items.len();
        for item in &items {
            let Some(applied) = self.check_range(cut, item) else {
                continue;
            };
            if viable == Viability::Viable && Viability::of(&applied) == Viability::NotViable {
                self.summary.fail(
                    format!("not a viable prefix: {}", item.kind),
                    cut,
                    Some(item),
                );
            }
        }
    }

    /// The statement with `item` inserted at the cursor, or `None` after recording why the range
    /// of its text doesn't fit. An item without a text replaces the word before the cursor.
    fn check_range(&mut self, cut: &Cut, item: &CompletionItem) -> Option<String> {
        let cursor = cut.text.len();
        let (range, text) = match &item.completion_text {
            Some(completion) => {
                let text = if completion.is_snippet {
                    // The text a snippet inserts before its first tab stop or placeholder,
                    // where the user types next: `f(${1:a}, ${2:b})` is `f(`. See
                    // https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#snippet_syntax
                    let mut text = String::with_capacity(completion.text.len());
                    let mut chars = completion.text.chars();
                    while let Some(c) = chars.next() {
                        match c {
                            '\\' => text.extend(chars.next()),
                            '$' => break,
                            c => text.push(c),
                        }
                    }
                    text
                } else {
                    completion.text.clone()
                };
                (
                    usize::from(completion.range.start())..usize::from(completion.range.end()),
                    text,
                )
            }
            None => (cut.token_start.min(cursor)..cursor, item.label.clone()),
        };
        let reason = if range.end > cursor {
            Some("range: past the cursor")
        } else if range.end < cursor {
            Some("range: ends before the cursor")
        } else if !cut.text.is_char_boundary(range.start) {
            Some("range: not on a character boundary")
        } else {
            None
        };
        if let Some(reason) = reason {
            self.summary
                .fail(format!("{reason}: {}", item.kind), cut, Some(item));
            return None;
        }
        let mut applied = cut.text.get(..range.start)?.to_owned();
        applied.push_str(&text);
        Some(applied)
    }
}

/// Whether Postgres' parser can continue a text to a statement.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Viability {
    Viable,
    NotViable,
}

impl Viability {
    /// Whether `text` is a statement or the start of one: the parser accepts it, fails only at
    /// its end, or accepts it with one of [`FOLLOWERS`] appended.
    fn of(text: &str) -> Self {
        let fails_before_end = |text: &str| match pgls_query::parse(text) {
            Ok(_) => false,
            Err(pgls_query::Error::Parse(message)) => !message.ends_with("at end of input"),
            Err(_) => true,
        };
        if !fails_before_end(text)
            || FOLLOWERS
                .iter()
                .any(|follower| !fails_before_end(&format!("{text}{follower}")))
        {
            Self::Viable
        } else {
            Self::NotViable
        }
    }
}

#[derive(Default)]
struct Summary {
    requests: usize,
    items: usize,
    /// The failures by kind: how many, and the shortest examples.
    failures: BTreeMap<String, Failures>,
}

#[derive(Default)]
struct Failures {
    count: usize,
    /// The length of the text before the cursor, and the example. The shortest are kept.
    examples: Vec<(usize, String)>,
}

impl Summary {
    /// Records a failure of `kind` at `cut`. The examples of a kind are at different cut points,
    /// with the first failing item of each.
    fn fail(&mut self, kind: String, cut: &Cut, item: Option<&CompletionItem>) {
        let failures = self.failures.entry(kind).or_default();
        failures.count += 1;
        let (line, col) = cut.file.line_col_at(cut.stmt_start);
        let skipped = cut
            .text
            .chars()
            .count()
            .saturating_sub(EXAMPLE_PREFIX_CHARS_MAX);
        let tail: String = cut.text.chars().skip(skipped).collect();
        let at = format!(
            "{}.sql:{line}:{col} `{}`",
            cut.file.name,
            tail.replace('\n', " ")
        );
        if failures
            .examples
            .iter()
            .any(|(_, example)| example.starts_with(&at))
        {
            return;
        }
        let mut example = at;
        if let Some(item) = item {
            let text = item
                .completion_text
                .as_ref()
                .map_or(item.label.as_str(), |completion| completion.text.as_str());
            write!(example, " + `{text}`").unwrap();
        }
        failures.examples.push((cut.text.len(), example));
        failures.examples.sort_unstable();
        failures.examples.truncate(EXAMPLES_MAX);
    }

    fn report(&self) -> String {
        let failed: usize = self.failures.values().map(|failures| failures.count).sum();
        let mut report = format!(
            "requests: {}\nitems: {}\nfailures: {failed}\n",
            self.requests, self.items
        );
        for (kind, failures) in &self.failures {
            write!(report, "\n{kind}: {}\n", failures.count).unwrap();
            for (_, example) in &failures.examples {
                writeln!(report, "  {example}").unwrap();
            }
        }
        report
    }
}
