//! The memory that the workspace keeps for open documents.
//!
//! Counts every allocation of the test binary with the allocator in `common`, so the test runs
//! on its own in this file. `MEMORY_DOCUMENTS` and `MEMORY_GROUPS` (four statements and one SQL function
//! body each) change the size of the workload, e.g. `MEMORY_DOCUMENTS=20 MEMORY_GROUPS=1250` for
//! 20 documents with 5,000 statements each. Run it in a release build for numbers that match
//! what users see.

mod common;

use common::{MIB, current, mib, peak, reset_peak};
use pgls_analyse::RuleCategories;
use pgls_configuration::{Merge, PartialConfiguration, database::PartialDatabaseConfiguration};
use pgls_fs::PgLSPath;
use pgls_text_size::{TextRange, TextSize};
use pgls_workspace::{
    Workspace,
    features::{
        code_actions::CodeActionsParams, diagnostics::PullFileDiagnosticsParams,
        inlay_hints::PullInlayHintsParams, on_hover::OnHoverParams,
    },
    workspace::server::DOCUMENT_CACHE_ENTRIES_MAX,
    workspace::{
        CloseFileParams, OpenFileParams, RegisterProjectFolderParams, UpdateSettingsParams,
    },
};
use sqlx::PgPool;

fn env(name: &str, default: usize) -> usize {
    std::env::var(name).map_or(default, |value| value.parse().unwrap())
}

/// A migration-style file: per group a table, an index, a SQL function and a query.
fn migration_file(document: usize, groups: usize) -> String {
    let mut sql = String::new();
    for i in 0..groups {
        sql.push_str(&format!(
            "create table t{document}_{i} (id bigint primary key, name text not null, status text, created_at timestamp default now());\n\
             create index t{document}_{i}_name_idx on t{document}_{i} (name);\n\
             create function f{document}_{i}(a int, b text) returns int language sql stable as $$ select count(*)::int from t{document}_{i} where id = a and name = b $$;\n\
             select t.id, t.name, f{document}_{i}(1, 'x') from t{document}_{i} t where t.status = 'active' order by t.created_at;\n"
        ));
    }
    sql
}

