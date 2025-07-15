use biome_deserialize::Merge;
use pgt_analyse::RuleCategories;
use pgt_configuration::{PartialConfiguration, database::PartialDatabaseConfiguration};
use pgt_diagnostics::Diagnostic;
use pgt_fs::PgTPath;
use pgt_text_size::TextRange;
use sqlx::PgPool;

use crate::{
    Workspace, WorkspaceError,
    workspace::{
        OpenFileParams, RegisterProjectFolderParams, UpdateSettingsParams, server::WorkspaceServer,
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

#[sqlx::test(migrator = "pgt_test_utils::MIGRATIONS")]
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

    let path = PgTPath::new("test.sql");
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
