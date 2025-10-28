use std::sync::Arc;

use biome_deserialize::{Merge, StringSet};
use pgls_analyse::RuleCategories;
use pgls_configuration::{
    PartialConfiguration, PartialTypecheckConfiguration, database::PartialDatabaseConfiguration,
    files::PartialFilesConfiguration,
};

#[cfg(not(target_os = "windows"))]
use pgls_configuration::plpgsql_check::PartialPlPgSqlCheckConfiguration;
use pgls_diagnostics::Diagnostic;
use pgls_fs::PgLSPath;
use pgls_text_size::TextRange;
use sqlx::{Executor, PgPool};

use crate::{
    Workspace, WorkspaceError,
    features::code_actions::ExecuteStatementResult,
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
        .pull_diagnostics(crate::workspace::PullDiagnosticsParams {
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
        Some("lint/safety/banDropTable")
    );

    assert_eq!(
        diagnostic.location().span,
        Some(TextRange::new(106.into(), 136.into()))
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
        .pull_diagnostics(crate::workspace::PullDiagnosticsParams {
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

    let diagnostics_result = workspace.pull_diagnostics(crate::workspace::PullDiagnosticsParams {
        path: path.clone(),
        categories: RuleCategories::all(),
        max_diagnostics: 100,
        only: vec![],
        skip: vec![],
    });

    assert!(
        diagnostics_result.is_ok_and(|res| res.diagnostics.is_empty()
            && res.errors == 0
            && res.skipped_diagnostics == 0)
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
        .pull_diagnostics(crate::workspace::PullDiagnosticsParams {
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

    assert_eq!(
        diagnostic.location().span,
        Some(TextRange::new(124.into(), 201.into()))
    );
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
        .pull_diagnostics(crate::workspace::PullDiagnosticsParams {
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
        .pull_diagnostics(crate::workspace::PullDiagnosticsParams {
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
        .pull_diagnostics(crate::workspace::PullDiagnosticsParams {
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
            }),
            ..Default::default()
        },
        gitignore_matches: vec![],
        vcs_base_path: None,
        workspace_directory: None,
    });

    let diagnostics = workspace
        .pull_diagnostics(crate::workspace::PullDiagnosticsParams {
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
        .pull_diagnostics(crate::workspace::PullDiagnosticsParams {
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
            .filter(|d| d.category().is_some_and(|c| c.name() == "typecheck"))
            .count(),
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
        .pull_diagnostics(crate::workspace::PullDiagnosticsParams {
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
            .filter(|d| d.category().is_some_and(|c| c.name() == "typecheck"))
            .count(),
        0,
        "Expected no typecheck diagnostic"
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
        .pull_diagnostics(crate::workspace::PullDiagnosticsParams {
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
        .pull_diagnostics(crate::workspace::PullDiagnosticsParams {
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
            .pull_diagnostics(crate::workspace::PullDiagnosticsParams {
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
            Some("typecheck")
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
            .pull_diagnostics(crate::workspace::PullDiagnosticsParams {
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
