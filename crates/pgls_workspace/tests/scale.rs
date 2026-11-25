//! What the requests on a file cost grows linearly with its number of statements.
//!
//! Counts the allocations and allocated bytes of each request at `n` and `4n` statements,
//! without a database. A count is the same on every run, unlike a timing, and a request that
//! does something per statement for every statement allocates quadratically more. Counts every
//! allocation of the test binary with the allocator in `common`, including tree-sitter's, so the
//! test runs on its own in this file.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "a test fails on unexpected data; clippy exempts `#[test]` functions, not their helpers"
)]

mod common;

use common::{allocated, count_native_allocations, migration_file};
use pgls_analyse::RuleCategories;
use pgls_configuration::{
    Merge, PartialConfiguration, PartialFormatConfiguration, database::PartialDatabaseConfiguration,
};
use pgls_fs::PgLSPath;
use pgls_workspace::{
    Workspace,
    features::{
        diagnostics::{DatabaseChecks, PullFileDiagnosticsParams},
        format::PullFileFormattingParams,
    },
    workspace::{
        ChangeFileParams, OpenFileParams, RegisterProjectFolderParams, UpdateSettingsParams,
        server::WorkspaceServer,
    },
};

/// The statements of the smaller file. `migration_file` writes four per group.
const STATEMENTS: usize = 500;

/// How much more a request may allocate on a file with four times the statements: linear, with
/// 10% to spare.
const GROWTH_MAX: f64 = 4.4;

const REQUESTS: [&str; 4] = ["open", "cold pull", "keystroke and pull", "format"];

/// The allocations and allocated bytes of `request`.
fn measure(request: impl FnOnce()) -> (usize, usize) {
    let (allocations, bytes) = allocated();
    request();
    let (after, after_bytes) = allocated();
    (after - allocations, after_bytes - bytes)
}

/// The allocations and allocated bytes of each of [`REQUESTS`] on a file with `statements`
/// statements, in a new workspace.
fn costs(statements: usize) -> [(usize, usize); 4] {
    let mut conf = PartialConfiguration::init();
    conf.merge_with(PartialConfiguration {
        db: Some(PartialDatabaseConfiguration {
            disable_connection: Some(true),
            ..Default::default()
        }),
        format: Some(PartialFormatConfiguration {
            enabled: Some(true),
            ..Default::default()
        }),
        ..Default::default()
    });
    let workspace = WorkspaceServer::new();
    let project_key = workspace
        .register_project_folder(RegisterProjectFolderParams {
            path: None,
            real_path: None,
        })
        .unwrap();
    workspace
        .update_settings(UpdateSettingsParams {
            project_key,
            configuration: conf,
            gitignore_matches: vec![],
            vcs_base_path: None,
            workspace_directory: None,
            replace: false,
        })
        .unwrap();
    let path = PgLSPath::new("scale.sql");
    let content = migration_file(0, statements / 4);
    // A space typed in the middle of the file, built before it is measured.
    let middle = content[..content.len() / 2].rfind('\n').unwrap() + 1;
    let edited = format!("{} {}", &content[..middle], &content[middle..]);
    let pull = || {
        workspace
            .pull_file_diagnostics(PullFileDiagnosticsParams {
                project_key,
                path: path.clone(),
                categories: RuleCategories::all(),
                max_diagnostics: u32::MAX,
                only: vec![],
                skip: vec![],
                database_checks: DatabaseChecks::Run,
            })
            .unwrap();
    };
    let open = measure(|| {
        workspace
            .open_file(OpenFileParams {
                project_key,
                path: path.clone(),
                content: content.clone(),
                version: 1,
            })
            .unwrap();
    });
    let cold_pull = measure(pull);
    let keystroke = measure(|| {
        workspace
            .change_file(ChangeFileParams {
                project_key,
                path: path.clone(),
                version: 2,
                content: edited.clone(),
            })
            .unwrap();
        pull();
    });
    let format = measure(|| {
        let result = workspace
            .pull_file_formatting(PullFileFormattingParams {
                project_key,
                path: path.clone(),
                range: None,
            })
            .unwrap();
        assert_eq!(result.statements.len(), statements);
    });
    [open, cold_pull, keystroke, format]
}

#[test]
fn costs_grow_linearly_with_the_statements() {
    count_native_allocations();
    // The first requests of the binary initialise what lives for the whole process.
    costs(8);
    let small = costs(STATEMENTS);
    let large = costs(4 * STATEMENTS);

    let mut failures = Vec::new();
    for ((request, (allocations, bytes)), (large_allocations, large_bytes)) in
        REQUESTS.iter().zip(small).zip(large)
    {
        let growth = large_allocations as f64 / allocations as f64;
        let bytes_growth = large_bytes as f64 / bytes as f64;
        println!(
            "{request}: {allocations} → {large_allocations} allocations ({growth:.2}×), \
             {bytes} → {large_bytes} bytes ({bytes_growth:.2}×)"
        );
        if growth > GROWTH_MAX || bytes_growth > GROWTH_MAX {
            failures.push(*request);
        }
    }
    assert!(
        failures.is_empty(),
        "{failures:?} grow faster than {GROWTH_MAX}× from {STATEMENTS} to {} statements",
        4 * STATEMENTS
    );
}
