use std::{
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
    time::{Duration, Instant},
};

use pgls_analyse::{Applicability, RuleCategories};
use pgls_configuration::{Merge, StringSet};
use pgls_configuration::{
    PartialConfiguration, PartialFormatConfiguration, PartialTypecheckConfiguration,
    database::PartialDatabaseConfiguration,
    files::PartialFilesConfiguration,
    migrations::{PartialMigrationsConfiguration, SectionMarkers},
};

use pgls_configuration::ConfigurationPathHint;
#[cfg(not(target_os = "windows"))]
use pgls_configuration::plpgsql_check::PartialPlPgSqlCheckConfiguration;
use pgls_diagnostics::{Diagnostic, Severity};
use pgls_fs::{ConfigName, FileSystem, FindFiles, MemoryFileSystem, PgLSPath};
use pgls_text_size::{TextRange, TextSize};
use sqlx::{Executor, PgPool};

use crate::{
    DynRef, Workspace, WorkspaceError,
    configuration::{
        IncompleteSearch, NestedConfigurationSearch, apply_nested_configurations,
        is_searched_for_configuration, load_configuration,
    },
    features::code_actions::{
        CodeActionKind, CodeActionsParams, CommandAction, CommandActionCategory,
        EXECUTE_STATEMENT_MAX_ROWS, ExecuteStatementParams, ExecuteStatementResult, FixFileMode,
        FixFileParams, TextEdit,
    },
    features::format::PullFileFormattingParams,
    features::inlay_hints::{InlayHint, InlayHintKind, PullInlayHintsParams},
    features::on_hover::OnHoverParams,
    settings::FilesSettings,
    workspace::{
        IsPathIgnoredParams, OpenFileParams, ProjectKey, RegisterProjectFolderParams, StatementId,
        UnregisterProjectFolderParams, UpdateNestedSettingsParams, UpdateSettingsParams,
        server::WorkspaceServer,
    },
};

/// A workspace with a project without a folder, which every file belongs to.
fn get_test_workspace(
    partial_config: Option<PartialConfiguration>,
) -> Result<(WorkspaceServer, ProjectKey), WorkspaceError> {
    let workspace = WorkspaceServer::new();

    let project_key =
        workspace.register_project_folder(RegisterProjectFolderParams { path: None })?;

    workspace.update_settings(UpdateSettingsParams {
        project_key,
        configuration: partial_config.unwrap_or(PartialConfiguration::init()),
        gitignore_matches: vec![],
        vcs_base_path: None,
        workspace_directory: None,
    })?;

    Ok((workspace, project_key))
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_diagnostics(test_db: PgPool) {
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

    let (workspace, project_key) =
        get_test_workspace(Some(conf)).expect("Unable to create test workspace");

    let path = PgLSPath::new("test.sql");
    let content = r#"
      create table users (
          id serial primary key,
          name text not null
      );

      drop table non_existing_table;

      select 1;
    "#;

    workspace
        .open_file(OpenFileParams {
            project_key,
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let diagnostics = workspace
        .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
            project_key,
            path: path.clone(),
            categories: RuleCategories::all(),
            max_diagnostics: 100,
            only: vec![],
            skip: vec![],
        })
        .expect("Unable to pull diagnostics")
        .diagnostics;

    assert_eq!(diagnostics.len(), 1, "Expected one diagnostic");

    let diagnostic = &diagnostics[0];

    assert_eq!(
        diagnostic.category().map(|c| c.name()),
        Some("lint/banDropTable")
    );

    assert_eq!(
        diagnostic.location().span,
        Some(TextRange::new(106.into(), 136.into()))
    );
}

#[test]
fn test_suppresses_multiline_alter_table_drop_column() {
    let (workspace, project_key) =
        get_test_workspace(None).expect("Unable to create test workspace");

    let path = PgLSPath::new("test.sql");
    let content = r#"ALTER TABLE users
    -- pgt-ignore banDropColumn
    DROP COLUMN deprecated_field;"#;

    workspace
        .open_file(OpenFileParams {
            project_key,
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let diagnostics = workspace
        .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
            project_key,
            path: path.clone(),
            categories: RuleCategories::all(),
            max_diagnostics: 100,
            only: vec!["safety/banDropColumn".parse().unwrap()],
            skip: vec![],
        })
        .expect("Unable to pull diagnostics")
        .diagnostics;

    assert!(
        diagnostics.is_empty(),
        "Expected no diagnostics, got {diagnostics:#?}"
    );
}

/// The categories of the diagnostics of a migration file, sorted. Suppression errors have no
/// category.
fn migration_diagnostics(
    section_markers: SectionMarkers,
    lint_down: Option<bool>,
    content: &str,
) -> Vec<String> {
    let mut conf = PartialConfiguration::init();
    conf.merge_with(PartialConfiguration {
        migrations: Some(PartialMigrationsConfiguration {
            section_markers: Some(section_markers),
            lint_down,
            ..Default::default()
        }),
        ..Default::default()
    });
    let (workspace, project_key) =
        get_test_workspace(Some(conf)).expect("Unable to create test workspace");

    let path = PgLSPath::new("20250301120000_migration.sql");
    workspace
        .open_file(OpenFileParams {
            project_key,
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let diagnostics = workspace
        .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
            project_key,
            path: path.clone(),
            categories: RuleCategories::all(),
            max_diagnostics: 100,
            only: vec![],
            skip: vec![],
        })
        .expect("Unable to pull diagnostics")
        .diagnostics;

    let mut categories: Vec<String> = diagnostics
        .iter()
        .map(|d| d.category().map_or("", |c| c.name()).to_string())
        .collect();
    categories.sort();
    categories
}

const DBMATE_DROP_NAME: &str = r#"-- migrate:up
ALTER TABLE users DROP COLUMN name;

-- migrate:down
-- pgls-ignore banDropColumn
ALTER TABLE users DROP COLUMN email;
DROP TABLE users;
seect 1;
"#;

fn drops_and_errors(categories: Vec<String>) -> Vec<String> {
    categories
        .into_iter()
        .filter(|category| {
            category.starts_with("lint/banDrop") || category == "syntax" || category.is_empty()
        })
        .collect()
}

#[test]
fn lints_down_sections_by_default() {
    assert_eq!(
        drops_and_errors(migration_diagnostics(
            SectionMarkers::Dbmate,
            None,
            DBMATE_DROP_NAME
        )),
        ["lint/banDropColumn", "lint/banDropTable", "syntax"]
    );
}

#[test]
fn skips_down_sections_without_lint_down() {
    // The syntax error is still reported, and the suppression isn't reported as unused.
    assert_eq!(
        drops_and_errors(migration_diagnostics(
            SectionMarkers::Dbmate,
            Some(false),
            DBMATE_DROP_NAME
        )),
        ["lint/banDropColumn", "syntax"]
    );
}

#[test]
fn goose_sections_run_without_transaction() {
    let content = r#"-- +goose Up
CREATE INDEX CONCURRENTLY IF NOT EXISTS users_email_idx ON users (email);
CREATE INDEX CONCURRENTLY IF NOT EXISTS users_name_idx ON users (name);

-- +goose Down
DROP INDEX CONCURRENTLY IF EXISTS users_email_idx;
DROP INDEX CONCURRENTLY IF EXISTS users_name_idx;
"#;
    let in_transaction = |categories: Vec<String>| {
        categories
            .iter()
            .filter(|category| *category == "lint/banConcurrentIndexCreationInTransaction")
            .count()
    };

    // Both sections run in a transaction.
    assert_eq!(
        in_transaction(migration_diagnostics(SectionMarkers::Goose, None, content)),
        4
    );
    let content = format!("-- +goose NO TRANSACTION\n{content}");
    assert_eq!(
        in_transaction(migration_diagnostics(SectionMarkers::Goose, None, &content)),
        0
    );
}

#[test]
fn test_legacy_suppression_still_suppresses_and_warns() {
    let (workspace, project_key) =
        get_test_workspace(None).expect("Unable to create test workspace");

    let path = PgLSPath::new("test.sql");
    let content = r#"-- pgls-ignore lint/safety/banDropColumn
ALTER TABLE users DROP COLUMN deprecated_field;"#;

    workspace
        .open_file(OpenFileParams {
            project_key,
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let diagnostics = workspace
        .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
            project_key,
            path: path.clone(),
            categories: RuleCategories::all(),
            max_diagnostics: 100,
            only: vec![],
            skip: vec![],
        })
        .expect("Unable to pull diagnostics")
        .diagnostics;

    assert!(
        !diagnostics
            .iter()
            .any(|d| d.category().map(|c| c.name()) == Some("lint/banDropColumn")),
        "{diagnostics:#?}"
    );
    assert!(
        diagnostics
            .iter()
            .any(|d| d.description_text().contains("Use `banDropColumn` instead")),
        "{diagnostics:#?}"
    );
}

#[test]
fn test_unreachable_database_diagnostics_keep_static_results_and_back_off() {
    let mut conf = PartialConfiguration::init();
    conf.merge_with(PartialConfiguration {
        db: Some(PartialDatabaseConfiguration {
            connection_string: Some(
                "postgres://postgres:postgres@127.0.0.1:65432/postgres".to_string(),
            ),
            conn_timeout_secs: Some(1),
            ..Default::default()
        }),
        ..Default::default()
    });

    let (workspace, project_key) =
        get_test_workspace(Some(conf)).expect("Unable to create test workspace");

    let path = PgLSPath::new("test.sql");
    let content = "drop table non_existing_table;";

    workspace
        .open_file(OpenFileParams {
            project_key,
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let pull_diagnostics = || {
        workspace
            .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
                project_key,
                path: path.clone(),
                categories: RuleCategories::all(),
                max_diagnostics: 100,
                only: vec![],
                skip: vec![],
            })
            .expect("Unable to pull diagnostics")
            .diagnostics
    };

    let diagnostics = pull_diagnostics();
    assert!(
        diagnostics.iter().any(|d| d
            .category()
            .is_some_and(|c| c.name() == "lint/banDropTable")),
        "Expected static lint diagnostics even when database is unreachable"
    );

    let second_pull_started = Instant::now();
    let diagnostics = pull_diagnostics();
    let second_pull_elapsed = second_pull_started.elapsed();

    assert!(
        diagnostics.iter().any(|d| d
            .category()
            .is_some_and(|c| c.name() == "lint/banDropTable")),
        "Expected static lint diagnostics during database retry backoff"
    );
    assert!(
        second_pull_elapsed < Duration::from_millis(750),
        "Expected diagnostics during database retry backoff to finish quickly, took {second_pull_elapsed:?}"
    );
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_syntax_error(test_db: PgPool) {
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

    let (workspace, project_key) =
        get_test_workspace(Some(conf)).expect("Unable to create test workspace");

    let path = PgLSPath::new("test.sql");
    let content = r#"
      seect 1;
    "#;

    workspace
        .open_file(OpenFileParams {
            project_key,
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let diagnostics = workspace
        .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
            project_key,
            path: path.clone(),
            categories: RuleCategories::all(),
            max_diagnostics: 100,
            only: vec![],
            skip: vec![],
        })
        .expect("Unable to pull diagnostics")
        .diagnostics;

    assert_eq!(diagnostics.len(), 1, "Expected one diagnostic");

    let diagnostic = &diagnostics[0];

    assert_eq!(diagnostic.category().map(|c| c.name()), Some("syntax"));

    assert_eq!(
        diagnostic.location().span,
        Some(TextRange::new(7.into(), 15.into()))
    );
}

#[tokio::test]
async fn named_parameter_normalization_preserves_syntax_diagnostic_offsets() {
    let (workspace, project_key) =
        get_test_workspace(None).expect("Unable to create test workspace");
    let path = PgLSPath::new("named-parameter.sql");
    let content = "SELECT * FROM :very_long_schema.existing_table AS t seect 1;";

    workspace
        .open_file(OpenFileParams {
            project_key,
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let diagnostics = workspace
        .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
            project_key,
            path,
            categories: RuleCategories::all(),
            max_diagnostics: 100,
            only: vec![],
            skip: vec![],
        })
        .expect("Unable to pull diagnostics")
        .diagnostics;

    let syntax_diagnostic = diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic
                .category()
                .is_some_and(|category| category.name() == "syntax")
        })
        .expect("Expected one syntax diagnostic");

    let expected_span = TextRange::new(0.into(), u32::try_from(content.len()).unwrap().into());
    assert_eq!(syntax_diagnostic.location().span, Some(expected_span));

    let qualifier_start = content
        .find(":very_long_schema")
        .expect("query contains the named qualifier");
    assert_ne!(
        syntax_diagnostic.location().span,
        Some(TextRange::new(
            u32::try_from(qualifier_start).unwrap().into(),
            u32::try_from(qualifier_start + ":very_long_schema".len())
                .unwrap()
                .into()
        ))
    );
}

