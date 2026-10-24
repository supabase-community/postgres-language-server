//! Timings of the resolver-based editor features on large files.
//!
//! Run with
//! `cargo test --release -p pgls_workspace --lib perf_tests -- --ignored --nocapture --test-threads=1`.
//! `PERF_GROUPS` sets the sizes of the migration files (4 top-level statements and a function
//! body per group), `PERF_CALLS` the calls of the huge statement, and `PERF_SEED_ROWS` the rows
//! of the seed `INSERT`s.

use std::{sync::Arc, time::Instant};

use pgls_configuration::{Merge, PartialConfiguration, database::PartialDatabaseConfiguration};
use pgls_fs::PgLSPath;
use pgls_text_size::{TextRange, TextSize};
use sqlx::PgPool;

use super::document::AnalyserDiagnosticsMapper;
use crate::{
    Workspace,
    features::code_actions::CodeActionsParams,
    features::inlay_hints::PullInlayHintsParams,
    features::on_hover::OnHoverParams,
    features::semantic_tokens::PullSemanticTokensParams,
    workspace::{
        OpenFileParams, RegisterProjectFolderParams, UpdateSettingsParams, server::WorkspaceServer,
    },
};

fn workspace(test_db: &PgPool) -> WorkspaceServer {
    let mut conf = PartialConfiguration::init();
    conf.merge_with(PartialConfiguration {
        db: Some(PartialDatabaseConfiguration {
            database: Some(
                test_db
                    .connect_options()
                    .get_database()
                    .unwrap()
                    .to_string(),
            ),
            ..Default::default()
        }),
        ..Default::default()
    });
    let workspace = WorkspaceServer::new();
    workspace
        .register_project_folder(RegisterProjectFolderParams {
            path: None,
            set_as_current_workspace: true,
        })
        .unwrap();
    workspace
        .update_settings(UpdateSettingsParams {
            configuration: conf,
            gitignore_matches: vec![],
            vcs_base_path: None,
            workspace_directory: None,
        })
        .unwrap();
    workspace
}

/// A migration-style file: per group a table, an index, a SQL function and a query.
fn migration_file(groups: usize) -> String {
    let mut sql = String::new();
    for i in 0..groups {
        sql.push_str(&format!(
            "create table t{i} (id bigint primary key, name text not null, status text, created_at timestamptz default now());\n\
             create index t{i}_name_idx on t{i} (name);\n\
             create function f{i}(a int, b text) returns int language sql stable as $$ select count(*)::int from t{i} where id = a and name = b $$;\n\
             select t.id, t.name, f{i}(1, 'x') from t{i} t where t.status = 'active' order by t.created_at;\n"
        ));
    }
    sql
}

/// One huge statement: a select with many function calls.
fn huge_statement(calls: usize) -> String {
    let mut sql = String::from("create table big (id int, a int, b text);\nselect\n");
    for i in 0..calls {
        if i > 0 {
            sql.push_str(",\n");
        }
        sql.push_str(&format!("  coalesce(a, {i}) + length(b) + abs(id) as c{i}"));
    }
    sql.push_str("\nfrom big;\n");
    sql
}

/// A seed file: one INSERT with many rows, each with an enum literal and a function call.
fn seed_insert(rows: usize) -> String {
    let mut sql = String::from(
        "create type seed_status as enum ('active', 'inactive');\n\
         create table seed (id int, name text, status seed_status, created_at timestamptz);\n\
         insert into seed (id, name, status, created_at) values\n",
    );
    for i in 0..rows {
        if i > 0 {
            sql.push_str(",\n");
        }
        sql.push_str(&format!("  ({i}, 'name {i}', 'active', now())"));
    }
    sql.push_str(";\nselect name from seed;\n");
    sql
}

fn ms(start: Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1000.0
}

/// The fastest of three runs.
fn time<T>(f: impl Fn() -> T) -> (f64, T) {
    let mut best = f64::MAX;
    let mut result = None;
    for _ in 0..3 {
        let start = Instant::now();
        result = Some(f());
        best = best.min(ms(start));
    }
    (best, result.unwrap())
}

/// The fastest of three runs that each start without a cached walk, and the fastest of three
/// repeated runs.
fn time_cold_warm<T>(workspace: &WorkspaceServer, f: impl Fn() -> T) -> (f64, f64, T) {
    let (cold, _) = time(|| {
        workspace.editor_walks.clear();
        f()
    });
    f();
    let (warm, result) = time(&f);
    (cold, warm, result)
}

