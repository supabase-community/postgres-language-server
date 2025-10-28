use crate::{CliDiagnostic, VcsIntegration};
use pgls_configuration::{ConfigurationPathHint, PartialConfiguration};
use pgls_fs::FileSystem;
use pgls_workspace::PartialConfigurationExt;
use pgls_workspace::configuration::{LoadedConfiguration, load_configuration};
use pgls_workspace::workspace::{RegisterProjectFolderParams, UpdateSettingsParams};
use pgls_workspace::{DynRef, Workspace};

/// Load configuration from disk and emit warnings for deprecated filenames.
pub fn load_config(
    fs: &DynRef<'_, dyn FileSystem>,
    config_hint: ConfigurationPathHint,
) -> Result<LoadedConfiguration, CliDiagnostic> {
    load_configuration(fs, config_hint).map_err(CliDiagnostic::from)
}

/// Configure the workspace and VCS integration according to the provided configuration.
pub fn setup_workspace(
    workspace: &dyn Workspace,
    fs: &DynRef<'_, dyn FileSystem>,
    configuration: PartialConfiguration,
    vcs: VcsIntegration,
) -> Result<(), CliDiagnostic> {
    let (vcs_base_path, gitignore_matches) = match vcs {
        VcsIntegration::Enabled => configuration
            .retrieve_gitignore_matches(fs, fs.working_directory().as_deref())
            .map_err(CliDiagnostic::from)?,
        VcsIntegration::Disabled => (None, vec![]),
    };

    workspace
        .register_project_folder(RegisterProjectFolderParams {
            path: fs.working_directory(),
            set_as_current_workspace: true,
        })
        .map_err(CliDiagnostic::from)?;

    workspace
        .update_settings(UpdateSettingsParams {
            workspace_directory: fs.working_directory(),
            configuration,
            vcs_base_path,
            gitignore_matches,
        })
        .map_err(CliDiagnostic::from)?;

    Ok(())
}