#[tokio::test]
async fn format_preserves_named_parameters() {
    let mut conf = PartialConfiguration::init();
    conf.merge_with(PartialConfiguration {
        format: Some(PartialFormatConfiguration {
            enabled: Some(true),
            ..Default::default()
        }),
        ..Default::default()
    });

    let (workspace, project_key) =
        get_test_workspace(Some(conf)).expect("Unable to create test workspace");
    let path = PgLSPath::new("named-parameter-format.sql");
    let content = "SELECT x FROM :raw_data.t WHERE t.a = :'agen_code' AND t.b = :var_date;";

    workspace
        .open_file(OpenFileParams {
            project_key,
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let result = workspace
        .pull_file_formatting(PullFileFormattingParams {
            project_key,
            path,
            range: None,
        })
        .expect("Unable to format file");

    assert!(result.formatted.contains(":raw_data.t"));
    assert!(result.formatted.contains(":'agen_code'"));
    assert!(result.formatted.contains(":var_date"));
    assert!(!result.formatted.contains("$1"));
}

#[tokio::test]
async fn correctly_ignores_files() {
    let mut conf = PartialConfiguration::init();
    conf.merge_with(PartialConfiguration {
        files: Some(PartialFilesConfiguration {
            ignore: Some(StringSet::from_iter(["test.sql".to_string()])),
            ..Default::default()
        }),
        ..Default::default()
    });

    let (workspace, project_key) =
        get_test_workspace(Some(conf)).expect("Unable to create test workspace");

    let path = PgLSPath::new("test.sql");
    let content = r#"
      seect 1;
    "#;

    let diagnostics_result =
        workspace.pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
            project_key,
            path: path.clone(),
            categories: RuleCategories::all(),
            max_diagnostics: 100,
            only: vec![],
            skip: vec![],
        });

    assert!(
        diagnostics_result
            .is_ok_and(|res| res.diagnostics.is_empty() && res.skipped_diagnostics == 0)
    );

    let close_file_result = workspace.close_file(crate::workspace::CloseFileParams {
        project_key,
        path: path.clone(),
    });

    assert!(close_file_result.is_ok());

    let execute_statement_result =
        workspace.execute_statement(crate::workspace::ExecuteStatementParams {
            project_key,
            path: path.clone(),
            statement_id: StatementId::Root {
                content: Arc::from(content),
            },
        });

    assert!(execute_statement_result.is_ok_and(|res| res == ExecuteStatementResult::default()));
}

#[cfg(all(test, not(target_os = "windows")))]
#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_dedupe_diagnostics(test_db: PgPool) {
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

    let (workspace, project_key) =
        get_test_workspace(Some(conf)).expect("Unable to create test workspace");

    let path = PgLSPath::new("test.sql");

    let setup_sql = "CREATE EXTENSION IF NOT EXISTS plpgsql_check;";
    test_db.execute(setup_sql).await.expect("setup sql failed");

    let content = r#"
        CREATE OR REPLACE FUNCTION public.f1()
        RETURNS void
        LANGUAGE plpgsql
        AS $function$
        decare r text;
        BEGIN
            select '1' into into r;
        END;
        $function$;
    "#;

    workspace
        .open_file(OpenFileParams {
            project_key,
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let diagnostics = workspace
        .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
            project_key,
            path: path.clone(),
            categories: RuleCategories::all(),
            max_diagnostics: 100,
            only: vec![],
            skip: vec![],
        })
        .expect("Unable to pull diagnostics")
        .diagnostics;

    assert_eq!(diagnostics.len(), 1, "Expected one diagnostic");

    let diagnostic = &diagnostics[0];

    assert_eq!(
        diagnostic.category().map(|c| c.name()),
        Some("plpgsql_check")
    );

    // plpgsql_check locates the syntax error instead of flagging the whole function body
    let span = diagnostic.location().span.expect("Expected a span");
    assert_eq!(&content[span], "decare");
}

/// Without a trigger, plpgsql_check cannot check a trigger function, so the syntax error of
/// libpg_query remains.
#[cfg(all(test, not(target_os = "windows")))]
#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_plpgsql_syntax_error_without_plpgsql_check(test_db: PgPool) {
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

    let (workspace, project_key) =
        get_test_workspace(Some(conf)).expect("Unable to create test workspace");

    let path = PgLSPath::new("test.sql");

    let setup_sql = "CREATE EXTENSION IF NOT EXISTS plpgsql_check;";
    test_db.execute(setup_sql).await.expect("setup sql failed");

    let content = r#"
        CREATE OR REPLACE FUNCTION public.f1()
        RETURNS trigger
        LANGUAGE plpgsql
        AS $function$
        BEGIN
            retrun new;
        END;
        $function$;
    "#;

    workspace
        .open_file(OpenFileParams {
            project_key,
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let diagnostics = workspace
        .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
            project_key,
            path: path.clone(),
            categories: RuleCategories::all(),
            max_diagnostics: 100,
            only: vec![],
            skip: vec![],
        })
        .expect("Unable to pull diagnostics")
        .diagnostics;

    assert_eq!(diagnostics.len(), 1, "Expected one diagnostic");
    assert_eq!(diagnostics[0].category().map(|c| c.name()), Some("syntax"));
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_plpgsql_assign_composite_types(test_db: PgPool) {
    let conf = PartialConfiguration::init();

    let (workspace, project_key) =
        get_test_workspace(Some(conf)).expect("Unable to create test workspace");

    let path = PgLSPath::new("test.sql");

    let setup_sql = r"
        create table if not exists _fetch_cycle_continuation_data (
            next_id bigint,
            next_state jsonb null default '{}'::jsonb
            constraint abstract_no_data check(false) no inherit
        );
    ";
    test_db.execute(setup_sql).await.expect("setup sql failed");

    let content = r#"
        create or replace function continue_fetch_cycle_prototype ()
        returns _fetch_cycle_continuation_data language plpgsql as $prototype$
        declare
            result _fetch_cycle_continuation_data := null;
        begin
            result.next_id := 0;
            result.next_state := '{}'::jsonb

            return result;
        end;
        $prototype$
    "#;

    workspace
        .open_file(OpenFileParams {
            project_key,
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let diagnostics = workspace
        .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
            project_key,
            path: path.clone(),
            categories: RuleCategories::all(),
            max_diagnostics: 100,
            only: vec![],
            skip: vec![],
        })
        .expect("Unable to pull diagnostics")
        .diagnostics;

    assert_eq!(diagnostics.len(), 0, "Expected no diagnostic");
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_positional_params(test_db: PgPool) {
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

    let (workspace, project_key) =
        get_test_workspace(Some(conf)).expect("Unable to create test workspace");

    let path = PgLSPath::new("test.sql");

    let setup_sql = r"
      create table users (
          id serial primary key,
          name text not null,
          email text not null
      );
    ";
    test_db.execute(setup_sql).await.expect("setup sql failed");

    let content = r#"select * from users where id = @one and name = :two and email = :'three';"#;

    workspace
        .open_file(OpenFileParams {
            project_key,
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let diagnostics = workspace
        .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
            project_key,
            path: path.clone(),
            categories: RuleCategories::all(),
            max_diagnostics: 100,
            only: vec![],
            skip: vec![],
        })
        .expect("Unable to pull diagnostics")
        .diagnostics;

    assert_eq!(diagnostics.len(), 0, "Expected no diagnostic");
}

#[cfg(all(test, not(target_os = "windows")))]
#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_disable_plpgsql_check(test_db: PgPool) {
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

    let (workspace, project_key) =
        get_test_workspace(Some(conf)).expect("Unable to create test workspace");

    let path = PgLSPath::new("test.sql");

    let setup_sql = "CREATE EXTENSION IF NOT EXISTS plpgsql_check;";
    test_db.execute(setup_sql).await.expect("setup sql failed");

    let content = r#"
        CREATE OR REPLACE FUNCTION public.f1()
        RETURNS void
        LANGUAGE plpgsql
        AS $function$
        decare r text;
        BEGIN
            select '1' into into r;
        END;
        $function$;
    "#;

    test_db.execute(setup_sql).await.expect("setup sql failed");

    workspace
        .open_file(OpenFileParams {
            project_key,
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let diagnostics = workspace
        .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
            project_key,
            path: path.clone(),
            categories: RuleCategories::all(),
            max_diagnostics: 100,
            only: vec![],
            skip: vec![],
        })
        .expect("Unable to pull diagnostics")
        .diagnostics;

    assert_eq!(
        diagnostics
            .iter()
            .filter(|d| d.category().is_some_and(|c| c.name() == "plpgsql_check"))
            .count(),
        1,
        "Expected one plpgsql_check diagnostic"
    );

    let _ = workspace.update_settings(UpdateSettingsParams {
        project_key,
        configuration: PartialConfiguration {
            plpgsql_check: Some(PartialPlPgSqlCheckConfiguration {
                enabled: Some(false),
                ..Default::default()
            }),
            ..Default::default()
        },
        gitignore_matches: vec![],
        vcs_base_path: None,
        workspace_directory: None,
    });

    let diagnostics = workspace
        .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
            project_key,
            path: path.clone(),
            categories: RuleCategories::all(),
            max_diagnostics: 100,
            only: vec![],
            skip: vec![],
        })
        .expect("Unable to pull diagnostics")
        .diagnostics;

    assert_eq!(
        diagnostics
            .iter()
            .filter(|d| d.category().is_some_and(|c| c.name() == "plpgsql_check"))
            .count(),
        0,
        "Expected no plpgsql_check diagnostic"
    );
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_disable_typecheck(test_db: PgPool) {
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

    let (workspace, project_key) =
        get_test_workspace(Some(conf)).expect("Unable to create test workspace");

    let path = PgLSPath::new("test.sql");

    let setup_sql = r"
      create table users (
          id serial primary key,
          email text not null
      );
    ";
    test_db.execute(setup_sql).await.expect("setup sql failed");

    let content = r#"select name from users where id = 1;"#;

    workspace
        .open_file(OpenFileParams {
            project_key,
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let diagnostics = workspace
        .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
            project_key,
            path: path.clone(),
            categories: RuleCategories::all(),
            max_diagnostics: 100,
            only: vec![],
            skip: vec![],
        })
        .expect("Unable to pull diagnostics")
        .diagnostics;

    assert_eq!(
        diagnostics.iter().filter(|d| is_typecheck(d)).count(),
        1,
        "Expected one typecheck diagnostic"
    );

    let _ = workspace.update_settings(UpdateSettingsParams {
        project_key,
        configuration: PartialConfiguration {
            typecheck: Some(PartialTypecheckConfiguration {
                enabled: Some(false),
                ..Default::default()
            }),
            ..Default::default()
        },
        gitignore_matches: vec![],
        vcs_base_path: None,
        workspace_directory: None,
    });

    let diagnostics = workspace
        .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
            project_key,
            path: path.clone(),
            categories: RuleCategories::all(),
            max_diagnostics: 100,
            only: vec![],
            skip: vec![],
        })
        .expect("Unable to pull diagnostics")
        .diagnostics;

    assert_eq!(
        diagnostics.iter().filter(|d| is_typecheck(d)).count(),
        0,
        "Expected no typecheck diagnostic"
    );
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_create_as_typecheck_diagnostic_offsets(test_db: PgPool) {
    test_db
        .execute("CREATE TABLE public.t (a INT);")
        .await
        .expect("setup sql failed");

    let options = test_db.connect_options();
    let mut conf = PartialConfiguration::init();
    conf.merge_with(PartialConfiguration {
        db: Some(PartialDatabaseConfiguration {
            host: Some(options.get_host().to_string()),
            port: Some(options.get_port()),
            database: Some(options.get_database().unwrap().to_string()),
            ..Default::default()
        }),
        ..Default::default()
    });

    let (workspace, project_key) =
        get_test_workspace(Some(conf)).expect("Unable to create test workspace");
    let path = PgLSPath::new("test.sql");
    let content = "SELECT 1;\nCREATE VIEW v1 AS SELECT t.nope FROM t;";

    workspace
        .open_file(OpenFileParams {
            project_key,
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let diagnostics = workspace
        .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
            project_key,
            path,
            categories: RuleCategories::all(),
            max_diagnostics: 100,
            only: vec![],
            skip: vec![],
        })
        .expect("Unable to pull diagnostics")
        .diagnostics;

    let typecheck_diagnostics = diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic
                .category()
                .is_some_and(|category| category.name() == "lint/unknownColumn")
        })
        .collect::<Vec<_>>();

    assert_eq!(
        typecheck_diagnostics.len(),
        1,
        "Expected one unknownColumn diagnostic, got {diagnostics:#?}"
    );

    let expected_start = content.find("t.nope").expect("missing test identifier");
    let expected_span = TextRange::new(
        u32::try_from(expected_start).unwrap().into(),
        u32::try_from(expected_start + "t.nope".len())
            .unwrap()
            .into(),
    );

    assert_eq!(
        typecheck_diagnostics[0].location().span,
        Some(expected_span)
    );
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn named_identifier_params_skip_only_affected_typecheck(test_db: PgPool) {
    let connect_options = test_db.connect_options();
    let host = connect_options.get_host().to_string();
    let port = connect_options.get_port();
    let database = connect_options
        .get_database()
        .expect("test database must have a name")
        .to_string();

    let mut conf = PartialConfiguration::init();
    conf.merge_with(PartialConfiguration {
        db: Some(PartialDatabaseConfiguration {
            host: Some(host),
            port: Some(port),
            database: Some(database),
            ..Default::default()
        }),
        ..Default::default()
    });

    test_db
        .execute("CREATE TABLE named_parameter_typecheck_users (id integer PRIMARY KEY);")
        .await
        .expect("test table setup must succeed");

    let (workspace, project_key) =
        get_test_workspace(Some(conf)).expect("Unable to create test workspace");
    let path = PgLSPath::new("named-identifier-parameter.sql");
    let content = r#"
SELECT * FROM :raw_data.documents;
SELECT missing_column FROM named_parameter_typecheck_users;
"#;

    workspace
        .open_file(OpenFileParams {
            project_key,
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let diagnostics = workspace
        .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
            project_key,
            path,
            categories: RuleCategories::all(),
            max_diagnostics: 100,
            only: vec![],
            skip: vec![],
        })
        .expect("Unable to pull diagnostics")
        .diagnostics;

    let typecheck_diagnostics = diagnostics
        .iter()
        .filter(|diagnostic| is_typecheck(diagnostic))
        .collect::<Vec<_>>();

    assert_eq!(
        typecheck_diagnostics.len(),
        1,
        "only the second statement should produce a typecheck diagnostic: {diagnostics:#?}"
    );

    let missing_column_start = content
        .find("missing_column")
        .expect("test SQL contains the invalid column");
    let missing_column_end = missing_column_start + "missing_column".len();
    assert_eq!(
        typecheck_diagnostics[0].location().span,
        Some(TextRange::new(
            u32::try_from(missing_column_start).unwrap().into(),
            u32::try_from(missing_column_end).unwrap().into(),
        ))
    );
}