fn time_once<T>(f: impl FnOnce() -> T) -> (f64, T) {
    let start = Instant::now();
    let result = f();
    (ms(start), result)
}

fn measure(workspace: &WorkspaceServer, name: &str, content: &str) {
    let path = PgLSPath::new(format!("{name}.sql"));
    let (open, ()) = time_once(|| {
        workspace
            .open_file(OpenFileParams {
                path: path.clone(),
                content: content.into(),
                version: 1,
            })
            .unwrap()
    });
    let statements = {
        let documents = workspace.documents.read().unwrap();
        documents
            .get(&path)
            .unwrap()
            .iter(AnalyserDiagnosticsMapper)
            .count()
    };
    println!(
        "\n== {name}: {} bytes, {} lines, {statements} statements (incl. function bodies); open {open:.1} ms",
        content.len(),
        content.lines().count()
    );

    // The parse/mapper cost alone: iterate the document twice (the second is "warm").
    let (t, _) = time(|| {
        let documents = workspace.documents.read().unwrap();
        documents
            .get(&path)
            .unwrap()
            .iter(AnalyserDiagnosticsMapper)
            .count()
    });
    println!(
        "  {:<40} {t:>9.1} ms",
        "iterate document (AnalyserDiagnosticsMapper)"
    );

    let snapshot = {
        let settings = workspace.workspaces();
        let settings = settings.settings().unwrap();
        workspace
            .connection
            .with_pool(&settings.db, |pool| workspace.snapshot.load(pool))
            .unwrap()
            .unwrap()
    };
    // Positions: the first and the last query of the file.
    let first = TextSize::try_from(content.find("select").unwrap()).unwrap();
    let last = TextSize::try_from(content.rfind("select").unwrap()).unwrap();
    // A visible range of ~60 lines around a position.
    let visible = |at: TextSize| {
        let start = usize::from(at);
        let end = content[start..]
            .char_indices()
            .filter(|(_, c)| *c == '\n')
            .nth(60)
            .map_or(content.len(), |(i, _)| start + i);
        TextRange::new(at, TextSize::try_from(end).unwrap())
    };

    for (label, at) in [("start", first), ("end", last)] {
        {
            let (cold, warm, hints) = time_cold_warm(workspace, || {
                workspace
                    .pull_inlay_hints(PullInlayHintsParams {
                        path: path.clone(),
                        range: visible(at),
                    })
                    .unwrap()
                    .hints
                    .len()
            });
            println!(
                "  {:<40} {cold:>9.1} ms  cold {warm:>9.1} ms  warm ({hints} hints)",
                format!("inlay hints (60 lines) @{label}")
            );
        }
        let (cold, warm, blocks) = time_cold_warm(workspace, || {
            workspace
                .on_hover(OnHoverParams {
                    path: path.clone(),
                    position: at + TextSize::from(8),
                })
                .unwrap()
                .markdown_blocks
                .len()
        });
        println!(
            "  {:<40} {cold:>9.1} ms  cold {warm:>9.1} ms  warm ({blocks} blocks)",
            format!("hover @{label}")
        );
        // The parts of hover: the resolved references, and the materialized snapshot.
        let (cold, warm, _) = time_cold_warm(workspace, || {
            let documents = workspace.documents.read().unwrap();
            let doc = documents.get(&path).unwrap();
            workspace
                .references_at(&path, doc, &snapshot, at + TextSize::from(8))
                .len()
        });
        println!(
            "  {:<40} {cold:>9.1} ms  cold {warm:>9.1} ms  warm",
            format!("  hover: references_at @{label}")
        );
        let (t, _) = time(|| {
            let documents = workspace.documents.read().unwrap();
            let doc = documents.get(&path).unwrap();
            // Clear the one-entry cache, as hovering another statement would.
            *workspace.snapshot_before.lock().unwrap() = None;
            workspace.snapshot_before(doc, at, Arc::clone(&snapshot));
        });
        println!(
            "  {:<40} {t:>9.1} ms",
            format!("  hover: snapshot_before (miss) @{label}")
        );
        let (cold, warm, tokens) = time_cold_warm(workspace, || {
            workspace
                .pull_semantic_tokens(PullSemanticTokensParams {
                    path: path.clone(),
                    range: Some(visible(at)),
                })
                .unwrap()
                .tokens
                .len()
        });
        println!(
            "  {:<40} {cold:>9.1} ms  cold {warm:>9.1} ms  warm ({tokens} tokens)",
            format!("semantic tokens range (60 lines) @{label}")
        );
        let (t, actions) = time(|| {
            workspace
                .pull_code_actions(CodeActionsParams {
                    path: path.clone(),
                    cursor_position: at + TextSize::from(8),
                    only: vec![],
                    skip: vec![],
                })
                .unwrap()
                .actions
                .len()
        });
        println!(
            "  {:<40} {t:>9.1} ms  ({actions} actions)",
            format!("code actions @{label}")
        );
    }
    {
        let (cold, warm, tokens) = time_cold_warm(workspace, || {
            workspace
                .pull_semantic_tokens(PullSemanticTokensParams {
                    path: path.clone(),
                    range: None,
                })
                .unwrap()
                .tokens
                .len()
        });
        println!(
            "  {:<40} {cold:>9.1} ms  cold {warm:>9.1} ms  warm ({tokens} tokens)",
            "semantic tokens full"
        );
    }

    // Typing in the last statement: each edit changes it, and the editor asks for the tokens and
    // hints of the visible range.
    let mut edits = Vec::new();
    for version in 2..5 {
        let mut edited = content.to_string();
        edited.insert_str(
            usize::from(last) + 6,
            &" ".repeat(usize::try_from(version).unwrap()),
        );
        workspace
            .change_file(crate::workspace::ChangeFileParams {
                path: path.clone(),
                version,
                content: edited,
            })
            .unwrap();
        let (t, _) = time_once(|| {
            workspace
                .pull_semantic_tokens(PullSemanticTokensParams {
                    path: path.clone(),
                    range: Some(visible(last)),
                })
                .unwrap();
            workspace
                .pull_inlay_hints(PullInlayHintsParams {
                    path: path.clone(),
                    range: visible(last),
                })
                .unwrap();
        });
        edits.push(t);
    }
    let best = edits.iter().copied().fold(f64::MAX, f64::min);
    println!(
        "  {:<40} {best:>9.1} ms",
        "edit @end, then tokens + hints @end"
    );

    // An edit while a request walks the whole file.
    workspace.editor_walks.clear();
    let (request, change) = std::thread::scope(|scope| {
        let request = scope.spawn(|| {
            time_once(|| {
                workspace
                    .pull_semantic_tokens(PullSemanticTokensParams {
                        path: path.clone(),
                        range: None,
                    })
                    .unwrap()
            })
            .0
        });
        std::thread::sleep(std::time::Duration::from_millis(5));
        let (change, ()) = time_once(|| {
            workspace
                .change_file(crate::workspace::ChangeFileParams {
                    path: path.clone(),
                    version: 10,
                    content: content.to_string(),
                })
                .unwrap()
        });
        (request.join().unwrap(), change)
    });
    println!(
        "  {:<40} {change:>9.1} ms  (the request took {request:.1} ms)",
        "edit during a full token request"
    );
}

