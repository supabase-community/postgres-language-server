//! When the whole input parses, the splitter's statements parse one by one to the same statements
//! as the whole: the splitter neither cuts a statement in pieces nor merges two. On any text, its
//! ranges are ordered, not empty, in bounds and on char boundaries.
//!
//! One kind of cut is by design: where a `;` is missing, the splitter starts a statement at a
//! statement keyword, even where Postgres reads it as a bare column label (`select 1 select;`,
//! `select x abort`). Inputs with such a cut are skipped.

#![no_main]

use libfuzzer_sys::fuzz_target;
use pgls_pretty_print::normalize_ast;
use pgls_query::NodeEnum;
use pgls_text_size::{TextRange, TextSize};

/// Longer inputs mostly cost time; the splitter's decisions are local.
const INPUT_LEN_MAX: usize = 3000;

fuzz_target!(|data: &[u8]| {
    let Ok(sql) = std::str::from_utf8(data) else {
        return;
    };
    if sql.len() > INPUT_LEN_MAX || sql.contains('\0') {
        return;
    }
    // libpg_query's own splitter mustn't panic on any text either.
    let _ = pgls_query::split_with_parser(sql);
    let split = Split {
        sql,
        ranges: pgls_statement_splitter::split(sql).ranges,
    };
    // On any text, the statements are ordered, not empty, and in bounds and on char boundaries.
    let mut end = TextSize::from(0);
    for &range in &split.ranges {
        assert!(
            !range.is_empty()
                && range.start() >= end
                && sql.get(std::ops::Range::<usize>::from(range)).is_some(),
            "splitter_vs_parser: statement empty, overlapping, out of bounds or off a char \
             boundary\nrange {range:?} of {:?}\ninput: {sql:?}",
            split.ranges
        );
        end = range.end();
    }
    let Some(whole) = Statements::parse(sql) else {
        return;
    };
    let mut pieces = Vec::new();
    for &range in &split.ranges {
        match Statements::parse(&sql[range]) {
            Some(statements) => pieces.extend(statements.0),
            None if split.has_bare_label_cut() => return,
            None => panic!(
                "splitter_vs_parser: a statement doesn't parse although the input does\n\
                 statement: {:?}\nstatements: {:?}\ninput: {sql:?}",
                &sql[range],
                split.texts()
            ),
        }
    }
    if whole.0 != pieces && !split.has_bare_label_cut() {
        panic!(
            "splitter_vs_parser: the statements differ from the input's\n{} statements in the \
             input, {} in the splitter's\nstatements: {:?}\ninput: {sql:?}",
            whole.0.len(),
            pieces.len(),
            split.texts()
        );
    }
});

/// The statements of a text that parses, without locations.
struct Statements(Vec<NodeEnum>);

impl Statements {
    fn parse(sql: &str) -> Option<Statements> {
        let parsed = pgls_query::parse(sql).ok()?;
        Some(Statements(
            parsed
                .protobuf
                .stmts
                .into_iter()
                .filter_map(|raw| raw.stmt?.node)
                .map(|mut node| {
                    normalize_ast(&mut node);
                    node
                })
                .collect(),
        ))
    }
}

struct Split<'a> {
    sql: &'a str,
    ranges: Vec<TextRange>,
}

impl Split<'_> {
    /// Whether the splitter cut without a `;` where a complete statement is followed by one
    /// starting with a keyword: the missing-semicolon recovery that is by design.
    fn has_bare_label_cut(&self) -> bool {
        self.ranges.windows(2).any(|pair| {
            let [before, after] = pair else {
                return false;
            };
            let (before, after) = (&self.sql[*before], &self.sql[*after]);
            let word = after
                .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                .next()
                .unwrap_or_default();
            !before.ends_with(';')
                && pgls_lexer::keyword_category(word).is_some()
                && Statements::parse(before).is_some()
        })
    }

    fn texts(&self) -> Vec<&str> {
        self.ranges.iter().map(|&range| &self.sql[range]).collect()
    }
}
