use std::{
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

#[cfg(not(target_os = "windows"))]
use pgls_configuration::plpgsql_check::PartialPlPgSqlCheckConfiguration;
use pgls_diagnostics::Diagnostic;
use pgls_fs::PgLSPath;
use pgls_text_size::TextRange;
use sqlx::{Executor, PgPool};

use crate::{
    Workspace, WorkspaceError,
    features::code_actions::{
        CodeActionKind, ExecuteStatementResult, FixFileMode, FixFileParams, TextEdit,
    },
    features::format::PullFileFormattingParams,
    features::inlay_hints::{InlayHint, InlayHintKind, PullInlayHintsParams},
    features::on_hover::OnHoverParams,
    workspace::{
        OpenFileParams, RegisterProjectFolderParams, StatementId, UpdateSettingsParams,
        server::WorkspaceServer,
    },
};

fn get_test_workspace(
    partial_config: Option<PartialConfiguration>,
) -> Result<WorkspaceServer, WorkspaceError> {
    let workspace = WorkspaceServer::new();

    workspace.register_project_folder(RegisterProjectFolderParams {
        path: None,
        set_as_current_workspace: true,
    })?;

    workspace.update_settings(UpdateSettingsParams {
        configuration: partial_config.unwrap_or(PartialConfiguration::init()),
        gitignore_matches: vec![],
        vcs_base_path: None,
        workspace_directory: None,
    })?;

    Ok(workspace)
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

    let workspace = get_test_workspace(Some(conf)).expect("Unable to create test workspace");

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
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let diagnostics = workspace
        .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
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
    let workspace = get_test_workspace(None).expect("Unable to create test workspace");

    let path = PgLSPath::new("test.sql");
    let content = r#"ALTER TABLE users
    -- pgt-ignore banDropColumn
    DROP COLUMN deprecated_field;"#;

    workspace
        .open_file(OpenFileParams {
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let diagnostics = workspace
        .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
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
    let workspace = get_test_workspace(Some(conf)).expect("Unable to create test workspace");

    let path = PgLSPath::new("20250301120000_migration.sql");
    workspace
        .open_file(OpenFileParams {
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let diagnostics = workspace
        .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
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

    assert_eq!(
        in_transaction(migration_diagnostics(SectionMarkers::Goose, None, content)),
        2
    );
    let content = format!("-- +goose NO TRANSACTION\n{content}");
    assert_eq!(
        in_transaction(migration_diagnostics(SectionMarkers::Goose, None, &content)),
        0
    );
}

#[test]
fn test_legacy_suppression_still_suppresses_and_warns() {
    let workspace = get_test_workspace(None).expect("Unable to create test workspace");

    let path = PgLSPath::new("test.sql");
    let content = r#"-- pgls-ignore lint/safety/banDropColumn
ALTER TABLE users DROP COLUMN deprecated_field;"#;

    workspace
        .open_file(OpenFileParams {
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let diagnostics = workspace
        .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
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

    let workspace = get_test_workspace(Some(conf)).expect("Unable to create test workspace");

    let path = PgLSPath::new("test.sql");
    let content = "drop table non_existing_table;";

    workspace
        .open_file(OpenFileParams {
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let pull_diagnostics = || {
        workspace
            .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
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

    let workspace = get_test_workspace(Some(conf)).expect("Unable to create test workspace");

    let path = PgLSPath::new("test.sql");
    let content = r#"
      seect 1;
    "#;

    workspace
        .open_file(OpenFileParams {
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let diagnostics = workspace
        .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
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
    let workspace = get_test_workspace(None).expect("Unable to create test workspace");
    let path = PgLSPath::new("named-parameter.sql");
    let content = "SELECT * FROM :very_long_schema.existing_table AS t seect 1;";

    workspace
        .open_file(OpenFileParams {
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let diagnostics = workspace
        .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
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

    let workspace = get_test_workspace(Some(conf)).expect("Unable to create test workspace");
    let path = PgLSPath::new("named-parameter-format.sql");
    let content = "SELECT x FROM :raw_data.t WHERE t.a = :'agen_code' AND t.b = :var_date;";

    workspace
        .open_file(OpenFileParams {
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let result = workspace
        .pull_file_formatting(PullFileFormattingParams { path, range: None })
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

    let workspace = get_test_workspace(Some(conf)).expect("Unable to create test workspace");

    let path = PgLSPath::new("test.sql");
    let content = r#"
      seect 1;
    "#;

    let diagnostics_result =
        workspace.pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
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

    let close_file_result =
        workspace.close_file(crate::workspace::CloseFileParams { path: path.clone() });

    assert!(close_file_result.is_ok());

    let execute_statement_result =
        workspace.execute_statement(crate::workspace::ExecuteStatementParams {
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

    let workspace = get_test_workspace(Some(conf)).expect("Unable to create test workspace");

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
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let diagnostics = workspace
        .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
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

    let workspace = get_test_workspace(Some(conf)).expect("Unable to create test workspace");

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
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let diagnostics = workspace
        .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
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

    let workspace = get_test_workspace(Some(conf)).expect("Unable to create test workspace");

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
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let diagnostics = workspace
        .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
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

    let workspace = get_test_workspace(Some(conf)).expect("Unable to create test workspace");

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
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let diagnostics = workspace
        .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
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

    let workspace = get_test_workspace(Some(conf)).expect("Unable to create test workspace");

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
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let diagnostics = workspace
        .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
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

    let workspace = get_test_workspace(Some(conf)).expect("Unable to create test workspace");

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
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let diagnostics = workspace
        .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
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

    let workspace = get_test_workspace(Some(conf)).expect("Unable to create test workspace");
    let path = PgLSPath::new("test.sql");
    let content = "SELECT 1;\nCREATE VIEW v1 AS SELECT t.nope FROM t;";

    workspace
        .open_file(OpenFileParams {
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let diagnostics = workspace
        .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
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

    let workspace = get_test_workspace(Some(conf)).expect("Unable to create test workspace");
    let path = PgLSPath::new("named-identifier-parameter.sql");
    let content = r#"
SELECT * FROM :raw_data.documents;
SELECT missing_column FROM named_parameter_typecheck_users;
"#;

    workspace
        .open_file(OpenFileParams {
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let diagnostics = workspace
        .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
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
    let workspace = get_test_workspace(Some(conf)).expect("Unable to create test workspace");
    let path = PgLSPath::new("test.sql");
    workspace
        .open_file(OpenFileParams {
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");
    workspace
        .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
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

    let workspace = get_test_workspace(Some(conf)).expect("Unable to create test workspace");

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
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let diagnostics = workspace
        .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
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

    let workspace = get_test_workspace(Some(conf)).expect("Unable to create test workspace");

    let path = PgLSPath::new("test.sql");
    let content =
        "create function add(a int, b int) returns int as 'SELECT 424242+$1+$2;' language sql;";

    workspace
        .open_file(OpenFileParams {
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let result = workspace
        .pull_file_formatting(PullFileFormattingParams {
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

    let workspace = get_test_workspace(Some(conf)).expect("Unable to create test workspace");

    let path = PgLSPath::new("test.sql");
    workspace
        .open_file(OpenFileParams {
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    workspace
        .pull_file_formatting(PullFileFormattingParams { path, range: None })
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

    let workspace = get_test_workspace(Some(conf)).expect("Unable to create test workspace");

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
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let diagnostics = workspace
        .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
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
        let workspace =
            get_test_workspace(Some(glob_conf.clone())).expect("Unable to create test workspace");

        workspace
            .open_file(OpenFileParams {
                path: path_glob.clone(),
                content: file_content.into(),
                version: 1,
            })
            .expect("Unable to open test file");

        let diagnostics_glob = workspace
            .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
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
        let workspace =
            get_test_workspace(Some(glob_conf.clone())).expect("Unable to create test workspace");

        workspace
            .open_file(OpenFileParams {
                path: path_glob.clone(),
                content: file_content.into(),
                version: 1,
            })
            .expect("Unable to open test file");

        let diagnostics_glob = workspace
            .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
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

    let workspace = get_test_workspace(Some(conf)).expect("Unable to create test workspace");

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
            path: path.clone(),
            content: sanitized_content,
            version: 1,
        })
        .expect("Unable to open test file");

    let completions = workspace
        .get_completions(crate::workspace::GetCompletionsParams {
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
    let workspace = get_test_workspace(Some(conf)).expect("Unable to create test workspace");
    let path = PgLSPath::new("test.sql");

    let labels = |content: &str| {
        let position = content
            .find('|')
            .map(|idx| pgls_text_size::TextSize::new(idx as u32))
            .expect("Unable to find cursor position in test content");
        workspace
            .open_file(OpenFileParams {
                path: path.clone(),
                content: content.replace('|', ""),
                version: 1,
            })
            .expect("Unable to open test file");
        let completions = workspace
            .get_completions(crate::workspace::GetCompletionsParams {
                path: path.clone(),
                position,
            })
            .expect("Unable to request completions");
        workspace
            .close_file(crate::workspace::CloseFileParams { path: path.clone() })
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

/// A workspace that reads dbmate section markers, so a `transaction:false` section runs its
/// statements outside a transaction, where `CONCURRENTLY` is allowed.
fn dbmate_workspace() -> WorkspaceServer {
    let mut conf = PartialConfiguration::init();
    conf.merge_with(PartialConfiguration {
        migrations: Some(PartialMigrationsConfiguration {
            section_markers: Some(SectionMarkers::Dbmate),
            ..Default::default()
        }),
        ..Default::default()
    });
    get_test_workspace(Some(conf)).expect("Unable to create test workspace")
}

#[test]
fn test_code_actions_fix_and_suppress_lint_diagnostics() {
    let workspace = dbmate_workspace();

    let path = PgLSPath::new("test.sql");
    let content = "-- migrate:up transaction:false\nselect 1;\n  drop index users_email_idx;";
    workspace
        .open_file(OpenFileParams {
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let actions = workspace
        .pull_code_actions(crate::features::code_actions::CodeActionsParams {
            path: path.clone(),
            cursor_position: 47.into(),
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
            range: TextRange::empty(54.into()),
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
            range: TextRange::empty(42.into()),
            new_text: "  -- pgls-ignore requireConcurrentIndexDeletion\n".into(),
        }]
    );
}

#[test]
fn test_no_concurrently_fix_in_a_transaction() {
    // Without section markers, a file with several statements is assumed to run in a
    // transaction, where `DROP INDEX CONCURRENTLY` fails. Only the suppression is offered.
    let workspace = get_test_workspace(None).expect("Unable to create test workspace");

    let path = PgLSPath::new("test.sql");
    let content = "select 1;\n  drop index users_email_idx;";
    workspace
        .open_file(OpenFileParams {
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let titles: Vec<_> = workspace
        .pull_code_actions(crate::features::code_actions::CodeActionsParams {
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

/// The caches of the least recently used documents are dropped to stay within the budget, and
/// recomputed when the document is used again.
#[test]
fn document_caches_stay_within_the_budget() {
    let mut workspace = get_test_workspace(None).expect("Unable to create test workspace");
    workspace.document_caches = super::cache_budget::CacheBudget::new(20);

    let content = "drop table a;\ndrop table b;\ndrop table c;\nselect 1;\nselect 2;\nselect 3;\n";
    let paths = [PgLSPath::new("a.sql"), PgLSPath::new("b.sql")];
    for path in &paths {
        workspace
            .open_file(OpenFileParams {
                path: path.clone(),
                content: content.into(),
                version: 1,
            })
            .expect("Unable to open test file");
    }
    let diagnostics = |path: &PgLSPath| {
        workspace
            .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
                path: path.clone(),
                categories: RuleCategories::all(),
                max_diagnostics: 100,
                only: vec![],
                skip: vec![],
            })
            .expect("Unable to pull diagnostics")
            .diagnostics
            .len()
    };
    let cached = |path: &PgLSPath| {
        let documents = workspace.documents.read().unwrap();
        let doc = documents.get(path).unwrap();
        (
            doc.cache_entries(),
            doc.analysis().lock().unwrap().is_some(),
        )
    };

    let expected = diagnostics(&paths[0]);
    assert_eq!(expected, 3);
    assert_eq!(cached(&paths[0]), (12, true));

    // Both documents together keep 24 parse results and trees, so the first one's caches are
    // dropped.
    assert_eq!(diagnostics(&paths[1]), expected);
    assert_eq!(cached(&paths[0]), (0, false));
    assert_eq!(cached(&paths[1]), (12, true));

    // And recomputed when it is used again.
    assert_eq!(diagnostics(&paths[0]), expected);
    assert_eq!(cached(&paths[0]), (12, true));
    assert_eq!(cached(&paths[1]), (0, false));

    workspace
        .close_file(crate::workspace::CloseFileParams {
            path: paths[0].clone(),
        })
        .expect("Unable to close test file");
    assert_eq!(workspace.document_caches.total(), 0);
}

/// Code actions are requested on every cursor move. They reuse the analysis that the
/// diagnostics made until the document or the settings change.
#[test]
fn code_actions_reuse_the_analysis() {
    let workspace = get_test_workspace(None).expect("Unable to create test workspace");
    let path = PgLSPath::new("test.sql");
    workspace
        .open_file(OpenFileParams {
            path: path.clone(),
            content: "drop table users;".into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let cached = || {
        let documents = workspace.documents.read().unwrap();
        let analysis = documents.get(&path).unwrap().analysis().lock().unwrap();
        analysis.as_ref().map(|cached| Arc::clone(&cached.analysis))
    };
    let lint_actions = || {
        workspace
            .pull_code_actions(crate::features::code_actions::CodeActionsParams {
                path: path.clone(),
                cursor_position: 3.into(),
                only: vec![],
                skip: vec![],
            })
            .expect("Unable to pull code actions")
            .actions
            .into_iter()
            .filter(|action| matches!(action.kind, CodeActionKind::Edit(_)))
            .map(|action| action.title)
            .collect::<Vec<_>>()
    };

    workspace
        .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
            path: path.clone(),
            categories: RuleCategories::all(),
            max_diagnostics: 100,
            only: vec![],
            skip: vec![],
        })
        .expect("Unable to pull diagnostics");
    let analysis = cached().expect("the analysis of the diagnostics");
    assert_eq!(
        lint_actions(),
        ["Suppress rule banDropTable for this statement"]
    );
    assert!(Arc::ptr_eq(&analysis, &cached().unwrap()));

    // An edit drops the analysis.
    workspace
        .change_file(crate::workspace::ChangeFileParams {
            path: path.clone(),
            version: 2,
            content: "drop table users;\ndrop table posts;".into(),
        })
        .expect("Unable to change test file");
    assert!(cached().is_none());
    assert_eq!(
        lint_actions(),
        ["Suppress rule banDropTable for this statement"]
    );
    let analysis = cached().expect("the analysis of the code actions");

    // So does a change of the settings.
    let mut conf = PartialConfiguration::init();
    conf.merge_with(
        serde_json::from_str::<PartialConfiguration>(
            r#"{"linter": {"rules": {"safety": {"banDropTable": "off"}}}}"#,
        )
        .unwrap(),
    );
    workspace
        .update_settings(UpdateSettingsParams {
            configuration: conf,
            gitignore_matches: vec![],
            vcs_base_path: None,
            workspace_directory: None,
        })
        .expect("Unable to update settings");
    assert!(lint_actions().is_empty());
    assert!(!Arc::ptr_eq(&analysis, &cached().unwrap()));
}

#[test]
fn test_fix_file() {
    let workspace = dbmate_workspace();

    let path = PgLSPath::new("test.sql");
    let content = "-- migrate:up transaction:false\ndrop index users_email_idx;\n-- pgls-ignore requireConcurrentIndexDeletion\ndrop index users_name_idx;";
    workspace
        .open_file(OpenFileParams {
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let fix = |fix_file_mode| {
        workspace
            .fix_file(FixFileParams {
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
        "-- migrate:up transaction:false\ndrop index CONCURRENTLY users_email_idx;\n-- pgls-ignore requireConcurrentIndexDeletion\ndrop index users_name_idx;"
    );
    assert_eq!(safe.fixes_applied, 1);
    assert_eq!(safe.skipped_suggested_fixes, 1);

    let all = fix(FixFileMode::SafeAndUnsafeFixes);
    assert_eq!(
        all.code,
        "-- migrate:up transaction:false\ndrop index CONCURRENTLY IF EXISTS users_email_idx;\n-- pgls-ignore requireConcurrentIndexDeletion\ndrop index users_name_idx;"
    );
    assert_eq!(all.fixes_applied, 2);
    assert_eq!(all.skipped_suggested_fixes, 0);
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
    let workspace = get_test_workspace(Some(conf)).expect("Unable to create test workspace");

    let path = PgLSPath::new("test.sql");
    let content = "create function file_point(a int, b int) returns int language sql as 'select a + b';\nselect db_point(1, y => 2), file_point(3, 4);\nselect db_point(5, 6);\n";
    workspace
        .open_file(OpenFileParams {
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let second_line = content.find("select").unwrap();
    let third_line = content.rfind("select").unwrap();
    let hints = workspace
        .pull_inlay_hints(PullInlayHintsParams {
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
    let workspace = get_test_workspace(Some(conf)).expect("Unable to create test workspace");

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
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let hover = |needle: &str| {
        let offset = content.find(needle).expect("needle in content") + 2;
        workspace
            .on_hover(OnHoverParams {
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

/// Hover shows the objects as the statement sees them: created earlier in the file, changed by
/// the file, or unchanged in the database, also after the file changes.
#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn hover_follows_the_ddl_before_the_statement(test_db: PgPool) {
    test_db
        .execute(
            "create table hover_db_items (id int);
             create table hover_db_tags (id int);",
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
    let workspace = get_test_workspace(Some(conf)).expect("Unable to create test workspace");

    let path = PgLSPath::new("test.sql");
    let statements = "alter table hover_db_items add column extra int;\n\
        select id from hover_file_items;\n\
        select extra from hover_db_items;\n\
        select id from hover_db_tags;\n";
    let hover = |content: &str, version: i32, needle: &str| {
        if version == 1 {
            workspace.open_file(OpenFileParams {
                path: path.clone(),
                content: content.into(),
                version,
            })
        } else {
            workspace.change_file(crate::workspace::ChangeFileParams {
                path: path.clone(),
                content: content.into(),
                version,
            })
        }
        .expect("Unable to update test file");
        let offset = content.find(needle).expect("needle in content") + 2;
        let blocks = workspace
            .on_hover(OnHoverParams {
                path: path.clone(),
                position: u32::try_from(offset).unwrap().into(),
            })
            .expect("Unable to hover")
            .markdown_blocks;
        assert_eq!(blocks.len(), 1, "{blocks:?}");
        blocks.into_iter().next().unwrap()
    };

    let content = format!("create table hover_file_items (id int);\n{statements}");
    let block = hover(&content, 1, "hover_file_items;");
    assert!(block.contains("- id: int4"), "{block}");
    assert!(!block.contains("- name"), "{block}");
    // A table of the database that the file changed.
    let block = hover(&content, 1, "hover_db_items;");
    assert!(block.contains("- extra: int4"), "{block}");
    // A table of the database that the file didn't change.
    let block = hover(&content, 1, "hover_db_tags;");
    assert!(block.contains("public.hover_db_tags"), "{block}");

    // The statements before the hovered one changed.
    let content = format!("create table hover_file_items (id int, name text);\n{statements}");
    let block = hover(&content, 2, "hover_file_items;");
    assert!(block.contains("- name: text"), "{block}");
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
    let workspace = get_test_workspace(Some(conf)).expect("Unable to create test workspace");

    let path = PgLSPath::new("test.sql");
    let content = format!(
        "create table long_file_items (id int);\n{}",
        "select id from long_file_items;\n".repeat(5_000)
    );
    workspace
        .open_file(OpenFileParams {
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
    let workspace = get_test_workspace(Some(conf)).expect("Unable to create test workspace");

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
            path: path.clone(),
            content: content.into(),
            version: 1,
        })
        .expect("Unable to open test file");

    let hints = workspace
        .pull_inlay_hints(PullInlayHintsParams {
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

/// Lint, hover and completions see the same catalog at each statement: after a down section,
/// after a statement that doesn't parse, and after a statement on identifier parameters.
#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn lint_hover_and_completions_see_the_same_catalog(test_db: PgPool) {
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
    let workspace = get_test_workspace(Some(conf)).expect("Unable to create test workspace");
    let path = PgLSPath::new("20250301120000_migration.sql");

    // Opens `content` without the `|`, and returns the typecheck diagnostics of the file, the
    // completions at the `|`, and the hover on each needle.
    let check = |content: &str, needles: &[&str]| {
        let cursor = content.find('|').expect("cursor in content");
        let content = content.replace('|', "");
        workspace
            .open_file(OpenFileParams {
                path: path.clone(),
                content: content.clone(),
                version: 1,
            })
            .expect("Unable to open test file");
        let typecheck: Vec<_> = workspace
            .pull_file_diagnostics(crate::workspace::PullFileDiagnosticsParams {
                path: path.clone(),
                categories: RuleCategories::all(),
                max_diagnostics: 100,
                only: vec![],
                skip: vec![],
            })
            .expect("Unable to pull diagnostics")
            .diagnostics
            .into_iter()
            .filter(is_typecheck)
            .map(|diagnostic| {
                let span = diagnostic.location().span.expect("span");
                content[span].to_string()
            })
            .collect();
        let completions: Vec<_> = workspace
            .get_completions(crate::workspace::GetCompletionsParams {
                path: path.clone(),
                position: u32::try_from(cursor).unwrap().into(),
            })
            .expect("Unable to request completions")
            .items
            .into_iter()
            .map(|item| item.label)
            .collect();
        let hovers: Vec<_> = needles
            .iter()
            .map(|needle| {
                let offset = content.rfind(needle).expect("needle in content") + 2;
                workspace
                    .on_hover(OnHoverParams {
                        path: path.clone(),
                        position: u32::try_from(offset).unwrap().into(),
                    })
                    .expect("Unable to hover")
                    .markdown_blocks
                    .join("\n")
            })
            .collect();
        workspace
            .close_file(crate::workspace::CloseFileParams { path: path.clone() })
            .expect("Unable to close test file");
        (typecheck, completions, hovers)
    };

    // The changes of a down section don't carry into the next up section, also for its first
    // statement.
    let (typecheck, completions, hovers) = check(
        "-- migrate:up\n\
         create table walk_items (id int);\n\
         -- migrate:down\n\
         alter table walk_items add column down_only int;\n\
         -- migrate:up\n\
         select | from walk_items;\n\
         select down_only from walk_items;\n",
        &["walk_items;"],
    );
    assert_eq!(typecheck, ["down_only"]);
    assert!(completions.contains(&"id".to_string()), "{completions:?}");
    assert!(
        !completions.contains(&"down_only".to_string()),
        "{completions:?}"
    );
    assert!(hovers[0].contains("- id: int4"), "{hovers:?}");
    assert!(!hovers[0].contains("down_only"), "{hovers:?}");

    // A statement that doesn't parse may have created anything, but the objects created before
    // it are still known.
    let (typecheck, completions, hovers) = check(
        "create table walk_parsed (id int);\n\
         create tabel walk_unparsed (id int);\n\
         select id from walk_unparsed;\n\
         select | from walk_parsed;\n",
        &["walk_parsed;"],
    );
    assert!(typecheck.is_empty(), "{typecheck:?}");
    assert!(completions.contains(&"id".to_string()), "{completions:?}");
    assert!(hovers[0].contains("- id: int4"), "{hovers:?}");

    // The names of a statement with identifier parameters are placeholders: `:tbl` doesn't
    // create a table `tbl`.
    let (typecheck, completions, hovers) = check(
        "create table :tbl (id int);\n\
         select id from tbl;\n\
         select | from tbl;\n",
        &["tbl;"],
    );
    assert!(typecheck.is_empty(), "{typecheck:?}");
    assert!(!completions.contains(&"id".to_string()), "{completions:?}");
    assert_eq!(hovers, [""]);
}