/// The typecheck diagnostics of `content`, checked against `test_db`.
fn typecheck_diagnostics(
    test_db: &PgPool,
    content: &str,
) -> Vec<pgls_diagnostics::serde::Diagnostic> {
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
    let (workspace, project_key) =
        get_test_workspace(Some(conf)).expect("Unable to create test workspace");
    let path = PgLSPath::new("test.sql");
    workspace
        .open_file(OpenFileParams {
            project_key,
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");
    workspace
        .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
            project_key,
            path,
            categories: RuleCategories::all(),
            max_diagnostics: 100,
            only: vec![],
            skip: vec![],
        })
        .expect("Unable to pull diagnostics")
        .diagnostics
        .into_iter()
        .filter(is_typecheck)
        .collect()
}

/// Postgres treats non-ASCII characters as letters in identifiers, so diagnostics and hover
/// cover the whole name.
#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn non_ascii_names_get_whole_spans(test_db: PgPool) {
    let content = "create table café (id int);\nselect a.größe from café a;\n";
    let diagnostics = typecheck_diagnostics(&test_db, content);
    assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
    let span = diagnostics[0].location().span.expect("a span");
    assert_eq!(&content[span], "a.größe");

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
    let (workspace, project_key) =
        get_test_workspace(Some(conf)).expect("Unable to create test workspace");
    let path = PgLSPath::new("test.sql");
    let content = "create table café (größe int);\ncreate table tee (größe int);\n\
        select a.größe from café a, tee b;\n";
    workspace
        .open_file(OpenFileParams {
            project_key,
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");
    // On the `ß`, after the first non-ASCII character. Without the resolved reference, hover
    // shows the columns of both tables.
    let position = content.find("ße from").unwrap();
    let blocks = workspace
        .on_hover(OnHoverParams {
            project_key,
            path: path.clone(),
            position: u32::try_from(position).unwrap().into(),
        })
        .expect("Unable to hover")
        .markdown_blocks;
    assert_eq!(blocks.len(), 1, "{blocks:?}");
    assert!(blocks[0].contains("public.café.größe"), "{blocks:?}");
}

/// GitHub issue #835: `RETURNS TABLE` columns are not parameters of the body, and `r.received_at`
/// is a column of `r` even if an output column has the same name.
#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn sql_function_output_columns_are_not_parameters(test_db: PgPool) {
    test_db
        .execute("CREATE TABLE ev (id bigint PRIMARY KEY, received_at timestamptz NOT NULL);")
        .await
        .expect("test table setup must succeed");

    let content = r#"
CREATE OR REPLACE FUNCTION next_after(p_above bigint) RETURNS TABLE(id bigint, received_at timestamptz)
    LANGUAGE sql STABLE
    AS $$
    SELECT r.id, r.received_at
    FROM   ev r
    WHERE  r.id > p_above
    ORDER  BY r.received_at, r.id
    LIMIT  1
$$;
"#;
    let diagnostics = typecheck_diagnostics(&test_db, content);
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");

    // A type error involving the input parameter is still reported.
    let content = r#"
CREATE OR REPLACE FUNCTION next_after(p_above bigint) RETURNS TABLE(id bigint, received_at timestamptz)
    LANGUAGE sql STABLE
    AS $$
    SELECT r.id, r.received_at
    FROM   ev r
    WHERE  r.received_at > p_above
    ORDER  BY r.received_at, r.id
    LIMIT  1
$$;
"#;
    let diagnostics = typecheck_diagnostics(&test_db, content);
    assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
    let message = diagnostics[0].description_text();
    assert!(
        message.contains("timestamp with time zone > bigint"),
        "{message}"
    );

    // Output parameters are not in scope in the body.
    let diagnostics = typecheck_diagnostics(
        &test_db,
        "CREATE FUNCTION f(a int, OUT x int) LANGUAGE sql AS $$ SELECT x FROM ev $$;",
    );
    assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
    let message = diagnostics[0].description_text();
    assert!(message.contains("Column x does not exist"), "{message}");
}

/// GitHub issue #856: parameters of a SQL function are resolved by the static typecheck rules,
/// not by EXPLAIN, which got array parameters and parameters in `extract(... from ...)` wrong.
#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn sql_function_bodies_are_not_explained(test_db: PgPool) {
    let content = r#"
create or replace function interval_to_ms_config(p_interval interval)
returns text
language sql
immutable
as $$
    select ceil(extract(epoch from p_interval) * 1000)::bigint::text || 'ms'
$$;

create or replace function int_array_nonnegative_not_null(p_arr int[])
returns boolean
language sql
immutable
as $$
    select bool_and(x is not null and x >= 0)
    from unnest(p_arr) as x
$$;
"#;
    let diagnostics = typecheck_diagnostics(&test_db, content);
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");

    // An unknown column of a database table is still reported, by the static rules.
    test_db
        .execute("CREATE TABLE fn_body_users (id integer PRIMARY KEY, email text);")
        .await
        .expect("test table setup must succeed");
    let diagnostics = typecheck_diagnostics(
        &test_db,
        "CREATE FUNCTION f(p_id int) RETURNS text LANGUAGE sql AS $$ SELECT emial FROM fn_body_users WHERE id = p_id $$;",
    );
    assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
    assert_eq!(
        diagnostics[0].category().map(|category| category.name()),
        Some("lint/unknownColumn")
    );
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_named_params(_test_db: PgPool) {
    let conf = PartialConfiguration::init();

    let (workspace, project_key) =
        get_test_workspace(Some(conf)).expect("Unable to create test workspace");

    let path = PgLSPath::new("test.sql");

    let content = r#"
SELECT
  1
FROM
  assessments AS a
WHERE
  a.id = $assessment_id
FOR NO KEY UPDATE;
    "#;

    workspace
        .open_file(OpenFileParams {
            project_key,
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let diagnostics = workspace
        .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
            project_key,
            path: path.clone(),
            categories: RuleCategories::all(),
            max_diagnostics: 100,
            only: vec![],
            skip: vec![],
        })
        .expect("Unable to pull diagnostics")
        .diagnostics;

    assert_eq!(
        diagnostics
            .iter()
            .filter(|d| d.category().is_some_and(|c| c.name() == "syntax"))
            .count(),
        0,
        "Expected no syntax diagnostic"
    );
}

