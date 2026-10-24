//! Memory of the workspace's caches while editor features run on many large documents, measured
//! with a counting allocator. Each test binary has its own allocator, so this one only counts
//! what this test allocates.

mod common;

use std::sync::atomic::Ordering;

use common::counting_allocator::{ALLOCATED, Counting, PEAK, mib};

use pgls_configuration::{Merge, PartialConfiguration, database::PartialDatabaseConfiguration};
use pgls_fs::PgLSPath;
use pgls_text_size::{TextRange, TextSize};
use pgls_workspace::{
    features::{
        code_actions::CodeActionsParams, diagnostics::PullFileDiagnosticsParams,
        inlay_hints::PullInlayHintsParams, on_hover::OnHoverParams,
        semantic_tokens::PullSemanticTokensParams,
    },
    workspace::{
        CloseFileParams, OpenFileParams, RegisterProjectFolderParams, UpdateSettingsParams,
    },
};
use sqlx::PgPool;

#[global_allocator]
static GLOBAL: Counting = Counting;

/// A migration-style file: per group a table, an index, a SQL function with its body, and a
/// query, so five statements.
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

/// What the editor features may keep for all documents together.
const EDITOR_FEATURES_MIB_MAX: f64 = 128.0;

/// The peak on top of the baseline, for the default sizes.
const PEAK_MIB_MAX: f64 = 1024.0;

/// What may stay allocated after all documents are closed.
const RETAINED_MIB_MAX: f64 = 16.0;

/// The number of documents and their statements. `MEMORY_DOCUMENTS` and `MEMORY_STATEMENTS`
/// override them.
fn sizes() -> (usize, usize) {
    let documents = std::env::var("MEMORY_DOCUMENTS").map_or(20, |n| n.parse().unwrap());
    let statements = std::env::var("MEMORY_STATEMENTS").map_or(1_000, |n| n.parse().unwrap());
    (documents, statements)
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn editor_features_on_many_documents_stay_within_budget(test_db: PgPool) {
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
    let workspace = pgls_workspace::workspace::server_sync();
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

    // Hover iterates the document with a `ParseIterator`, which recurses once per statement on
    // this branch (fixed on main), so run on a thread with a large stack.
    std::thread::scope(|scope| {
        std::thread::Builder::new()
            .stack_size(1 << 30)
            .spawn_scoped(scope, || {
                let (documents, statements) = sizes();
                let content = migration_file(statements / 5);
                let len = TextSize::of(content.as_str());
                let last = TextSize::try_from(content.rfind("select").unwrap()).unwrap()
                    + TextSize::from(8);
                let step = |label: &str| {
                    if std::env::var("MEMORY_STEPS").is_ok() {
                        println!(
                            "  {label}: {:.1} MiB",
                            mib(ALLOCATED.load(Ordering::Relaxed))
                        );
                    }
                };
                let run = |path: &PgLSPath| {
                    step("open");
                    workspace
                        .pull_file_diagnostics(PullFileDiagnosticsParams {
                            path: path.clone(),
                            categories: pgls_analyse::RuleCategories::all(),
                            max_diagnostics: 100,
                            only: vec![],
                            skip: vec![],
                        })
                        .unwrap();
                    step("diagnostics");
                    let after_diagnostics = ALLOCATED.load(Ordering::Relaxed);
                    workspace
                        .on_hover(OnHoverParams {
                            path: path.clone(),
                            position: last,
                        })
                        .unwrap();
                    step("hover");
                    workspace
                        .pull_inlay_hints(PullInlayHintsParams {
                            path: path.clone(),
                            range: TextRange::up_to(len),
                        })
                        .unwrap();
                    step("inlay hints");
                    workspace
                        .pull_semantic_tokens(PullSemanticTokensParams {
                            path: path.clone(),
                            range: None,
                        })
                        .unwrap();
                    step("semantic tokens");
                    workspace
                        .pull_code_actions(CodeActionsParams {
                            path: path.clone(),
                            cursor_position: last,
                            only: vec![],
                            skip: vec![],
                        })
                        .unwrap();
                    step("code actions");
                    // What hover, inlay hints, semantic tokens and code actions keep on top of
                    // what the diagnostics keep.
                    ALLOCATED
                        .load(Ordering::Relaxed)
                        .saturating_sub(after_diagnostics)
                };

                // Load the snapshot and warm everything that lives as long as the workspace.
                let warmup = PgLSPath::new("warmup.sql");
                workspace
                    .open_file(OpenFileParams {
                        path: warmup.clone(),
                        content: migration_file(2),
                        version: 1,
                    })
                    .unwrap();
                let _ = run(&warmup);
                workspace
                    .close_file(CloseFileParams { path: warmup })
                    .unwrap();

                let baseline = ALLOCATED.load(Ordering::Relaxed);
                PEAK.store(baseline, Ordering::Relaxed);
                let mut editor = 0;
                let paths: Vec<PgLSPath> = (0..documents)
                    .map(|i| PgLSPath::new(format!("doc{i}.sql")))
                    .collect();
                for path in &paths {
                    workspace
                        .open_file(OpenFileParams {
                            path: path.clone(),
                            content: content.clone(),
                            version: 1,
                        })
                        .unwrap();
                    editor += run(path);
                }
                let open = ALLOCATED.load(Ordering::Relaxed).saturating_sub(baseline);
                let peak = PEAK.load(Ordering::Relaxed).saturating_sub(baseline);
                for path in paths {
                    workspace.close_file(CloseFileParams { path }).unwrap();
                }
                let retained = ALLOCATED.load(Ordering::Relaxed).saturating_sub(baseline);
                println!(
                    "{documents} documents x {statements} statements ({:.1} MiB of text): \
                     peak {:.1} MiB, with all open {:.1} MiB (editor features {:.1} MiB), \
                     after closing {:.1} MiB",
                    mib(content.len() * documents),
                    mib(peak),
                    mib(open),
                    mib(editor),
                    mib(retained)
                );
                // The cached walks of the editor features are bounded by their statement and
                // checkpoint budgets (about 100 MiB), whatever the number of documents.
                assert!(
                    mib(editor) < EDITOR_FEATURES_MIB_MAX,
                    "{:.1} MiB",
                    mib(editor)
                );
                // The statement caches of each document are bounded per document on this
                // branch, and across documents by #913 on main.
                assert!(mib(peak) < PEAK_MIB_MAX, "{:.1} MiB", mib(peak));
                // Closing the documents drops their caches. What's left is the database
                // snapshot that hover built for the last statement it hovered.
                assert!(mib(retained) < RETAINED_MIB_MAX, "{:.1} MiB", mib(retained));
            })
            .unwrap()
            .join()
            .unwrap();
    });
}
