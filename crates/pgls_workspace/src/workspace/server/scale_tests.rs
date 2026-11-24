//! Operation counts of the workspace on generated files of 1k and 20k statements: allocations
//! per request, statements walked, queries sent to the database, tree-sitter parses. The budgets
//! are those of checkpoint 0 (S8): counts, not timings, so they hold on every machine. Each
//! change that makes a request cheaper lowers its budget to the new count.
//!
//! The 1k-statement file runs with every test run. The 20k-statement tests are ignored, and the
//! `Scale` job of CI runs them in release.
//!
//! The allocator counts per thread, so the tests of this binary don't count each other's
//! allocations, and work that a request hands to another thread isn't counted.

use std::sync::atomic::Ordering;

use pgls_analyse::RuleCategories;
use pgls_configuration::{
    Merge, PartialConfiguration, PartialFormatConfiguration, database::PartialDatabaseConfiguration,
};
use pgls_fs::PgLSPath;
use pgls_test_utils::scale::{Allocations, CountingAllocator, Dump};
use pgls_text_size::TextSize;
use sqlx::PgPool;

use super::Counters;
use super::document_cache::TreeParses;
use crate::Workspace;
use crate::features::completions::GetCompletionsParams;
use crate::features::diagnostics::DatabaseChecks;
use crate::features::format::PullFileFormattingParams;
use crate::features::on_hover::OnHoverParams;
use crate::features::semantic_tokens::PullSemanticTokensParams;
use crate::workspace::{
    ChangeFileParams, OpenFileParams, ProjectKey, PullFileDiagnosticsParams,
    RegisterProjectFolderParams, UpdateSettingsParams, server::WorkspaceServer,
};

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

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

/// A workspace with a generated file open.
struct Scale {
    workspace: WorkspaceServer,
    project_key: ProjectKey,
    path: PgLSPath,
    version: i32,
}

