//! Memory of the editor walk cache over a long editing session: many edits of one document,
//! with hover, inlay hints and semantic tokens requested in between, some of them on versions
//! that are outdated by the time they finish. A leak in the cache (checkpoints or references of
//! earlier versions, document snapshots kept alive by stale requests) would make the memory grow
//! with the number of edits.

mod common;

use std::sync::atomic::Ordering;

use common::counting_allocator::{ALLOCATED, Counting, mib};
use pgls_configuration::{Merge, PartialConfiguration, database::PartialDatabaseConfiguration};
use pgls_fs::PgLSPath;
use pgls_text_size::{TextRange, TextSize};
use pgls_workspace::{
    Workspace,
    features::{
        inlay_hints::PullInlayHintsParams, on_hover::OnHoverParams,
        semantic_tokens::PullSemanticTokensParams,
    },
    workspace::{
        ChangeFileParams, OpenFileParams, RegisterProjectFolderParams, UpdateSettingsParams,
        server::{EditorWalkSize, WorkspaceServer},
    },
};
use sqlx::PgPool;

#[global_allocator]
static GLOBAL: Counting = Counting;

/// Retained memory after the edits may be this many times the memory retained after the first
/// pass over the document. The document changes size with the edits, and so do its statement
/// caches, so it isn't exactly the same.
const RETAINED_GROWTH_MAX: f64 = 3.0;

/// The bounds of a document's walk cache: 16 checkpoints, and stale statements are pruned once
/// there are more than twice the document's statements and 1,024.
fn assert_within_bounds(size: EditorWalkSize) {
    assert!(size.checkpoints <= 16, "{size:?}");
    assert!(
        size.statements <= 2 * size.document_statements + 1024,
        "{size:?}"
    );
}

/// A tiny deterministic random number generator, so that every run makes the same edits.
struct Random(u64);

impl Random {
    fn below(&mut self, n: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        usize::try_from(self.0 >> 33).unwrap() % n.max(1)
    }
}

/// The statements of a group: a table, a SQL function with its body, and a query.
fn group(i: usize) -> [String; 3] {
    [
        format!("create table t{i} (id bigint primary key, name text, status text);\n"),
        format!(
            "create function f{i}(a int, b text) returns int language sql as $$ select count(*)::int from t{i} where id = a and name = b $$;\n"
        ),
        format!("select t.id, t.name, f{i}(1, 'x') from t{i} t where t.status = 'active';\n"),
    ]
}