#[tokio::test]
async fn test_format_keeps_sql_function_body_intact() {
    let mut conf = PartialConfiguration::init();
    conf.merge_with(PartialConfiguration {
        format: Some(PartialFormatConfiguration {
            enabled: Some(true),
            ..Default::default()
        }),
        ..Default::default()
    });

    let (workspace, project_key) =
        get_test_workspace(Some(conf)).expect("Unable to create test workspace");

    let path = PgLSPath::new("test.sql");
    let content =
        "create function add(a int, b int) returns int as 'SELECT 424242+$1+$2;' language sql;";

    workspace
        .open_file(OpenFileParams {
            project_key,
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let result = workspace
        .pull_file_formatting(PullFileFormattingParams {
            project_key,
            path: path.clone(),
            range: None,
        })
        .expect("Unable to pull formatting");

    assert_eq!(
        result.formatted.matches("424242").count(),
        1,
        "SQL function body should not be emitted as a second standalone statement:\n{}",
        result.formatted,
    );

    assert!(
        result.formatted.contains("select 424242 + $1 + $2;"),
        "SQL function body should be formatted inline:\n{}",
        result.formatted,
    );
}

fn format_content(content: &str) -> String {
    let mut conf = PartialConfiguration::init();
    conf.merge_with(PartialConfiguration {
        format: Some(PartialFormatConfiguration {
            enabled: Some(true),
            ..Default::default()
        }),
        ..Default::default()
    });

    let (workspace, project_key) =
        get_test_workspace(Some(conf)).expect("Unable to create test workspace");

    let path = PgLSPath::new("test.sql");
    workspace
        .open_file(OpenFileParams {
            project_key,
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    workspace
        .pull_file_formatting(PullFileFormattingParams {
            project_key,
            path,
            range: None,
        })
        .expect("Unable to pull formatting")
        .formatted
}

#[tokio::test]
async fn test_format_preserves_leading_comment() {
    let formatted = format_content("-- a header comment\nselect   1;");
    assert_eq!(formatted, "-- a header comment\nselect 1;");
}

#[tokio::test]
async fn test_format_preserves_trailing_comment() {
    let formatted = format_content("select   amount   from customers; -- selects amount");
    assert_eq!(formatted, "select amount from customers; -- selects amount");
}

#[tokio::test]
async fn test_format_preserves_between_statement_comment() {
    let formatted = format_content("select   1;\n\n-- in between\n\nselect   2;");
    assert_eq!(formatted, "select 1;\n\n-- in between\n\nselect 2;");
}

#[tokio::test]
async fn test_format_preserves_interior_comment() {
    let content = "SELECT\n-- pick the magic value\n1 AS a, 2 AS b FROM s.t;";
    let formatted = format_content(content);
    assert!(formatted.contains("-- pick the magic value"));
    assert_ne!(formatted, content, "the statement was reformatted");
}

#[tokio::test]
async fn test_format_with_trailing_comment_is_idempotent() {
    let first = format_content("SELECT * FROM t WHERE a = 1 -- condition\nAND b = 2;");
    let second = format_content(&first);

    assert!(first.contains("1 -- condition"));
    assert_eq!(first, second);
}

#[tokio::test]
async fn test_format_preserves_block_comment() {
    let content = "select 1 /* keep me */;";
    let formatted = format_content(content);
    assert!(formatted.contains("/* keep me */"));
}

#[tokio::test]
async fn test_format_preserves_comment_in_sql_function_body() {
    let content =
        "create function f() returns int as $$\n  select 1; -- inner comment\n$$ language sql;";
    let formatted = format_content(content);
    assert!(
        formatted.contains("-- inner comment"),
        "comment inside the function body should be preserved:\n{formatted}",
    );
}

#[tokio::test]
async fn test_format_normalizes_blank_lines_between_statements() {
    let formatted = format_content("\n\n\nselect   1;\n\n\n\nselect   2;\n\n\n");
    assert_eq!(formatted, "select 1;\n\nselect 2;\n");
}

#[tokio::test]
async fn test_format_keeps_comments_when_normalizing_blank_lines() {
    let formatted = format_content(
        "\n\n-- header\n\n\nselect 1; -- one\n\n\n-- about two\nselect 2;\n\n\n-- footer\n\n",
    );
    assert_eq!(
        formatted,
        "-- header\n\nselect 1; -- one\n\n-- about two\nselect 2;\n\n-- footer\n"
    );
}

#[tokio::test]
async fn test_format_keeps_statements_on_one_line_apart() {
    let formatted = format_content("select   1; select   2;");
    assert_eq!(formatted, "select 1; select 2;");
}

#[tokio::test]
async fn test_format_keeps_plpgsql_body() {
    let content = "CREATE FUNCTION f() RETURNS boolean LANGUAGE plpgsql AS $$\nBEGIN\n  RETURN true;\nEND\n$$;";
    let formatted = format_content(content);
    assert_eq!(
        formatted,
        "create function f()\nreturns boolean\nlanguage plpgsql\nas $$\nBEGIN\n  RETURN true;\nEND\n$$;"
    );
}

#[tokio::test]
async fn test_format_keeps_formatted_sql_function_body_as_written() {
    let content = "create function f() returns int language sql as $body$\n  select 1;\n$body$;";
    let formatted = format_content(content);
    assert_eq!(
        formatted,
        "create function f()\nreturns int\nlanguage sql\nas $$\n  select 1;\n$$;"
    );
}

#[tokio::test]
async fn test_format_puts_reformatted_sql_function_body_on_its_own_lines() {
    let content = "create function f() returns int language sql as $body$SELECT   1$body$;";
    let formatted = format_content(content);
    assert_eq!(
        formatted,
        "create function f()\nreturns int\nlanguage sql\nas $$\nselect 1;\n$$;"
    );
    assert_eq!(format_content(&formatted), formatted);
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_cstyle_comments(test_db: PgPool) {
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

    let (workspace, project_key) =
        get_test_workspace(Some(conf)).expect("Unable to create test workspace");

    let path = PgLSPath::new("test.sql");

    let content = r#"
        /*
         * a
         * multi-line
         * comment.
         */
        select 1; /* Another comment */
        -- A single line comment
        select 2; -- Another single line comment
    "#;

    workspace
        .open_file(OpenFileParams {
            project_key,
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let diagnostics = workspace
        .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
            project_key,
            path: path.clone(),
            categories: RuleCategories::all(),
            max_diagnostics: 100,
            only: vec![],
            skip: vec![],
        })
        .expect("Unable to pull diagnostics")
        .diagnostics;

    assert_eq!(diagnostics.len(), 0, "Expected no diagnostic");
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_search_path_configuration(test_db: PgPool) {
    // Setup test schemas and functions
    let setup_sql = r#"
        create schema if not exists private;

        create or replace function private.get_user_id() returns integer as $$
            select 1;
        $$ language sql;
    "#;
    test_db.execute(setup_sql).await.expect("setup sql failed");

    let path_glob = PgLSPath::new("test_glob.sql");
    let file_content = r#"
        select get_user_id();  -- on private schema
    "#;

    // first check that the we get a valid typecheck
    let mut glob_conf = PartialConfiguration::init();
    glob_conf.merge_with(PartialConfiguration {
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

    // without glob
    {
        let (workspace, project_key) =
            get_test_workspace(Some(glob_conf.clone())).expect("Unable to create test workspace");

        workspace
            .open_file(OpenFileParams {
                project_key,
                path: path_glob.clone(),
                content: file_content.into(),
                version: 1,
            })
            .expect("Unable to open test file");

        let diagnostics_glob = workspace
            .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
                project_key,
                path: path_glob.clone(),
                categories: RuleCategories::all(),
                max_diagnostics: 100,
                only: vec![],
                skip: vec![],
            })
            .expect("Unable to pull diagnostics")
            .diagnostics;

        assert_eq!(
            diagnostics_glob.len(),
            1,
            "get_user_id() should not be found in search_path"
        );

        // yep, type error!
        assert_eq!(
            diagnostics_glob[0].category().map(|c| c.name()),
            Some("lint/unknownFunction")
        );
    }

    // adding the glob
    glob_conf.merge_with(PartialConfiguration {
        typecheck: Some(PartialTypecheckConfiguration {
            enabled: Some(true),
            // Adding glob pattern to match the "private" schema
            search_path: Some(StringSet::from_iter(vec!["pr*".to_string()])),
        }),
        ..Default::default()
    }); // checking with the pattern should yield no diagnostics

    {
        let (workspace, project_key) =
            get_test_workspace(Some(glob_conf.clone())).expect("Unable to create test workspace");

        workspace
            .open_file(OpenFileParams {
                project_key,
                path: path_glob.clone(),
                content: file_content.into(),
                version: 1,
            })
            .expect("Unable to open test file");

        let diagnostics_glob = workspace
            .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
                project_key,
                path: path_glob.clone(),
                categories: RuleCategories::all(),
                max_diagnostics: 100,
                only: vec![],
                skip: vec![],
            })
            .expect("Unable to pull diagnostics")
            .diagnostics;

        assert_eq!(
            diagnostics_glob.len(),
            0,
            "Glob pattern should put private schema in search path"
        );
    }
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_multi_line_completions(test_db: PgPool) {
    let setup = r#"
        create schema auth;

        create table auth.users (
            id serial primary key,
            email text not null
        );
    "#;

    test_db.execute(setup).await.expect("setup sql failed");

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

    let (workspace, project_key) =
        get_test_workspace(Some(conf)).expect("Unable to create test workspace");

    let path = PgLSPath::new("test.sql");

    let content = r#"
select * from auth.users;

select * from auth.|;

select * from auth.users;
    "#
    .trim();

    let position = content
        .find('|')
        .map(|idx| pgls_text_size::TextSize::new(idx as u32))
        .expect("Unable to find cursor position in test content");

    let sanitized_content = content.replace('|', "");

    workspace
        .open_file(OpenFileParams {
            project_key,
            path: path.clone(),
            content: sanitized_content,
            version: 1,
        })
        .expect("Unable to open test file");

    let completions = workspace
        .get_completions(crate::workspace::GetCompletionsParams {
            project_key,
            path: path.clone(),
            position,
        })
        .expect("Unable to request completions");

    assert_eq!(
        completions.items.len(),
        1,
        "Expected one completion response"
    );

    assert_eq!(
        completions.items[0].completion_text.as_ref().unwrap().range,
        TextRange::new(position, position),
        "Expected no syntax diagnostic"
    );
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn completions_see_the_ddl_before_the_statement(test_db: PgPool) {
    test_db
        .execute("create schema auth; create table auth.users (id serial primary key);")
        .await
        .expect("setup sql failed");

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
    let (workspace, project_key) =
        get_test_workspace(Some(conf)).expect("Unable to create test workspace");
    let path = PgLSPath::new("test.sql");

    let labels = |content: &str| {
        let position = content
            .find('|')
            .map(|idx| pgls_text_size::TextSize::new(idx as u32))
            .expect("Unable to find cursor position in test content");
        workspace
            .open_file(OpenFileParams {
                project_key,
                path: path.clone(),
                content: content.replace('|', ""),
                version: 1,
            })
            .expect("Unable to open test file");
        let completions = workspace
            .get_completions(crate::workspace::GetCompletionsParams {
                project_key,
                path: path.clone(),
                position,
            })
            .expect("Unable to request completions");
        workspace
            .close_file(crate::workspace::CloseFileParams {
                project_key,
                path: path.clone(),
            })
            .expect("Unable to close test file");
        completions
            .items
            .into_iter()
            .map(|item| item.label)
            .collect::<Vec<_>>()
    };

    let tables = labels(
        "create table auth.sessions (id int, token text);\n\
         drop table auth.users;\n\
         select * from auth.|;\n\
         create table auth.later (id int);",
    );
    assert!(tables.contains(&"sessions".to_string()), "{tables:?}");
    assert!(!tables.contains(&"users".to_string()), "{tables:?}");
    assert!(!tables.contains(&"later".to_string()), "{tables:?}");

    let columns = labels(
        "create table auth.sessions (id int, token text);\n\
         select tok| from auth.sessions;",
    );
    assert!(columns.contains(&"token".to_string()), "{columns:?}");
}

/// Whether the diagnostic comes from a rule of the `typecheck` group or from the database.
fn is_typecheck(diagnostic: &pgls_diagnostics::serde::Diagnostic) -> bool {
    diagnostic.category().is_some_and(|category| {
        category.name() == "typecheck"
            || category.name().strip_prefix("lint/").is_some_and(|rule| {
                pgls_analyser::METADATA.group_of(rule) == Some(pgls_analyser::TYPECHECK_GROUP)
            })
    })
}

fn get_statement_execution_workspace(
    test_db: &PgPool,
    allow_statement_executions_against: &[String],
) -> (WorkspaceServer, ProjectKey) {
    let mut conf = PartialConfiguration::init();
    conf.merge_with(PartialConfiguration {
        db: Some(PartialDatabaseConfiguration {
            host: Some(test_db.connect_options().get_host().to_string()),
            database: Some(
                test_db
                    .connect_options()
                    .get_database()
                    .unwrap()
                    .to_string(),
            ),
            allow_statement_executions_against: Some(StringSet::from_iter(
                allow_statement_executions_against.iter().cloned(),
            )),
            ..Default::default()
        }),
        ..Default::default()
    });

    get_test_workspace(Some(conf)).expect("Unable to create test workspace")
}

/// Executes the single statement in `sql` through its code action, returning the action's
/// disabled reason and the result.
fn execute_sql(
    workspace: &WorkspaceServer,
    project_key: ProjectKey,
    sql: &str,
) -> (Option<String>, ExecuteStatementResult) {
    let path = PgLSPath::new("execute.sql");
    workspace
        .open_file(OpenFileParams {
            project_key,
            path: path.clone(),
            content: sql.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let action = workspace
        .pull_code_actions(CodeActionsParams {
            project_key,
            path: path.clone(),
            cursor_position: TextSize::new(0),
            only: vec![],
            skip: vec![],
        })
        .expect("Unable to pull code actions")
        .actions
        .into_iter()
        .find_map(|action| match action.kind {
            CodeActionKind::Command(CommandAction {
                category: CommandActionCategory::ExecuteStatement(statement_id),
            }) => Some((action.disabled_reason, statement_id)),
            _ => None,
        });
    let (disabled_reason, statement_id) =
        action.expect("Did not find the execute statement action");

    let result = workspace
        .execute_statement(ExecuteStatementParams {
            project_key,
            path: path.clone(),
            statement_id,
        })
        .expect("Unable to execute statement");

    workspace
        .close_file(crate::workspace::CloseFileParams { project_key, path })
        .expect("Unable to close test file");

    (disabled_reason, result)
}

fn allow_test_host(test_db: &PgPool) -> Vec<String> {
    vec![format!("{}/*", test_db.connect_options().get_host())]
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn execute_statement_is_disabled_without_matching_pattern(test_db: PgPool) {
    let (workspace, project_key) = get_statement_execution_workspace(&test_db, &[]);

    let (disabled_reason, result) = execute_sql(
        &workspace,
        project_key,
        "create table not_created (id int);",
    );

    let host = test_db.connect_options().get_host().to_string();
    let database = test_db
        .connect_options()
        .get_database()
        .unwrap()
        .to_string();
    let reason = format!(
        "Statement execution is disabled for `{host}/{database}`. Add a matching pattern to `db.allowStatementExecutionsAgainst`, e.g. [\"{host}/*\"]."
    );
    assert_eq!(disabled_reason.as_deref(), Some(reason.as_str()));
    assert_eq!(
        result,
        ExecuteStatementResult {
            message: reason,
            ..Default::default()
        }
    );

    let table_exists: bool = sqlx::query_scalar(
        "select exists (select 1 from pg_catalog.pg_tables where tablename = 'not_created')",
    )
    .fetch_one(&test_db)
    .await
    .unwrap();
    assert!(!table_exists, "A disabled statement must not be executed");
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn execute_statement_ignores_invalid_patterns(test_db: PgPool) {
    let mut patterns = vec!["local[host/*".to_string()];
    patterns.extend(allow_test_host(&test_db));
    let (workspace, project_key) = get_statement_execution_workspace(&test_db, &patterns);

    let (disabled_reason, result) = execute_sql(&workspace, project_key, "select 1 as one;");

    assert_eq!(disabled_reason, None);
    assert_eq!(result.rows, [[Some("1".to_string())]]);
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn execute_statement_returns_rows(test_db: PgPool) {
    let (workspace, project_key) =
        get_statement_execution_workspace(&test_db, &allow_test_host(&test_db));

    let (_, result) = execute_sql(
        &workspace,
        project_key,
        "select 1 as id, null::text as name, array[1, 2] as numbers, interval '1 day' as span, '{\"a\": 1}'::jsonb as data
         union all
         select 2, 'two', null, null, null;",
    );

    assert_eq!(
        result,
        ExecuteStatementResult {
            message: "2 rows".into(),
            columns: ["id", "name", "numbers", "span", "data"]
                .map(String::from)
                .into(),
            rows: vec![
                vec![
                    Some("1".into()),
                    None,
                    Some("{1,2}".into()),
                    Some("1 day".into()),
                    Some("{\"a\": 1}".into()),
                ],
                vec![Some("2".into()), Some("two".into()), None, None, None],
            ],
            truncated: false,
            rows_affected: None,
        }
    );
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn execute_statement_returns_columns_without_rows(test_db: PgPool) {
    let (workspace, project_key) =
        get_statement_execution_workspace(&test_db, &allow_test_host(&test_db));

    let (_, result) = execute_sql(&workspace, project_key, "select 1 as id where false;");

    assert_eq!(
        result,
        ExecuteStatementResult {
            message: "0 rows".into(),
            columns: vec!["id".into()],
            ..Default::default()
        }
    );
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn execute_statement_truncates_rows(test_db: PgPool) {
    let (workspace, project_key) =
        get_statement_execution_workspace(&test_db, &allow_test_host(&test_db));

    let (_, result) = execute_sql(
        &workspace,
        project_key,
        "select generate_series(1, 150) as n;",
    );

    assert!(result.truncated);
    assert_eq!(result.rows.len(), EXECUTE_STATEMENT_MAX_ROWS);
    assert_eq!(result.rows.last(), Some(&vec![Some("100".to_string())]));
    assert_eq!(result.rows_affected, None);
    assert_eq!(result.message, "Showing the first 100 rows (truncated).");

    // The connection is still usable after the dropped rows.
    let (_, result) = execute_sql(&workspace, project_key, "select 'next' as value;");
    assert_eq!(result.rows, [[Some("next".to_string())]]);
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn execute_statement_reports_rows_affected(test_db: PgPool) {
    let (workspace, project_key) =
        get_statement_execution_workspace(&test_db, &allow_test_host(&test_db));

    let (_, result) = execute_sql(&workspace, project_key, "create table executed (id int);");
    assert_eq!(
        result,
        ExecuteStatementResult {
            message: "Successfully executed statement. Rows affected: 0".into(),
            rows_affected: Some(0),
            ..Default::default()
        }
    );

    let (_, result) = execute_sql(
        &workspace,
        project_key,
        "insert into executed values (1), (2);",
    );
    assert_eq!(
        result,
        ExecuteStatementResult {
            message: "Successfully executed statement. Rows affected: 2".into(),
            rows_affected: Some(2),
            ..Default::default()
        }
    );

    let (_, result) = execute_sql(
        &workspace,
        project_key,
        "delete from executed where id = 1 returning id;",
    );
    assert_eq!(result.columns, ["id"]);
    assert_eq!(result.rows, [[Some("1".to_string())]]);
    assert_eq!(result.rows_affected, None);
}

/// A workspace that reads the section markers of a migration tool.
fn section_workspace(section_markers: SectionMarkers) -> (WorkspaceServer, ProjectKey) {
    let mut conf = PartialConfiguration::init();
    conf.merge_with(PartialConfiguration {
        migrations: Some(PartialMigrationsConfiguration {
            section_markers: Some(section_markers),
            ..Default::default()
        }),
        ..Default::default()
    });
    get_test_workspace(Some(conf)).expect("Unable to create test workspace")
}

/// Applies the fixes of the given rules to `content`.
fn fixed(
    workspace: &WorkspaceServer,
    project_key: ProjectKey,
    content: &str,
    fix_file_mode: FixFileMode,
    rules: &[&str],
) -> String {
    let path = PgLSPath::new("test.sql");
    workspace
        .open_file(OpenFileParams {
            project_key,
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");
    let code = workspace
        .fix_file(FixFileParams {
            project_key,
            path: path.clone(),
            fix_file_mode,
            only: rules.iter().map(|rule| rule.parse().unwrap()).collect(),
            skip: vec![],
        })
        .expect("Unable to fix file")
        .code;
    workspace
        .close_file(crate::workspace::CloseFileParams { project_key, path })
        .expect("Unable to close test file");
    code
}

#[test]
fn test_code_actions_fix_and_suppress_lint_diagnostics() {
    // A dbmate section with `transaction:false` and a single statement runs outside a
    // transaction, where `DROP INDEX CONCURRENTLY` works.
    let (workspace, project_key) = section_workspace(SectionMarkers::Dbmate);

    let path = PgLSPath::new("test.sql");
    let content = "-- migrate:up transaction:false\n  drop index users_email_idx;";
    workspace
        .open_file(OpenFileParams {
            project_key,
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let actions = workspace
        .pull_code_actions(crate::features::code_actions::CodeActionsParams {
            project_key,
            path: path.clone(),
            cursor_position: 37.into(),
            only: vec!["safety/requireConcurrentIndexDeletion".parse().unwrap()],
            skip: vec![],
        })
        .expect("Unable to pull code actions")
        .actions;

    let edits: Vec<_> = actions
        .iter()
        .filter_map(|action| match &action.kind {
            CodeActionKind::Edit(edit) => Some((action.title.as_str(), edit)),
            _ => None,
        })
        .collect();
    assert_eq!(edits.len(), 2, "{actions:#?}");

    let (title, fix) = edits[0];
    assert_eq!(title, "Add CONCURRENTLY");
    assert_eq!(fix.applicability, Applicability::Always);
    assert_eq!(
        fix.edits,
        vec![TextEdit {
            range: TextRange::empty(44.into()),
            new_text: " CONCURRENTLY".into(),
        }]
    );

    let (title, suppression) = edits[1];
    assert_eq!(
        title,
        "Suppress rule requireConcurrentIndexDeletion for this statement"
    );
    assert_eq!(
        suppression.edits,
        vec![TextEdit {
            range: TextRange::empty(32.into()),
            new_text: "  -- pgls-ignore requireConcurrentIndexDeletion\n".into(),
        }]
    );
}

#[test]
fn test_no_concurrently_fix_in_a_transaction() {
    // Without section markers, a file with several statements is assumed to run in a
    // transaction, where `DROP INDEX CONCURRENTLY` fails. Only the suppression is offered.
    let (workspace, project_key) =
        get_test_workspace(None).expect("Unable to create test workspace");

    let path = PgLSPath::new("test.sql");
    let content = "select 1;\n  drop index users_email_idx;";
    workspace
        .open_file(OpenFileParams {
            project_key,
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let titles: Vec<_> = workspace
        .pull_code_actions(crate::features::code_actions::CodeActionsParams {
            project_key,
            path: path.clone(),
            cursor_position: 15.into(),
            only: vec!["safety/requireConcurrentIndexDeletion".parse().unwrap()],
            skip: vec![],
        })
        .expect("Unable to pull code actions")
        .actions
        .into_iter()
        .filter(|action| matches!(action.kind, CodeActionKind::Edit(_)))
        .map(|action| action.title)
        .collect();
    assert_eq!(
        titles,
        ["Suppress rule requireConcurrentIndexDeletion for this statement"]
    );
}

#[test]
fn test_fix_file() {
    // goose sends the statements of a `NO TRANSACTION` migration one by one.
    let (workspace, project_key) = section_workspace(SectionMarkers::Goose);
    let content = "-- +goose NO TRANSACTION\n-- +goose Up\ndrop index users_email_idx;\n-- pgls-ignore requireConcurrentIndexDeletion\ndrop index users_name_idx;";
    let path = PgLSPath::new("test.sql");
    workspace
        .open_file(OpenFileParams {
            project_key,
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let fix = |fix_file_mode| {
        workspace
            .fix_file(FixFileParams {
                project_key,
                path: path.clone(),
                fix_file_mode,
                only: vec![
                    "safety/requireConcurrentIndexDeletion".parse().unwrap(),
                    "safety/preferRobustStmts".parse().unwrap(),
                ],
                skip: vec![],
            })
            .expect("Unable to fix file")
    };

    // The unsafe `IF EXISTS` fix only applies once `CONCURRENTLY` was added, in a second pass.
    let safe = fix(FixFileMode::SafeFixes);
    assert_eq!(
        safe.code,
        "-- +goose NO TRANSACTION\n-- +goose Up\ndrop index CONCURRENTLY users_email_idx;\n-- pgls-ignore requireConcurrentIndexDeletion\ndrop index users_name_idx;"
    );
    assert_eq!(safe.fixes_applied, 1);
    assert_eq!(safe.skipped_suggested_fixes, 1);

    let all = fix(FixFileMode::SafeAndUnsafeFixes);
    assert_eq!(
        all.code,
        "-- +goose NO TRANSACTION\n-- +goose Up\ndrop index CONCURRENTLY IF EXISTS users_email_idx;\n-- pgls-ignore requireConcurrentIndexDeletion\ndrop index users_name_idx;"
    );
    assert_eq!(all.fixes_applied, 2);
    assert_eq!(all.skipped_suggested_fixes, 0);
}

#[test]
fn test_no_concurrently_fix_in_a_dbmate_section_with_several_statements() {
    // dbmate sends a section to Postgres as one query, which runs in a transaction block if it
    // has more than one statement.
    let (workspace, project_key) = section_workspace(SectionMarkers::Dbmate);
    let content = "-- migrate:up transaction:false\ncreate index users_email_idx on users (email);\ncreate index users_name_idx on users (name);\n";
    assert_eq!(
        fixed(
            &workspace,
            project_key,
            content,
            FixFileMode::SafeAndUnsafeFixes,
            &["safety/requireConcurrentIndexCreation"]
        ),
        content
    );
}

#[test]
fn test_concurrently_in_a_query_with_several_statements_is_flagged() {
    let flagged = |markers, content| {
        migration_diagnostics(markers, None, content)
            .into_iter()
            .filter(|category| category == "lint/banConcurrentIndexCreationInTransaction")
            .count()
    };
    assert_eq!(
        flagged(
            SectionMarkers::Dbmate,
            "-- migrate:up transaction:false\ncreate index concurrently users_email_idx on users (email);\ncreate index concurrently users_name_idx on users (name);\n\n-- migrate:down transaction:false\ndrop index concurrently users_email_idx;\n"
        ),
        2
    );
    // goose sends a `StatementBegin` block as one query, and the other statements one by one.
    assert_eq!(
        flagged(
            SectionMarkers::Goose,
            "-- +goose NO TRANSACTION\n-- +goose Up\n-- +goose StatementBegin\ncreate index concurrently users_email_idx on users (email);\ncreate index concurrently users_name_idx on users (name);\n-- +goose StatementEnd\ncreate index concurrently users_age_idx on users (age);\n-- +goose StatementBegin\ndrop index concurrently users_id_idx;\n-- +goose StatementEnd\n"
        ),
        2
    );
}

#[test]
fn test_concurrently_in_a_function_body() {
    // A function runs in the transaction of the statement that calls it, so `CONCURRENTLY`
    // fails in its body.
    let (workspace, project_key) = section_workspace(SectionMarkers::Dbmate);
    let content = "-- migrate:up transaction:false\ncreate function add_index() returns void language sql as $$ create index users_email_idx on users (email) $$;\n";
    assert_eq!(
        fixed(
            &workspace,
            project_key,
            content,
            FixFileMode::SafeAndUnsafeFixes,
            &["safety/requireConcurrentIndexCreation"]
        ),
        content
    );

    let diagnostics = migration_diagnostics(
        SectionMarkers::Dbmate,
        None,
        "-- migrate:up transaction:false\ncreate function add_index() returns void language sql as $$ create index concurrently users_email_idx on users (email) $$;\n",
    );
    assert_eq!(
        diagnostics
            .iter()
            .filter(|category| *category == "lint/banConcurrentIndexCreationInTransaction")
            .count(),
        1,
        "{diagnostics:?}"
    );
}

#[test]
fn test_no_concurrently_fix_next_to_an_unparsable_statement() {
    // The file has two statements, even if the second only parses on Postgres 18.
    let (workspace, project_key) =
        get_test_workspace(None).expect("Unable to create test workspace");
    let content = "create index users_email_idx on users (email);\nalter table users add constraint users_age_check check (age > 0) not enforced;\n";
    assert_eq!(
        fixed(
            &workspace,
            project_key,
            content,
            FixFileMode::SafeAndUnsafeFixes,
            &["safety/requireConcurrentIndexCreation"]
        ),
        content
    );
}

#[test]
fn test_concurrently_fix_is_unsafe_without_section_markers() {
    // We don't know whether the migration tool runs a file with a single statement in a
    // transaction.
    let (workspace, project_key) =
        get_test_workspace(None).expect("Unable to create test workspace");
    let content = "create index users_email_idx on users (email);\n";
    let rules = &["safety/requireConcurrentIndexCreation"];
    assert_eq!(
        fixed(
            &workspace,
            project_key,
            content,
            FixFileMode::SafeFixes,
            rules
        ),
        content
    );
    assert_eq!(
        fixed(
            &workspace,
            project_key,
            content,
            FixFileMode::SafeAndUnsafeFixes,
            rules
        ),
        "create index CONCURRENTLY users_email_idx on users (email);\n"
    );
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_no_concurrently_fix_on_partitioned_tables(test_db: PgPool) {
    test_db
        .execute(
            "create table events (id bigint, created_at timestamptz) partition by range (created_at);
             create index events_created_at_idx on events (created_at);
             create table users (id bigint, email text);
             create index users_email_idx on users (email);",
        )
        .await
        .expect("Failed to setup test database");

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
        migrations: Some(PartialMigrationsConfiguration {
            section_markers: Some(SectionMarkers::Goose),
            ..Default::default()
        }),
        ..Default::default()
    });
    let (workspace, project_key) =
        get_test_workspace(Some(conf)).expect("Unable to create test workspace");

    // Postgres can't create or drop the index of a partitioned table concurrently.
    let content = "-- +goose NO TRANSACTION\n-- +goose Up\ncreate index events_id_idx on events (id);\ndrop index events_created_at_idx;\ncreate index users_id_idx on users (id);\ndrop index users_email_idx;\n";
    assert_eq!(
        fixed(
            &workspace,
            project_key,
            content,
            FixFileMode::SafeFixes,
            &[
                "safety/requireConcurrentIndexCreation",
                "safety/requireConcurrentIndexDeletion"
            ]
        ),
        "-- +goose NO TRANSACTION\n-- +goose Up\ncreate index events_id_idx on events (id);\ndrop index events_created_at_idx;\ncreate index CONCURRENTLY users_id_idx on users (id);\ndrop index CONCURRENTLY users_email_idx;\n"
    );
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_inlay_hints(test_db: PgPool) {
    test_db
        .execute(
            "create function db_point(x int, y int) returns int language sql as 'select x + y';",
        )
        .await
        .expect("Failed to setup test database");

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
    let (workspace, project_key) =
        get_test_workspace(Some(conf)).expect("Unable to create test workspace");

    let path = PgLSPath::new("test.sql");
    let content = "create function file_point(a int, b int) returns int language sql as 'select a + b';\nselect db_point(1, y => 2), file_point(3, 4);\nselect db_point(5, 6);\n";
    workspace
        .open_file(OpenFileParams {
            project_key,
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let second_line = content.find("select").unwrap();
    let third_line = content.rfind("select").unwrap();
    let hints = workspace
        .pull_inlay_hints(PullInlayHintsParams {
            project_key,
            path: path.clone(),
            range: TextRange::new(
                u32::try_from(second_line).unwrap().into(),
                u32::try_from(third_line).unwrap().into(),
            ),
        })
        .expect("Unable to pull inlay hints")
        .hints;

    let offset_of = |needle: &str| u32::try_from(content.find(needle).unwrap()).unwrap().into();
    assert_eq!(
        hints,
        vec![
            InlayHint {
                offset: offset_of("1, y"),
                label: "x:".into(),
                kind: InlayHintKind::Parameter,
            },
            InlayHint {
                offset: offset_of("3, 4"),
                label: "a:".into(),
                kind: InlayHintKind::Parameter,
            },
            InlayHint {
                offset: offset_of("4)"),
                label: "b:".into(),
                kind: InlayHintKind::Parameter,
            },
        ]
    );
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_hover_shows_resolved_references(test_db: PgPool) {
    test_db
        .execute(
            "create function db_pick(i int) returns int language sql as 'select i';
             create function db_pick(t text) returns text language sql as 'select t';
             create table hover_items (id int, label text);
             create table hover_tags (id int, label text);",
        )
        .await
        .expect("Failed to setup test database");

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
    let (workspace, project_key) =
        get_test_workspace(Some(conf)).expect("Unable to create test workspace");

    let path = PgLSPath::new("test.sql");
    let content = "create function file_pick(i int) returns int language sql as 'select i';\n\
        create function file_pick(t text) returns text language sql as 'select t';\n\
        select db_pick(1), file_pick('a'::text);\n\
        create function item_label(item_id int) returns text language sql\n\
        as 'select label from hover_items where id = item_id';\n\
        select t.label from hover_items i join hover_tags t on t.id = i.id;\n\
        select label as ambiguous from hover_items, hover_tags;\n";
    workspace
        .open_file(OpenFileParams {
            project_key,
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let hover = |needle: &str| {
        let offset = content.find(needle).expect("needle in content") + 2;
        workspace
            .on_hover(OnHoverParams {
                project_key,
                path: path.clone(),
                position: u32::try_from(offset).unwrap().into(),
            })
            .expect("Unable to hover")
            .markdown_blocks
    };

    // Only the overload the call resolves to.
    let blocks = hover("db_pick(1)");
    assert_eq!(blocks.len(), 1, "{blocks:?}");
    assert!(blocks[0].contains("public.db_pick(i integer) → integer"));

    // Functions created earlier in the file.
    let blocks = hover("file_pick('a'");
    assert_eq!(blocks.len(), 1, "{blocks:?}");

    // A column in the body of a SQL function.
    let blocks = hover("label from hover_items");
    assert_eq!(blocks.len(), 1, "{blocks:?}");
    assert!(blocks[0].contains("public.hover_items.label"));

    // The column of the joined table, by its alias.
    let blocks = hover("label from hover_items i");
    assert_eq!(blocks.len(), 1, "{blocks:?}");
    assert!(blocks[0].contains("public.hover_tags.label"));

    // Without a unique reference, hover falls back to the columns with the hovered name.
    let blocks = hover("label as ambiguous");
    assert_eq!(blocks.len(), 2, "{blocks:?}");
}

/// Hover and code actions on a file with thousands of statements, on a thread with the 2 MiB
/// stack of the language server's tokio workers. They used to recurse once per statement before
/// or after the cursor and overflow the stack, which aborts the server.
#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn hover_and_code_actions_on_long_files(test_db: PgPool) {
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
    let (workspace, project_key) =
        get_test_workspace(Some(conf)).expect("Unable to create test workspace");

    let path = PgLSPath::new("test.sql");
    let content = format!(
        "create table long_file_items (id int);\n{}",
        "select id from long_file_items;\n".repeat(5_000)
    );
    workspace
        .open_file(OpenFileParams {
            project_key,
            path: path.clone(),
            content: content.clone(),
            version: 1,
        })
        .expect("Unable to open test file");

    let first = u32::try_from(content.find("id from").unwrap()).unwrap();
    let last = u32::try_from(content.rfind("id from").unwrap()).unwrap();
    std::thread::scope(|scope| {
        std::thread::Builder::new()
            .stack_size(2 * 1024 * 1024)
            .spawn_scoped(scope, || {
                for position in [first, last] {
                    let blocks = workspace
                        .on_hover(OnHoverParams {
                            project_key,
                            path: path.clone(),
                            position: position.into(),
                        })
                        .expect("Unable to hover")
                        .markdown_blocks;
                    assert_eq!(blocks.len(), 1, "{blocks:?}");
                    assert!(
                        blocks[0].contains("public.long_file_items.id"),
                        "{blocks:?}"
                    );

                    let actions = workspace
                        .pull_code_actions(crate::features::code_actions::CodeActionsParams {
                            project_key,
                            path: path.clone(),
                            cursor_position: position.into(),
                            only: vec![],
                            skip: vec![],
                        })
                        .expect("Unable to pull code actions")
                        .actions;
                    let executes: Vec<_> = actions
                        .iter()
                        .filter(|action| action.title.starts_with("Execute Statement"))
                        .collect();
                    assert_eq!(executes.len(), 1, "{actions:#?}");
                }
            })
            .unwrap()
            .join()
            .unwrap();
    });
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn test_inlay_hints_follow_migration_sections(test_db: PgPool) {
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
        migrations: Some(PartialMigrationsConfiguration {
            section_markers: Some(SectionMarkers::Dbmate),
            ..Default::default()
        }),
        ..Default::default()
    });
    let (workspace, project_key) =
        get_test_workspace(Some(conf)).expect("Unable to create test workspace");

    let path = PgLSPath::new("20250301120000_migration.sql");
    let content = "-- migrate:up\n\
        create function up_point(x int, y int) returns int language sql as 'select x + y';\n\
        -- migrate:down\n\
        create function down_point(a int, b int) returns int language sql as 'select a + b';\n\
        select up_point(1, 2), down_point(3, 4);\n\
        -- migrate:up\n\
        select up_point(5, 6), down_point(7, 8);\n";
    workspace
        .open_file(OpenFileParams {
            project_key,
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let hints = workspace
        .pull_inlay_hints(PullInlayHintsParams {
            project_key,
            path: path.clone(),
            range: TextRange::new(0.into(), u32::try_from(content.len()).unwrap().into()),
        })
        .expect("Unable to pull inlay hints")
        .hints;

    // The down section sees the up section before it, and its own changes don't carry into the
    // next up section.
    let labels: Vec<_> = hints
        .iter()
        .map(|hint| {
            let offset = usize::from(hint.offset);
            format!("{}{}", hint.label, &content[offset..offset + 1])
        })
        .collect();
    assert_eq!(labels, ["x:1", "y:2", "a:3", "b:4", "x:5", "y:6"]);
}

/// The edit of the action that suppresses `rule` at `cursor`, and the lint diagnostics of `rule`
/// that are left once it is applied.
fn suppress(content: &str, cursor: usize, rule: &str) -> (TextEdit, usize) {
    let (workspace, project_key) =
        get_test_workspace(None).expect("Unable to create test workspace");
    let path = PgLSPath::new("test.sql");
    workspace
        .open_file(OpenFileParams {
            project_key,
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");
    let actions = workspace
        .pull_code_actions(crate::features::code_actions::CodeActionsParams {
            project_key,
            path: path.clone(),
            cursor_position: u32::try_from(cursor).unwrap().into(),
            only: vec![rule.parse().unwrap()],
            skip: vec![],
        })
        .expect("Unable to pull code actions")
        .actions;
    let edit = actions
        .into_iter()
        .find_map(|action| match action.kind {
            CodeActionKind::Edit(edit) if action.title.starts_with("Suppress rule") => {
                Some(edit.edits.into_iter().next().unwrap())
            }
            _ => None,
        })
        .expect("expected a suppression action");

    let mut suppressed = content.to_string();
    suppressed.replace_range(std::ops::Range::<usize>::from(edit.range), &edit.new_text);
    let path = PgLSPath::new("suppressed.sql");
    workspace
        .open_file(OpenFileParams {
            project_key,
            path: path.clone(),
            content: suppressed,
            version: 1,
        })
        .expect("Unable to open test file");
    let left = workspace
        .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
            project_key,
            path,
            categories: RuleCategories::all(),
            max_diagnostics: 100,
            only: vec![rule.parse().unwrap()],
            skip: vec![],
        })
        .expect("Unable to pull diagnostics")
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic
                .category()
                .is_some_and(|category| category.name() == format!("lint/{rule}"))
        })
        .count();
    (edit, left)
}

#[test]
fn test_suppression_goes_above_the_statement() {
    // The diagnostic is on the third line of the statement.
    let long = "a".repeat(70);
    let content =
        format!("select 1;\n  create table users (\n    id bigint,\n    {long} text\n  );\n");
    let (edit, left) = suppress(
        &content,
        content.find(&long).unwrap() + 1,
        "identifierTooLong",
    );
    assert_eq!(
        edit,
        TextEdit {
            range: TextRange::empty(10.into()),
            new_text: "  -- pgls-ignore identifierTooLong\n".into(),
        }
    );
    assert_eq!(left, 0);
}

#[test]
fn test_suppression_of_a_statement_in_a_function_body() {
    let content =
        "create function drop_users() returns void language sql\nas $$\n  drop table users;\n$$;\n";
    let (edit, left) = suppress(content, content.find("drop table").unwrap(), "banDropTable");
    assert_eq!(
        edit,
        TextEdit {
            range: TextRange::empty(0.into()),
            new_text: "-- pgls-ignore banDropTable\n".into(),
        }
    );
    assert_eq!(left, 0);
}

#[test]
fn test_suppression_of_a_statement_in_the_middle_of_a_line() {
    let content = "  select 1; drop table users;\n";
    let (edit, left) = suppress(content, content.find("drop").unwrap(), "banDropTable");
    assert_eq!(
        edit,
        TextEdit {
            range: TextRange::new(11.into(), 12.into()),
            new_text: "\n  -- pgls-ignore banDropTable\n  ".into(),
        }
    );
    assert_eq!(left, 0);
}

#[test]
fn test_suppression_keeps_crlf_line_endings() {
    let content = "select 1;\r\ndrop table users;\r\n";
    let (edit, left) = suppress(content, 13, "banDropTable");
    assert_eq!(edit.new_text, "-- pgls-ignore banDropTable\r\n");
    assert_eq!(left, 0);
}

/// A project of a workspace, set up like the language server does it.
struct TestProject {
    workspace: Rc<WorkspaceServer>,
    project_key: ProjectKey,
}

impl TestProject {
    /// A project at `root`, with the configuration file at or above it as its root settings and
    /// every configuration file below it as nested settings. Returns the folders of the nested
    /// settings.
    fn new(fs: &DynRef<'_, dyn FileSystem>, root: &Path) -> (Self, Vec<PathBuf>) {
        let project = Self::without_nested_configurations(fs, root);
        let directories = project.apply_nested_configurations(fs, root);
        (project, directories)
    }

    /// A project at `root`, with the configuration file at or above it as its root settings.
    fn without_nested_configurations(fs: &DynRef<'_, dyn FileSystem>, root: &Path) -> Self {
        let workspace = WorkspaceServer::new();
        let project_key = workspace
            .register_project_folder(RegisterProjectFolderParams {
                path: Some(root.to_path_buf()),
            })
            .unwrap();
        workspace
            .update_settings(UpdateSettingsParams {
                project_key,
                configuration: load_configuration(
                    fs,
                    ConfigurationPathHint::FromWorkspace(root.to_path_buf()),
                )
                .unwrap()
                .configuration,
                gitignore_matches: vec![],
                vcs_base_path: None,
                workspace_directory: Some(root.to_path_buf()),
            })
            .unwrap();

        Self {
            workspace: Rc::new(workspace),
            project_key,
        }
    }

    /// Applies the configuration files the search finds. Returns their folders and whether the
    /// search was incomplete.
    fn search(
        &self,
        fs: &DynRef<'_, dyn FileSystem>,
        search: NestedConfigurationSearch,
    ) -> (Vec<PathBuf>, Option<IncompleteSearch>) {
        let nested =
            apply_nested_configurations(&*self.workspace, fs, self.project_key, search, |c| c)
                .unwrap();
        let directories = nested
            .configurations
            .into_iter()
            .map(|nested| nested.directory)
            .collect();
        (directories, nested.incomplete)
    }

    /// Another project of the same workspace.
    fn other(&self, project_key: ProjectKey) -> Self {
        Self {
            workspace: Rc::clone(&self.workspace),
            project_key,
        }
    }

    /// Applies the configuration files below `root`, which have to load. Returns their folders.
    fn apply_nested_configurations(
        &self,
        fs: &DynRef<'_, dyn FileSystem>,
        root: &Path,
    ) -> Vec<PathBuf> {
        apply_nested_configurations(
            &*self.workspace,
            fs,
            self.project_key,
            NestedConfigurationSearch::new(root),
            |configuration| configuration,
        )
        .unwrap()
        .configurations
        .into_iter()
        .map(|nested| {
            assert!(nested.result.is_ok(), "{:?}", nested.result);
            nested.directory
        })
        .collect()
    }

    /// The severities of the `banDropColumn` diagnostics of a file with a `DROP COLUMN`.
    fn severities(&self, path: impl AsRef<Path>) -> Vec<Severity> {
        self.severities_of(path, "alter table users drop column name;")
    }

    /// The severities of the `banDropColumn` diagnostics of a file with the given content.
    fn severities_of(&self, path: impl AsRef<Path>, content: &str) -> Vec<Severity> {
        let path = PgLSPath::new(path.as_ref());
        self.workspace
            .open_file(OpenFileParams {
                project_key: self.project_key,
                path: path.clone(),
                content: content.into(),
                version: 1,
            })
            .expect("Unable to open test file");

        self.workspace
            .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
                project_key: self.project_key,
                path,
                categories: RuleCategories::all(),
                max_diagnostics: 100,
                only: vec![],
                skip: vec![],
            })
            .expect("Unable to pull diagnostics")
            .diagnostics
            .iter()
            .filter(|diagnostic| {
                diagnostic
                    .category()
                    .is_some_and(|category| category.name() == "lint/banDropColumn")
            })
            .map(|diagnostic| diagnostic.severity())
            .collect()
    }

    fn is_ignored(&self, path: impl AsRef<Path>) -> bool {
        self.workspace
            .is_path_ignored(IsPathIgnoredParams {
                project_key: self.project_key,
                pgls_path: PgLSPath::new(path.as_ref()),
            })
            .unwrap()
    }
}

/// The files settings of the configuration file at or above `root`.
fn root_files(fs: &DynRef<'_, dyn FileSystem>, root: &str) -> Arc<FilesSettings> {
    let configuration = load_configuration(
        fs,
        ConfigurationPathHint::FromWorkspace(PathBuf::from(root)),
    )
    .unwrap()
    .configuration;
    Arc::new(
        FilesSettings::from_configuration(&configuration, Some(PathBuf::from(root)), None, &[])
            .unwrap(),
    )
}

/// A memory file system with a configuration file in each of the given folders.
fn file_system_with_configurations(configurations: &[(&str, &str)]) -> MemoryFileSystem {
    let mut fs = MemoryFileSystem::default();
    for (directory, configuration) in configurations {
        fs.insert(
            Path::new(directory).join(ConfigName::pgls_jsonc()),
            *configuration,
        );
    }
    fs
}

#[test]
fn files_use_the_nearest_configuration_file() {
    let fs = file_system_with_configurations(&[
        ("/repo", r#"{ "files": { "ignore": ["legacy"] } }"#),
        ("/repo/db1", r#"{ "linter": { "enabled": false } }"#),
        (
            "/repo/db2",
            r#"{ "linter": { "rules": { "banDropColumn": "error" } } }"#,
        ),
        ("/repo/db2/nested", r#"{ "linter": { "enabled": false } }"#),
        // Inside a directory the root project ignores.
        ("/repo/legacy", r#"{ "linter": { "enabled": false } }"#),
        // Never looked at.
        (
            "/repo/node_modules/pkg",
            r#"{ "linter": { "enabled": false } }"#,
        ),
    ]);
    let fs: DynRef<'_, dyn FileSystem> = DynRef::Owned(Box::new(fs));

    let (project, directories) = TestProject::new(&fs, Path::new("/repo"));
    assert_eq!(
        directories,
        [
            PathBuf::from("/repo/db1"),
            PathBuf::from("/repo/db2"),
            PathBuf::from("/repo/db2/nested"),
        ]
    );

    assert_eq!(project.severities("/repo/a.sql"), [Severity::Warning]);
    assert_eq!(project.severities("/repo/db1/a.sql"), []);
    assert_eq!(project.severities("/repo/db2/sub/a.sql"), [Severity::Error]);
    assert_eq!(project.severities("/repo/db2/nested/a.sql"), []);
    assert_eq!(
        project.severities("/repo/node_modules/pkg/a.sql"),
        [Severity::Warning]
    );
    // A relative path is relative to the root of the project, like the paths of the CLI.
    assert_eq!(project.severities("db2/a.sql"), [Severity::Error]);
    assert!(project.is_ignored("/repo/legacy/a.sql"));
}

/// The search doesn't descend into the folders that the root configuration ignores, except below
/// a nested configuration file: its settings decide there, so the configuration files below it
/// are still found.
#[test]
fn the_search_skips_the_folders_the_root_configuration_ignores() {
    let fs = file_system_with_configurations(&[
        (
            "/repo",
            r#"{ "files": { "ignore": ["legacy", "db1/generated"] } }"#,
        ),
        ("/repo/legacy/app", "{}"),
        ("/repo/db1", "{}"),
        (
            "/repo/db1/generated",
            r#"{ "linter": { "enabled": false } }"#,
        ),
    ]);
    let fs: DynRef<'_, dyn FileSystem> = DynRef::Owned(Box::new(fs));
    let project = TestProject::without_nested_configurations(&fs, Path::new("/repo"));

    let (directories, incomplete) = project.search(
        &fs,
        NestedConfigurationSearch {
            root_files: Some(root_files(&fs, "/repo")),
            ..NestedConfigurationSearch::new("/repo")
        },
    );
    assert_eq!(
        directories,
        [
            PathBuf::from("/repo/db1"),
            PathBuf::from("/repo/db1/generated")
        ]
    );
    assert_eq!(incomplete, None);
    assert_eq!(project.severities("/repo/db1/generated/a.sql"), []);
}

/// The home directory and the filesystem roots hold unrelated projects, so their configuration
/// files are not applied, and the search says so.
#[test]
fn the_search_skips_the_home_directory() {
    let mut fs = file_system_with_configurations(&[(
        "/home/alice/project",
        r#"{ "linter": { "enabled": false } }"#,
    )]);
    fs.set_home_directory("/home/alice");
    let fs: DynRef<'_, dyn FileSystem> = DynRef::Owned(Box::new(fs));

    let project = TestProject::without_nested_configurations(&fs, Path::new("/home/alice"));
    let (directories, incomplete) =
        project.search(&fs, NestedConfigurationSearch::new("/home/alice"));
    assert_eq!(directories, Vec::<PathBuf>::new());
    assert_eq!(
        incomplete,
        Some(IncompleteSearch::NotAProjectFolder(PathBuf::from(
            "/home/alice"
        )))
    );
    assert_eq!(
        project.severities("/home/alice/project/a.sql"),
        [Severity::Warning]
    );

    let project = TestProject::without_nested_configurations(&fs, Path::new("/"));
    let (directories, incomplete) = project.search(&fs, NestedConfigurationSearch::new("/"));
    assert_eq!(directories, Vec::<PathBuf>::new());
    assert_eq!(
        incomplete,
        Some(IncompleteSearch::NotAProjectFolder(PathBuf::from("/")))
    );
}

/// A search for some paths, like the CLI's, only finds the configuration files on the way to
/// each path and below the paths that are folders.
#[test]
fn the_search_for_paths_only_looks_at_their_folders() {
    let mut fs = file_system_with_configurations(&[
        ("/repo", "{}"),
        ("/repo/db1", "{}"),
        ("/repo/db1/sub", "{}"),
        ("/repo/db2", "{}"),
        ("/repo/db2/nested", "{}"),
        ("/repo/db3", "{ this is not json"),
    ]);
    fs.insert(PathBuf::from("/repo/db1/sub/a.sql"), "select 1;");
    fs.insert(PathBuf::from("/repo/db2/b.sql"), "select 1;");
    let fs: DynRef<'_, dyn FileSystem> = DynRef::Owned(Box::new(fs));
    let project = TestProject::without_nested_configurations(&fs, Path::new("/repo"));

    let (directories, incomplete) = project.search(
        &fs,
        NestedConfigurationSearch {
            paths: Some(vec![
                PathBuf::from("db1/sub/a.sql"),
                PathBuf::from("/repo/db1/sub/b.sql"),
                PathBuf::from("/repo/db2"),
                PathBuf::from("/elsewhere/c.sql"),
            ]),
            ..NestedConfigurationSearch::new("/repo")
        },
    );
    assert_eq!(
        directories,
        [
            PathBuf::from("/repo/db1"),
            PathBuf::from("/repo/db2"),
            PathBuf::from("/repo/db1/sub"),
            PathBuf::from("/repo/db2/nested"),
        ]
    );
    assert_eq!(incomplete, None);
}

/// The search stops at its depth limit and says so.
#[test]
fn the_search_reports_its_depth_limit() {
    let deep = format!(
        "/repo/{}",
        ["d"; FindFiles::DEFAULT_MAX_DEPTH + 1].join("/")
    );
    let fs =
        file_system_with_configurations(&[("/repo", "{}"), ("/repo/db1", "{}"), (&deep, "{}")]);
    let fs: DynRef<'_, dyn FileSystem> = DynRef::Owned(Box::new(fs));
    let project = TestProject::without_nested_configurations(&fs, Path::new("/repo"));

    let (directories, incomplete) = project.search(&fs, NestedConfigurationSearch::new("/repo"));
    assert_eq!(directories, [PathBuf::from("/repo/db1")]);
    assert_eq!(
        incomplete,
        Some(IncompleteSearch::LimitReached(PathBuf::from("/repo")))
    );
}

/// A change of a configuration file only reloads the configuration if the search looks at its
/// folder.
#[test]
fn configuration_changes_in_skipped_folders_are_ignored() {
    let fs = file_system_with_configurations(&[
        (
            "/repo",
            r#"{ "files": { "ignore": ["legacy", "db1/generated"] } }"#,
        ),
        ("/repo/db1", "{}"),
    ]);
    let fs: DynRef<'_, dyn FileSystem> = DynRef::Owned(Box::new(fs));
    let root_files = root_files(&fs, "/repo");
    let is_searched = |directory: &str| {
        is_searched_for_configuration(
            &fs,
            Path::new("/repo"),
            Some(&root_files),
            Path::new(directory),
        )
    };

    assert!(is_searched("/repo"));
    assert!(is_searched("/repo/db2"));
    assert!(is_searched("/repo/db1/generated"));
    assert!(!is_searched("/repo/legacy"));
    assert!(!is_searched("/repo/legacy/app"));
    assert!(!is_searched("/repo/node_modules/pkg"));
    assert!(!is_searched("/repo/.git"));
    assert!(!is_searched("/elsewhere"));
}

/// Applying the nested configuration files again replaces the ones from before, so a deleted
/// configuration file no longer applies.
#[test]
fn nested_settings_replace_the_previous_ones() {
    let mut fs = file_system_with_configurations(&[
        ("/repo", "{}"),
        ("/repo/db1", r#"{ "linter": { "enabled": false } }"#),
    ]);
    fs.remove(&Path::new("/repo/db1").join(ConfigName::pgls_jsonc()));
    let without_db1: DynRef<'_, dyn FileSystem> = DynRef::Owned(Box::new(fs));
    let with_db1: DynRef<'_, dyn FileSystem> =
        DynRef::Owned(Box::new(file_system_with_configurations(&[
            ("/repo", "{}"),
            ("/repo/db1", r#"{ "linter": { "enabled": false } }"#),
        ])));

    let (project, _) = TestProject::new(&with_db1, Path::new("/repo"));
    assert_eq!(project.severities("/repo/db1/a.sql"), []);

    assert_eq!(
        project.apply_nested_configurations(&without_db1, Path::new("/repo")),
        Vec::<PathBuf>::new()
    );
    assert_eq!(project.severities("/repo/db1/a.sql"), [Severity::Warning]);
}

/// The files below a configuration file that cannot be applied use the defaults, not the settings
/// above it, which could point at another database.
#[test]
fn a_nested_configuration_that_cannot_be_applied_uses_the_defaults() {
    let fs = file_system_with_configurations(&[
        (
            "/repo",
            r#"{ "linter": { "rules": { "banDropColumn": "error" } } }"#,
        ),
        ("/repo/broken", r#"{ "files": { "ignore": ["[invalid"] } }"#),
        ("/repo/unreadable", r#"{ "linter": "#),
    ]);
    let fs: DynRef<'_, dyn FileSystem> = DynRef::Owned(Box::new(fs));
    let project = TestProject::without_nested_configurations(&fs, Path::new("/repo"));

    let nested = apply_nested_configurations(
        &*project.workspace,
        &fs,
        project.project_key,
        NestedConfigurationSearch::new("/repo"),
        |configuration| configuration,
    )
    .unwrap()
    .configurations;
    let failed: Vec<_> = nested
        .iter()
        .filter(|nested| nested.result.is_err())
        .map(|nested| nested.directory.clone())
        .collect();
    assert_eq!(
        failed,
        [
            PathBuf::from("/repo/broken"),
            PathBuf::from("/repo/unreadable")
        ]
    );

    assert_eq!(project.severities("/repo/a.sql"), [Severity::Error]);
    assert_eq!(
        project.severities("/repo/broken/a.sql"),
        [Severity::Warning]
    );
    assert_eq!(
        project.severities("/repo/unreadable/a.sql"),
        [Severity::Warning]
    );
}

/// Each project has its own nested settings, even when the folder of one is below the other, as
/// with nested workspace folders. Unregistering a project removes its nested settings with it.
#[test]
fn projects_have_their_own_nested_settings() {
    let fs = file_system_with_configurations(&[
        ("/outer", "{}"),
        ("/outer/inner", r#"{ "linter": { "enabled": false } }"#),
    ]);
    let fs: DynRef<'_, dyn FileSystem> = DynRef::Owned(Box::new(fs));
    let (outer, directories) = TestProject::new(&fs, Path::new("/outer"));
    assert_eq!(directories, [PathBuf::from("/outer/inner")]);

    let inner = outer.other(
        outer
            .workspace
            .register_project_folder(RegisterProjectFolderParams {
                path: Some(PathBuf::from("/outer/inner")),
            })
            .unwrap(),
    );
    inner
        .workspace
        .update_settings(UpdateSettingsParams {
            project_key: inner.project_key,
            configuration: serde_json::from_str(
                r#"{ "linter": { "rules": { "banDropColumn": "error" } } }"#,
            )
            .unwrap(),
            gitignore_matches: vec![],
            vcs_base_path: None,
            workspace_directory: None,
        })
        .unwrap();

    // The same file, by the project that the client says it belongs to.
    assert_eq!(outer.severities("/outer/inner/a.sql"), []);
    assert_eq!(inner.severities("/outer/inner/a.sql"), [Severity::Error]);

    outer
        .workspace
        .unregister_project_folder(UnregisterProjectFolderParams {
            path: PgLSPath::new("/outer"),
        })
        .unwrap();
    assert_eq!(inner.severities("/outer/inner/a.sql"), [Severity::Error]);
    assert!(matches!(
        outer
            .workspace
            .update_nested_settings(UpdateNestedSettingsParams {
                project_key: outer.project_key,
                nested: vec![],
            }),
        Err(WorkspaceError::NoProject(_))
    ));
}

/// Registering a folder again reuses its project instead of adding one that shadows it.
#[test]
fn registering_a_project_folder_again_reuses_its_project() {
    let workspace = WorkspaceServer::new();
    let register = |path: &str| {
        workspace
            .register_project_folder(RegisterProjectFolderParams {
                path: Some(PathBuf::from(path)),
            })
            .unwrap()
    };

    let first = register("/one");
    let second = register("/two");
    assert_ne!(first, second);
    assert_eq!(register("/one"), first);
}

#[test]
fn updating_the_settings_of_an_unknown_project_fails() {
    let workspace = WorkspaceServer::new();
    let result = workspace.update_settings(UpdateSettingsParams {
        project_key: ProjectKey::default(),
        configuration: PartialConfiguration::init(),
        gitignore_matches: vec![],
        vcs_base_path: None,
        workspace_directory: None,
    });
    assert!(matches!(result, Err(WorkspaceError::NoProject(_))));
}

/// The user configuration only applies to the files without a project configuration at or above
/// them, as the root settings. Nested configurations never merge it implicitly, but can extend it,
/// and its directory is not a nested configuration even when it is below the root.
#[test]
fn nested_configurations_do_not_merge_the_user_configuration() {
    let home = PathBuf::from("/home/alice");
    let user_config_directory = home
        .join("Library")
        .join("Application Support")
        .join("postgres-language-server");

    let mut fs = MemoryFileSystem::default();
    fs.set_home_directory(home.clone());
    fs.set_user_config_directories(vec![user_config_directory.clone()]);
    fs.insert(
        user_config_directory.join(ConfigName::pgls_jsonc()),
        r#"{ "linter": { "rules": { "banDropColumn": "error" } } }"#,
    );
    fs.insert(home.join("repo/db").join(ConfigName::pgls_jsonc()), "{}");
    fs.insert(
        home.join("repo/shared").join(ConfigName::pgls_jsonc()),
        r#"{ "extends": ["~/Library/Application Support/postgres-language-server/postgres-language-server.jsonc"] }"#,
    );
    let fs: DynRef<'_, dyn FileSystem> = DynRef::Owned(Box::new(fs));

    // The workspace has no project configuration.
    let root = home.join("repo");
    let loaded =
        load_configuration(&fs, ConfigurationPathHint::FromWorkspace(root.clone())).unwrap();
    assert_eq!(
        loaded.file_path(),
        Some(
            user_config_directory
                .join(ConfigName::pgls_jsonc())
                .as_path()
        )
    );

    let (project, directories) = TestProject::new(&fs, &root);
    assert_eq!(
        directories,
        [home.join("repo/db"), home.join("repo/shared")]
    );

    // No project configuration: the user configuration applies.
    assert_eq!(
        project.severities(home.join("repo/a.sql")),
        [Severity::Error]
    );
    // A nested configuration does not merge it, unless it extends it.
    assert_eq!(
        project.severities(home.join("repo/db/a.sql")),
        [Severity::Warning]
    );
    assert_eq!(
        project.severities(home.join("repo/shared/a.sql")),
        [Severity::Error]
    );
}

/// A user configuration directory below the workspace, e.g. with `XDG_CONFIG_HOME` pointing into
/// it, holds the user configuration, not a nested one.
#[test]
fn the_user_configuration_directories_are_not_nested_configurations() {
    let mut fs = file_system_with_configurations(&[
        ("/repo", "{}"),
        (
            "/repo/config/postgres-language-server",
            r#"{ "linter": { "enabled": false } }"#,
        ),
    ]);
    fs.set_user_config_directories(vec![PathBuf::from("/repo/config/postgres-language-server")]);
    let fs: DynRef<'_, dyn FileSystem> = DynRef::Owned(Box::new(fs));

    let (project, directories) = TestProject::new(&fs, Path::new("/repo"));
    assert_eq!(directories, Vec::<PathBuf>::new());
    assert_eq!(
        project.severities("/repo/config/postgres-language-server/a.sql"),
        [Severity::Warning]
    );
}

/// The migrations settings of a file come from its nearest configuration file: the section markers,
/// `lintDown`, and the migrations directory, which is relative to the nested configuration file.
#[test]
fn nested_configurations_have_their_own_migrations_settings() {
    const DBMATE_DROPS: &str = r#"-- migrate:up
ALTER TABLE users DROP COLUMN name;

-- migrate:down
ALTER TABLE users DROP COLUMN email;
"#;

    let mut fs = MemoryFileSystem::default();
    fs.insert(Path::new("/repo").join(ConfigName::pgls_jsonc()), "{}");
    fs.insert(
        Path::new("/repo/db").join(ConfigName::pgls_jsonc()),
        r#"{
            "migrations": {
                "migrationsDir": "migrations",
                "sectionMarkers": "dbmate",
                "lintDown": false,
                "after": 20240101000000
            }
        }"#,
    );
    let fs: DynRef<'_, dyn FileSystem> = DynRef::Owned(Box::new(fs));
    let (project, directories) = TestProject::new(&fs, Path::new("/repo"));
    assert_eq!(directories, [PathBuf::from("/repo/db")]);

    // The nested configuration skips the down section.
    assert_eq!(
        project.severities_of("/repo/db/migrations/20250301120000_drop.sql", DBMATE_DROPS),
        [Severity::Warning]
    );
    // The root configuration has no section markers, so both drops are checked.
    assert_eq!(
        project.severities_of("/repo/migrations/20250301120000_drop.sql", DBMATE_DROPS),
        [Severity::Warning, Severity::Warning]
    );

    // `after` applies to the migrations directory of the nested configuration only.
    assert!(project.is_ignored("/repo/db/migrations/20230101000000_old.sql"));
    assert!(!project.is_ignored("/repo/db/20230101000000_old.sql"));
    assert!(!project.is_ignored("/repo/migrations/20230101000000_old.sql"));
}