fn workspace(test_db: &PgPool) -> Box<dyn Workspace> {
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
    let workspace = pgls_workspace::workspace::server();
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

/// What an editor asks for on a document: diagnostics, then hover, inlay hints and code
/// actions at the end of the file.
fn use_document(workspace: &dyn Workspace, path: &PgLSPath, content: &str) {
    workspace
        .pull_file_diagnostics(PullFileDiagnosticsParams {
            path: path.clone(),
            categories: RuleCategories::all(),
            max_diagnostics: u32::MAX,
            only: vec![],
            skip: vec![],
        })
        .unwrap();
    let last = TextSize::try_from(content.rfind("select t.id").unwrap()).unwrap();
    workspace
        .on_hover(OnHoverParams {
            path: path.clone(),
            position: last + TextSize::from(9),
        })
        .unwrap();
    let len = TextSize::of(content);
    let start = TextSize::try_from(content.len().saturating_sub(4_000)).unwrap();
    workspace
        .pull_inlay_hints(PullInlayHintsParams {
            path: path.clone(),
            range: TextRange::new(start, len),
        })
        .unwrap();
    workspace
        .pull_code_actions(CodeActionsParams {
            path: path.clone(),
            cursor_position: last + TextSize::from(9),
            only: vec![],
            skip: vec![],
        })
        .unwrap();
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn open_documents_stay_within_the_budget(test_db: PgPool) {
    let documents = env("MEMORY_DOCUMENTS", 20);
    let groups = env("MEMORY_GROUPS", 200);
    let workspace = workspace(&test_db);

    // Loads the database snapshot, which stays for the lifetime of the workspace.
    let warmup = PgLSPath::new("warmup.sql");
    let content = migration_file(usize::MAX, 1);
    workspace
        .open_file(OpenFileParams {
            path: warmup.clone(),
            content: content.clone(),
            version: 1,
        })
        .unwrap();
    use_document(&*workspace, &warmup, &content);
    workspace
        .close_file(CloseFileParams { path: warmup })
        .unwrap();

    let contents: Vec<_> = (0..documents)
        .map(|document| migration_file(document, groups))
        .collect();
    let text: usize = contents.iter().map(String::len).sum();
    let baseline = reset_peak();

    // An editor with every document open.
    let mut first = (0, 0);
    for (document, content) in contents.iter().enumerate() {
        let path = PgLSPath::new(format!("{document}.sql"));
        workspace
            .open_file(OpenFileParams {
                path: path.clone(),
                content: content.clone(),
                version: 1,
            })
            .unwrap();
        use_document(&*workspace, &path, content);
        if document == 0 {
            first = (peak() - baseline, current() - baseline);
        }
    }
    let open_peak = peak() - baseline;
    let open_retained = current() - baseline;
    for document in 0..documents {
        workspace
            .close_file(CloseFileParams {
                path: PgLSPath::new(format!("{document}.sql")),
            })
            .unwrap();
    }
    let closed_retained = current().saturating_sub(baseline);

    // The CLI: each file is opened, checked and closed.
    let reset = reset_peak();
    let cli_started = std::time::Instant::now();
    for (document, content) in contents.iter().enumerate() {
        let path = PgLSPath::new(format!("cli_{document}.sql"));
        workspace
            .open_file(OpenFileParams {
                path: path.clone(),
                content: content.clone(),
                version: 1,
            })
            .unwrap();
        workspace
            .pull_file_diagnostics(PullFileDiagnosticsParams {
                path: path.clone(),
                categories: RuleCategories::all(),
                max_diagnostics: u32::MAX,
                only: vec![],
                skip: vec![],
            })
            .unwrap();
        workspace.close_file(CloseFileParams { path }).unwrap();
    }
    let cli_duration = cli_started.elapsed();
    let cli_peak = peak().saturating_sub(reset);
    let cli_retained = current().saturating_sub(baseline);

    println!(
        "{documents} documents, {:.1} MiB of SQL: one document peak {:.1} MiB, retained {:.1} MiB; \
         all open: peak {:.1} MiB, retained {:.1} MiB; retained after closing {:.1} MiB; \
         CLI peak {:.1} MiB, retained {:.1} MiB, {cli_duration:.1?}",
        mib(text),
        mib(first.0),
        mib(first.1),
        mib(open_peak),
        mib(open_retained),
        mib(closed_retained),
        mib(cli_peak),
        mib(cli_retained),
    );

    // Every statement has a parse result and a tree, and so does the body of each function.
    let entries_per_document = 2 * 5 * groups;
    let cached_documents = DOCUMENT_CACHE_ENTRIES_MAX / entries_per_document + 1;
    // A document whose caches were dropped still has its text, statement ranges and
    // suppressions.
    let uncached_document = 4 * text / documents;
    let budget =
        (cached_documents.min(documents) * first.1 + documents * uncached_document) * 5 / 4;
    assert!(
        open_retained <= budget,
        "{:.1} MiB retained, budget {:.1} MiB",
        mib(open_retained),
        mib(budget)
    );
    assert!(
        open_peak <= budget + first.0,
        "{:.1} MiB peak, budget {:.1} MiB",
        mib(open_peak),
        mib(budget + first.0)
    );
    // Closing the documents drops everything derived from them.
    assert!(
        closed_retained <= 4 * MIB,
        "{:.1} MiB retained after closing every document",
        mib(closed_retained)
    );
    // The CLI only ever has one file open.
    assert!(
        cli_peak <= 2 * first.0,
        "{:.1} MiB peak for the CLI, {:.1} MiB for one document",
        mib(cli_peak),
        mib(first.0)
    );
    assert!(
        cli_retained <= 4 * MIB,
        "{:.1} MiB retained after the CLI run",
        mib(cli_retained)
    );
}