/// One edit at a random position: rewriting a query, adding a table and the statements that use
/// it, deleting a statement, altering or dropping a table, or changing a function body.
/// Deletes keep the document at about 1,000 top-level statements.
fn edit(statements: &mut Vec<String>, groups: &mut usize, random: &mut Random) {
    let at = random.below(statements.len());
    let table = random.below(*groups);
    let kind = if statements.len() > 1_000 {
        2
    } else {
        random.below(6)
    };
    match kind {
        0 => {
            statements[at] = format!(
                "select t.name, f{table}({}, t.name) from t{table} t where t.id = {};\n",
                random.below(100),
                random.below(100)
            );
        }
        1 => {
            for (offset, statement) in group(*groups).into_iter().enumerate() {
                statements.insert(at + offset, statement);
            }
            *groups += 1;
        }
        2 if statements.len() > 700 => {
            let end = (at + 3).min(statements.len());
            statements.drain(at..end);
        }
        3 => statements.insert(
            at,
            format!(
                "alter table t{table} add column c{} int;\n",
                random.below(1_000)
            ),
        ),
        4 => {
            statements[at] = format!(
                "create function f{table}(a int, b text) returns int language sql as $$ select a + {} from t{table} where name = b $$;\n",
                random.below(1_000)
            );
        }
        _ => statements.insert(at, format!("drop table if exists t{table};\n")),
    }
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn editing_a_document_doesnt_grow_the_walk_cache(test_db: PgPool) {
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
    let edits: usize = std::env::var("MEMORY_EDITS").map_or(1_000, |n| n.parse().unwrap());

    // Hover iterates the document with a `ParseIterator`, which recurses once per statement on
    // this branch (fixed on main), so run on a thread with a large stack.
    std::thread::scope(|scope| {
        std::thread::Builder::new()
            .stack_size(1 << 30)
            .spawn_scoped(scope, || {
                let path = PgLSPath::new("churn.sql");
                let mut random = Random(42);
                let mut groups = 250;
                let mut statements: Vec<String> = (0..groups).flat_map(group).collect();
                // What an editor asks for while the user types: the tokens and hints of the
                // visible range. Every `full` time also the tokens of the whole document and a
                // hover.
                let requests = |content: &str, random: &mut Random, full: bool| {
                    let len = TextSize::of(content);
                    let window = |random: &mut Random| {
                        let start = TextSize::try_from(random.below(content.len())).unwrap();
                        TextRange::new(start, (start + TextSize::from(4_000)).min(len))
                    };
                    if full {
                        workspace
                            .pull_semantic_tokens(PullSemanticTokensParams {
                                path: path.clone(),
                                range: None,
                            })
                            .unwrap();
                        workspace
                            .on_hover(OnHoverParams {
                                path: path.clone(),
                                position: TextSize::try_from(random.below(content.len())).unwrap(),
                            })
                            .unwrap();
                    }
                    workspace
                        .pull_semantic_tokens(PullSemanticTokensParams {
                            path: path.clone(),
                            range: Some(window(random)),
                        })
                        .unwrap();
                    workspace
                        .pull_inlay_hints(PullInlayHintsParams {
                            path: path.clone(),
                            range: window(random),
                        })
                        .unwrap();
                };

                // Load the snapshot and warm what lives as long as the workspace.
                let warmup = PgLSPath::new("warmup.sql");
                workspace
                    .open_file(OpenFileParams {
                        path: warmup.clone(),
                        content: group(0).concat(),
                        version: 1,
                    })
                    .unwrap();
                workspace
                    .pull_semantic_tokens(PullSemanticTokensParams {
                        path: warmup.clone(),
                        range: None,
                    })
                    .unwrap();
                workspace
                    .close_file(pgls_workspace::workspace::CloseFileParams { path: warmup })
                    .unwrap();
                let baseline = ALLOCATED.load(Ordering::Relaxed);

                let content = statements.concat();
                workspace
                    .open_file(OpenFileParams {
                        path: path.clone(),
                        content: content.clone(),
                        version: 1,
                    })
                    .unwrap();
                requests(&content, &mut random, true);
                let first_pass = ALLOCATED.load(Ordering::Relaxed).saturating_sub(baseline);
                let first_size = workspace.editor_walk_size(&path).unwrap();

                let mut largest = first_size;
                for version in 2..i32::try_from(edits).unwrap() + 2 {
                    edit(&mut statements, &mut groups, &mut random);
                    let content = statements.concat();
                    if version % 100 == 0 {
                        // A request that the next edit makes outdated while it runs.
                        std::thread::scope(|scope| {
                            scope.spawn(|| {
                                workspace
                                    .pull_semantic_tokens(PullSemanticTokensParams {
                                        path: path.clone(),
                                        range: None,
                                    })
                                    .unwrap()
                            });
                            workspace
                                .change_file(ChangeFileParams {
                                    path: path.clone(),
                                    version,
                                    content: content.clone(),
                                })
                                .unwrap();
                        });
                        continue;
                    }
                    workspace
                        .change_file(ChangeFileParams {
                            path: path.clone(),
                            version,
                            content: content.clone(),
                        })
                        .unwrap();
                    if version % 10 == 0 {
                        requests(&content, &mut random, version % 100 == 50);
                        let size = workspace.editor_walk_size(&path).unwrap();
                        assert_within_bounds(size);
                        if size.statements > largest.statements {
                            largest = size;
                        }
                    }
                }

                let content = statements.concat();
                requests(&content, &mut random, true);
                let retained = ALLOCATED.load(Ordering::Relaxed).saturating_sub(baseline);
                let size = workspace.editor_walk_size(&path).unwrap();
                println!(
                    "{edits} edits of a document of {} statements ({:.1} KiB of text, {} after \
                     the edits): retained {:.1} MiB after the first pass, {:.1} MiB after the \
                     edits; walk cache after the first pass {first_size:?}, largest \
                     {largest:?}, after the edits {size:?}",
                    first_size.document_statements,
                    content.len() as f64 / 1024.0,
                    size.document_statements,
                    mib(first_pass),
                    mib(retained),
                );
                assert_within_bounds(size);
                // The full token request at the end walked the whole document, so every one of
                // its statements is cached.
                assert_eq!(
                    size.current_statements, size.document_statements,
                    "{size:?}"
                );
                assert!(
                    (retained as f64) < first_pass as f64 * RETAINED_GROWTH_MAX,
                    "retained {:.1} MiB after the edits, {:.1} MiB after the first pass",
                    mib(retained),
                    mib(first_pass)
                );
            })
            .unwrap()
            .join()
            .unwrap();
    });
}