impl Scale {
    /// A workspace with `configuration` and the formatter on, without a file.
    fn new(configuration: PartialConfiguration) -> Self {
        let workspace = WorkspaceServer::new();
        let project_key = workspace
            .register_project_folder(RegisterProjectFolderParams {
                path: None,
                real_path: None,
            })
            .expect("a project");
        let mut conf = PartialConfiguration::init();
        conf.merge_with(configuration);
        conf.merge_with(PartialConfiguration {
            format: Some(PartialFormatConfiguration {
                enabled: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        });
        workspace
            .update_settings(UpdateSettingsParams {
                project_key,
                configuration: conf,
                gitignore_matches: vec![],
                vcs_base_path: None,
                workspace_directory: None,
                replace: false,
            })
            .expect("settings");
        Self {
            workspace,
            project_key,
            path: PgLSPath::new("dump.sql"),
            version: 1,
        }
    }

    /// The configuration without a database.
    fn without_database() -> PartialConfiguration {
        PartialConfiguration {
            db: Some(PartialDatabaseConfiguration {
                disable_connection: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    /// The configuration with the test database.
    fn with_database(test_db: &PgPool) -> PartialConfiguration {
        PartialConfiguration {
            db: Some(PartialDatabaseConfiguration {
                database: Some(
                    test_db
                        .connect_options()
                        .get_database()
                        .expect("a database name")
                        .to_string(),
                ),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn counters(&self) -> &Counters {
        &self.workspace.counters
    }

    fn open(&self, sql: &str) -> Allocations {
        let params = OpenFileParams {
            project_key: self.project_key,
            path: self.path.clone(),
            content: sql.to_owned(),
            version: self.version,
        };
        let (result, allocations) = Allocations::during(|| self.workspace.open_file(params));
        result.expect("open the file");
        allocations
    }

    fn change(&mut self, content: String) -> Allocations {
        self.version += 1;
        let params = ChangeFileParams {
            project_key: self.project_key,
            path: self.path.clone(),
            version: self.version,
            content,
        };
        let (result, allocations) = Allocations::during(|| self.workspace.change_file(params));
        result.expect("change the file");
        allocations
    }

    /// Pulls the diagnostics and returns how many there are.
    fn pull(&self, database_checks: DatabaseChecks) -> (usize, Allocations) {
        let (result, allocations) = Allocations::during(|| {
            self.workspace
                .pull_file_diagnostics(PullFileDiagnosticsParams {
                    project_key: self.project_key,
                    path: self.path.clone(),
                    categories: RuleCategories::all(),
                    max_diagnostics: 100_000,
                    only: vec![],
                    skip: vec![],
                    database_checks,
                })
                .map(|result| result.diagnostics.len())
        });
        (result.expect("pull the diagnostics"), allocations)
    }

    /// Hovers at `at` and returns how many statements the request walked.
    fn hover(&self, at: usize) -> (usize, Allocations) {
        let walked = self.counters().walked_statements.load(Ordering::Relaxed);
        let (result, allocations) = Allocations::during(|| {
            self.workspace.on_hover(OnHoverParams {
                project_key: self.project_key,
                path: self.path.clone(),
                position: TextSize::new(u32::try_from(at).expect("a small file")),
            })
        });
        result.expect("hover");
        (
            self.counters().walked_statements.load(Ordering::Relaxed) - walked,
            allocations,
        )
    }

    fn completions(&self, at: usize) -> Allocations {
        let (result, allocations) = Allocations::during(|| {
            self.workspace.get_completions(GetCompletionsParams {
                project_key: self.project_key,
                path: self.path.clone(),
                position: TextSize::new(u32::try_from(at).expect("a small file")),
            })
        });
        result.expect("completions");
        allocations
    }

    fn semantic_tokens(&self) -> Allocations {
        let (result, allocations) = Allocations::during(|| {
            self.workspace
                .pull_semantic_tokens(PullSemanticTokensParams {
                    project_key: self.project_key,
                    path: self.path.clone(),
                    range: None,
                })
                .map(|result| result.tokens.len())
        });
        assert!(result.expect("semantic tokens") > 0);
        allocations
    }

    fn format(&self) -> Allocations {
        let (result, allocations) = Allocations::during(|| {
            self.workspace
                .pull_file_formatting(PullFileFormattingParams {
                    project_key: self.project_key,
                    path: self.path.clone(),
                    range: None,
                })
                .map(|result| result.statements.len())
        });
        assert!(result.expect("format") > 0);
        allocations
    }

    /// The budgets without a database, on a dump of `statements` statements.
    fn assert_within_budget(statements: usize) {
        let dump = Dump::new(statements);
        let n = statements as f64;

        // Rows 1–3: libpg_query's parse, and what its trees keep.
        let (trees, parse) = Allocations::during(|| {
            dump.statement_texts()
                .map(|text| (text.to_owned(), pgls_query::parse(text)))
                .collect::<Vec<_>>()
        });
        assert_budget!("pgls_query::parse (allocations)", parse.count, 35.0 * n);
        assert_budget!("pgls_query::parse (bytes)", parse.bytes, 23_000.0 * n);
        assert_budget!("parsed statements (bytes kept)", parse.live(), 22_500.0 * n);
        drop(trees);

        // Row 6: the lexer.
        let (tokens, lex) = Allocations::during(|| pgls_lexer::lex(&dump.sql).len());
        assert_budget!(
            "pgls_lexer::lex (allocations)",
            lex.count,
            0.38 * tokens as f64
        );

        // Row 12: the line index.
        let (_, line_index) = Allocations::during(|| {
            pgls_line_index::LineIndex::new(&dump.sql)
                .line_starts()
                .len()
        });
        assert_budget!("LineIndex::new (allocations)", line_index.count, 20);

        let mut scale = Scale::new(Scale::without_database());

        // Row 5: opening the file.
        let open = scale.open(&dump.sql);
        assert_budget!("open_file (allocations)", open.count, 18.5 * n);

        // Rows 13 and 4: the first pull parses, walks and lints every statement; what it keeps
        // is the document's heap.
        let (_, cold) = scale.pull(DatabaseChecks::Reuse);
        assert_budget!("pull, cold (allocations)", cold.count, 255.0 * n);
        assert_budget!(
            "document heap after the first pull (bytes)",
            cold.live(),
            23_000.0 * n
        );

        // Row 17: a pull of the unchanged file only collects the diagnostics.
        let (diagnostics, unchanged) = scale.pull(DatabaseChecks::Reuse);
        assert_budget!(
            "pull, file unchanged (allocations)",
            unchanged.count,
            52.0 * diagnostics as f64 + 30.0
        );

        // Row 27: formatting the whole file.
        assert_budget!(
            "format, whole file (allocations)",
            scale.format().count,
            255.0 * n
        );

        // Rows 7, 8, 14 and 15: a keystroke at the top, in the middle and at the end, then the
        // pull that the language server sends after it.
        let offsets = dump.edit_offsets();
        assert_eq!(offsets.len(), 3);
        for (place, at) in ["top", "middle", "end"].into_iter().zip(offsets) {
            let mut edited = dump.sql.clone();
            edited.insert(at, ' ');
            let change = scale.change(edited);
            assert_budget!(
                format!("change_file at the {place} (allocations)"),
                change.count,
                150
            );
            assert_budget!(
                format!("change_file at the {place} (bytes)"),
                change.bytes,
                1.15 * dump.sql.len() as f64
            );
            let (_, pull) = scale.pull(DatabaseChecks::Reuse);
            let per_statement = if place == "top" { 240.0 } else { 180.0 };
            assert_budget!(
                format!("pull after a keystroke at the {place} (allocations)"),
                pull.count,
                per_statement * n
            );
            scale.change(dump.sql.clone());
            scale.pull(DatabaseChecks::Reuse);
        }
    }

    /// The budgets with the test database, on a dump of `statements` statements.
    async fn assert_database_within_budget(test_db: PgPool, statements: usize) {
        let dump = Dump::new(statements);
        let n = statements as f64;
        let functions = dump.functions();

        let plpgsql_check: bool = sqlx::query_scalar(
            "select exists (select from pg_available_extensions where name = 'plpgsql_check')",
        )
        .fetch_one(&test_db)
        .await
        .expect("read the available extensions");
        // CI's Windows runner has no plpgsql_check.
        assert!(
            plpgsql_check || cfg!(target_os = "windows"),
            "plpgsql_check is available"
        );

        // Row 21: without the extension installed, no function is checked. The test database
        // may come with it installed.
        if plpgsql_check {
            sqlx::query("drop extension if exists plpgsql_check")
                .execute(&test_db)
                .await
                .expect("drop plpgsql_check");
            let scale = Scale::new(Scale::with_database(&test_db));
            scale.open(&dump.sql);
            scale.pull(DatabaseChecks::Run);
            assert_eq!(
                scale.counters().plpgsql_checks.load(Ordering::Relaxed),
                0,
                "plpgsql_check is only available"
            );
            drop(scale);

            sqlx::query("create extension plpgsql_check")
                .execute(&test_db)
                .await
                .expect("create plpgsql_check");
        }
        let mut scale = Scale::new(Scale::with_database(&test_db));
        scale.open(&dump.sql);
        let queries = |scale: &Scale| {
            let counters = scale.counters();
            (
                counters.typecheck_queries.load(Ordering::Relaxed),
                counters.plpgsql_checks.load(Ordering::Relaxed),
            )
        };

        // Rows 19 and 20: a pull that runs the checks queries at most once per statement, and
        // parses each statement once with tree-sitter.
        let ((_, _), parses) = TreeParses::during(|| scale.pull(DatabaseChecks::Run));
        assert_budget!("tree-sitter parses, first pull with checks", parses, n);
        let (typechecks, checks) = queries(&scale);
        assert_budget!(
            "plpgsql_check, first pull (functions checked)",
            checks,
            functions
        );
        assert!(
            checks > 0 || !plpgsql_check,
            "plpgsql_check checks the functions"
        );
        assert_budget!(
            "typecheck, first pull (statements explained)",
            typechecks,
            n
        );
        let ((_, _), parses) = TreeParses::during(|| scale.pull(DatabaseChecks::Run));
        assert_budget!(
            "tree-sitter parses, pull with checks, file unchanged",
            parses,
            n
        );
        let (typechecks_after, checks_after) = queries(&scale);
        assert_budget!(
            "database queries, pull with checks, file unchanged",
            (typechecks_after - typechecks) + (checks_after - checks),
            functions as f64 + n
        );

        // The requests need the database's catalog: without one, they return at once.
        let last_query = dump.query(1.0).expect("the dump has queries");
        let hover_at = last_query
            + dump.sql[last_query..]
                .find("from public.t")
                .expect("a FROM")
            + 12;

        // Rows 22 and 23: the first hover walks the file up to the statement, a repeated one
        // doesn't.
        let (walked, hover) = scale.hover(hover_at);
        assert!(walked > 0, "the first hover walks");
        assert_budget!(
            "hover, first (allocations per walked statement)",
            hover.count,
            250.0 * walked as f64
        );
        let (_, hover) = scale.hover(hover_at);
        assert_budget!("hover, repeated (allocations)", hover.count, 170);

        // Row 26: semantic tokens of the whole file.
        assert_budget!(
            "semantic tokens, whole file (allocations)",
            scale.semantic_tokens().count,
            400.0 * n
        );

        // Rows 24 and 25: completions cost per table of the file and per object of the catalog:
        // about 280 and 12.1 allocations at checkpoint 0. Like there, they run after a hover,
        // which walked the file.
        let catalog_objects: i64 = sqlx::query_scalar(
            "select (select count(*) from pg_class where relkind in ('r', 'v', 'm', 'p', 'f'))
                  + (select count(*) from pg_attribute a join pg_class c on c.oid = a.attrelid
                     where a.attnum > 0 and not a.attisdropped and c.relkind in ('r', 'v', 'm', 'p', 'f'))
                  + (select count(*) from pg_proc)",
        )
        .fetch_one(&test_db)
        .await
        .expect("count the catalog");
        let completions = scale.completions(last_query + "select t.".len());
        assert_budget!(
            "completions (allocations)",
            completions.count,
            1.05 * (280.0 * dump.tables() as f64 + 13.0 * catalog_objects as f64)
        );

        // Rows 18 and 20: the language server's pull after a keystroke reuses the checks: no
        // query, no tree-sitter parse.
        let at_end = *dump.edit_offsets().last().expect("the dump has queries");
        let mut edited = dump.sql.clone();
        edited.insert(at_end, ' ');
        scale.change(edited);
        let before = queries(&scale);
        let ((_, pull), parses) = TreeParses::during(|| scale.pull(DatabaseChecks::Reuse));
        assert_eq!(
            queries(&scale),
            before,
            "a pull that reuses the checks queries"
        );
        assert_eq!(
            parses, 0,
            "a pull that reuses the checks parses with tree-sitter"
        );

        // Row 16: that pull lints with the typecheck rules on the catalog.
        assert_budget!(
            "pull after a keystroke, with a database (allocations)",
            pull.count,
            405.0 * n
        );
    }
}

#[test]
fn scale_1k_statements_within_budget() {
    Scale::assert_within_budget(1_000);
}

#[test]
#[ignore = "slow in debug; CI's Scale job runs it in release"]
fn scale_20k_statements_within_budget() {
    Scale::assert_within_budget(20_000);
}

#[pgls_test_utils::db_test]
async fn scale_1k_database_checks_within_budget(test_db: PgPool) {
    Scale::assert_database_within_budget(test_db, 1_000).await;
}

#[pgls_test_utils::db_test]
#[ignore = "slow in debug; CI's Scale job runs it in release"]
async fn scale_20k_database_checks_within_budget(test_db: PgPool) {
    Scale::assert_database_within_budget(test_db, 20_000).await;
}
