use crate::{CliDiagnostic, VcsIntegration};
use pgls_configuration::{ConfigurationPathHint, PartialConfiguration};
use pgls_fs::FileSystem;
use pgls_workspace::PartialConfigurationExt;
use pgls_workspace::configuration::{LoadedConfiguration, load_configuration};
use pgls_workspace::settings::FilesSettings;
use pgls_workspace::workspace::{ProjectKey, RegisterProjectFolderParams, UpdateSettingsParams};
use pgls_workspace::{DynRef, Workspace};

/// Load configuration from disk and emit warnings for deprecated filenames.
pub fn load_config(
    fs: &DynRef<'_, dyn FileSystem>,
    config_hint: ConfigurationPathHint,
) -> Result<LoadedConfiguration, CliDiagnostic> {
    load_configuration(fs, config_hint).map_err(CliDiagnostic::from)
}

/// Registers the project of the working directory and configures it and the VCS integration
/// according to the provided configuration. Returns the key of the project and the files settings
/// of the configuration.
pub fn setup_workspace(
    workspace: &dyn Workspace,
    fs: &DynRef<'_, dyn FileSystem>,
    configuration: PartialConfiguration,
    vcs: VcsIntegration,
) -> Result<(ProjectKey, FilesSettings), CliDiagnostic> {
    let (vcs_base_path, gitignore_matches) = match vcs {
        VcsIntegration::Enabled => configuration
            .retrieve_gitignore_matches(fs, fs.working_directory().as_deref())
            .map_err(CliDiagnostic::from)?,
        VcsIntegration::Disabled => (None, vec![]),
    };

    let project_key = workspace
        .register_project_folder(RegisterProjectFolderParams {
            path: fs.working_directory(),
        })
        .map_err(CliDiagnostic::from)?;

    let files = FilesSettings::from_configuration(
        &configuration,
        fs.working_directory(),
        vcs_base_path.clone(),
        &gitignore_matches,
    )?;
    workspace
        .update_settings(UpdateSettingsParams {
            project_key,
            workspace_directory: fs.working_directory(),
            configuration,
            vcs_base_path,
            gitignore_matches,
        })
        .map_err(CliDiagnostic::from)?;

    Ok((project_key, files))
}