#[ignore = "timing benchmark"]
#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn perf_resolver_features(test_db: PgPool) {
    let workspace = workspace(&test_db);
    // Load the snapshot once, so that it doesn't count in the first request.
    let path = PgLSPath::new("warmup.sql");
    workspace
        .open_file(OpenFileParams {
            path: path.clone(),
            content: "select 1;".into(),
            version: 1,
        })
        .unwrap();
    workspace
        .pull_inlay_hints(PullInlayHintsParams {
            path: path.clone(),
            range: TextRange::up_to(9.into()),
        })
        .unwrap();
    workspace
        .on_hover(OnHoverParams {
            path,
            position: 2.into(),
        })
        .unwrap();

    let groups: Vec<usize> = std::env::var("PERF_GROUPS")
        .ok()
        .map(|groups| groups.split(',').map(|n| n.parse().unwrap()).collect())
        .unwrap_or_else(|| vec![250, 1000, 4000]);
    let calls: usize = std::env::var("PERF_CALLS").map_or(3000, |n| n.parse().unwrap());
    // Hover iterates the document with `ParseIterator`, which recurses once per statement, so
    // measure on a thread with a large stack.
    std::thread::scope(|scope| {
        std::thread::Builder::new()
            .stack_size(1 << 30)
            .spawn_scoped(scope, || {
                for groups in groups {
                    measure(
                        &workspace,
                        &format!("migration_{groups}"),
                        &migration_file(groups),
                    );
                }
                measure(&workspace, &format!("huge_{calls}"), &huge_statement(calls));
                if let Ok(rows) = std::env::var("PERF_SEED_ROWS") {
                    for rows in rows.split(',') {
                        let rows: usize = rows.parse().unwrap();
                        measure(&workspace, &format!("seed_{rows}"), &seed_insert(rows));
                    }
                }
            })
            .unwrap()
            .join()
            .unwrap();
    });
}
