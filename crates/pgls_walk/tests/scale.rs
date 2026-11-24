//! Allocations of the walk over a generated file, on the catalog of a fresh Postgres 18
//! database. The budgets are those of checkpoint 0 (S8, rows 28–33): counts, not timings, so they
//! hold on every machine. Each change that makes the walk cheaper lowers its budget to the new
//! count.
//!
//! The 1k-statement file runs with every test run; the 20k-statement file is ignored, and the
//! `Scale` job of CI runs it in release.

#![allow(
    clippy::expect_used,
    reason = "a test fails on unexpected data; clippy exempts `#[test]` functions, not their helpers"
)]

use std::sync::Arc;

use pgls_catalog::{CatalogBase, Snapshot};
use pgls_test_utils::scale::{Allocations, CountingAllocator, Dump};
use pgls_text_size::{TextRange, TextSize};
use pgls_walk::{AnalysableStatement, ResolveOptions, Walk, WalkParams, walk_statements};

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

/// The statements of a dump, parsed, and the catalog of a fresh database.
struct Fixture {
    stmts: Vec<AnalysableStatement>,
    ranges: Vec<TextRange>,
    base: Arc<CatalogBase>,
    search_path: Vec<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    /// Only what the catalog needs, as before a statement that a request is about.
    ApplyOnly,
    /// The resolution of every statement, as the linter asks for it.
    Lint,
}

impl Fixture {
    fn new(dump: &Dump) -> Self {
        let mut stmts = Vec::new();
        let mut ranges = Vec::new();
        let mut start = TextSize::new(0);
        for text in dump.statement_texts() {
            let range = TextRange::at(start, TextSize::of(text));
            start = range.end() + TextSize::of("\n\n");
            let root = pgls_query::parse(text)
                .expect("the dump parses")
                .into_root()
                .expect("a statement");
            stmts.push(AnalysableStatement::new(root, range).with_sql(text));
            ranges.push(range);
        }
        let version = pgls_postgres_regress::version(18).expect("the fixtures of Postgres 18");
        let snapshot: Snapshot =
            serde_json::from_str(&version.catalog_json()).expect("the catalog of Postgres 18");
        let search_path = snapshot.expand_search_path(&["public".to_owned()]);
        Self {
            stmts,
            ranges,
            base: Arc::new(CatalogBase::new(Arc::new(snapshot))),
            search_path,
        }
    }

    /// Walks every statement and returns the walk and what it allocated.
    fn walk(&self, mode: Mode) -> (Walk, Allocations) {
        let never = || false;
        Allocations::during(|| {
            walk_statements(
                WalkParams {
                    stmts: &self.stmts,
                    unparsable: &[],
                    statement_ranges: &self.ranges,
                    catalog_base: Some(Arc::clone(&self.base)),
                    search_path: &self.search_path,
                    sections: &[],
                    end: TextSize::from(u32::MAX - 1),
                    best_effort: false,
                    from: None,
                    checkpoints: &[],
                    resolve: ResolveOptions {
                        collect_references: false,
                        collect_lookups: false,
                    },
                    should_stop: &never,
                },
                |stmt, ctx| {
                    if mode == Mode::Lint {
                        std::hint::black_box(ctx.resolution(stmt));
                    }
                },
            )
        })
    }
}

/// Asserts that `count` is within `budget`, and prints both: `--nocapture` shows today's counts.
macro_rules! assert_budget {
    ($what:expr, $count:expr, $budget:expr) => {{
        let (count, budget) = ($count as f64, $budget as f64);
        println!("{}: {count} (budget {budget})", $what);
        assert!(
            count <= budget,
            "{}: {count} over the budget of {budget}",
            $what
        );
    }};
}

impl Fixture {
    /// Walks a dump of `statements` statements and asserts the budgets.
    fn assert_within_budget(statements: usize) {
        let dump = Dump::new(statements);
        let fixture = Fixture::new(&dump);
        let n = statements as f64;

        // Row 28: the walk that only applies the statements to the catalog.
        let (walk, apply) = fixture.walk(Mode::ApplyOnly);
        assert_budget!("walk, apply only (allocations)", apply.count, 140.0 * n);

        // Row 29: the walk that resolves every statement.
        let (_, lint) = fixture.walk(Mode::Lint);
        assert_budget!("walk, lint (allocations)", lint.count, 290.0 * n);

        // Row 32: a checkpoint or a savepoint clones the catalog.
        let (clone, allocations) = Allocations::during(|| walk.catalog.clone());
        drop(clone);
        assert_budget!("Catalog::clone (allocations)", allocations.count, 1.7 * n);

        // Row 33: hover and completions materialise the catalog. Today: 57,135 allocations at 1k
        // statements and 281,382 at 20k, so about 45,300 for the base and 11.8 per statement.
        let (_, allocations) = Allocations::during(|| walk.catalog.snapshot());
        assert_budget!(
            "Catalog::snapshot (allocations)",
            allocations.count,
            1.05 * (45_333.0 + 11.81 * n)
        );

        // Rows 30 and 31: a transaction around every ten statements costs little, independent of
        // the catalog's size, where it once copied the catalog per `BEGIN` (13.5 times the walk).
        let wrapped = Fixture::new(&Dump::with_transactions(statements));
        // A `BEGIN` and a `COMMIT` per pair.
        let pairs = (wrapped.stmts.len() - fixture.stmts.len()) / 2;
        let (_, with_transactions) = wrapped.walk(Mode::ApplyOnly);
        let per_pair = with_transactions.count.saturating_sub(apply.count) as f64 / pairs as f64;
        assert_budget!("walk, allocations per BEGIN … COMMIT", per_pair, 60.0);
    }
}

#[test]
fn walk_1k_statements_within_budget() {
    Fixture::assert_within_budget(1_000);
}

#[test]
#[ignore = "slow in debug; CI's Scale job runs it in release"]
fn scale_20k_walk_within_budget() {
    Fixture::assert_within_budget(20_000);
}
