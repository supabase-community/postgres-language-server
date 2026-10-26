use std::path::PathBuf;

use pgls_configuration::{ConfigurationPathHint, PartialConfiguration};
use pgls_fs::{ConfigName, FileSystem};
use tracing::info;

use crate::{
    DynRef, Workspace, WorkspaceError,
    configuration::{ConfigurationLayers, LoadedConfiguration, load_configuration},
    settings::{FilesSettings, connection_string_error},
    workspace::{ProjectKey, RegisterProjectFolderParams, UpdateSettingsParams},
};

use super::PartialConfigurationExt;

/// What [configure_project] configures a project from.
pub struct ConfigureProjectParams<'a> {
    /// Where to look for the configuration file. The folder of a workspace or LSP hint is the
    /// folder of the project; otherwise it is the working directory.
    pub hint: ConfigurationPathHint,
    /// The layers laid over the configuration file.
    pub layers: &'a ConfigurationLayers,
    /// Whether, and from where, the VCS ignore file applies.
    pub vcs: VcsIntegration,
}

/// Whether the VCS ignore file of a project applies, and where it is looked up from, if the
/// configuration enables the VCS integration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VcsIntegration {
    /// From the folder of the configuration file.
    Enabled,
    /// From the working directory of the process.
    FromWorkingDirectory,
    /// Never, whatever the configuration says.
    Disabled,
}

/// A project that [configure_project] registered.
#[derive(Debug)]
pub struct ConfiguredProject {
    pub project_key: ProjectKey,
    /// The folder the project was registered with. Empty for a project without a folder.
    pub folder: PathBuf,
    /// The files the settings come from: the configuration file and the files it extends. Set
    /// even if the configuration could not be applied.
    pub configuration_files: Vec<PathBuf>,
    /// What the user should know about the configuration.
    pub notices: Vec<ConfigurationNotice>,
    /// The configuration that was applied, or why none was. The project keeps its previous
    /// settings then, or the defaults if it is new.
    pub result: Result<AppliedConfiguration, WorkspaceError>,
}

/// The root configuration of a project, as [configure_project] applied it.
#[derive(Debug)]
pub struct AppliedConfiguration {
    /// The configuration file with the layers laid over it.
    pub configuration: PartialConfiguration,
    /// Its files settings, which decide the folders that are searched for nested configuration
    /// files.
    pub files: FilesSettings,
}

/// Something about a configuration that the user should know, which callers report the way
/// their interface does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigurationNotice {
    /// The configuration file has a deprecated name.
    DeprecatedFileName(PathBuf),
    /// The user configuration applies to the folder, which has no configuration file.
    UserConfiguration {
        file_path: PathBuf,
        directory: PathBuf,
    },
    /// The configuration uses deprecated settings, with a description of each.
    DeprecatedSettings(Vec<String>),
    /// The connection string is invalid, so the database features are disabled.
    InvalidConnectionString(String),
}

/// Registers a project and applies its configuration file, with `layers` laid over it, to its
/// root settings. The configuration files below its folder are applied separately by
/// [super::apply_nested_configurations].
///
/// Fails only if the project can't be registered. A configuration that can't be loaded or
/// applied is reported in [ConfiguredProject::result].
pub fn configure_project(
    workspace: &dyn Workspace,
    fs: &DynRef<'_, dyn FileSystem>,
    params: ConfigureProjectParams,
) -> Result<ConfiguredProject, WorkspaceError> {
    let ConfigureProjectParams { hint, layers, vcs } = params;
    let folder = match &hint {
        ConfigurationPathHint::FromWorkspace(path) | ConfigurationPathHint::FromLsp(path) => {
            Some(path.clone())
        }
        ConfigurationPathHint::FromUser(_) | ConfigurationPathHint::None => fs.working_directory(),
    };
    let project_key = workspace.register_project_folder(RegisterProjectFolderParams {
        path: folder.clone(),
    })?;

    let mut configuration_files = Vec::new();
    let mut notices = Vec::new();
    let result = load_configuration(fs, hint).and_then(|loaded| {
        info!(
            "Configuration loaded successfully from disk at {}",
            loaded
                .file_path
                .as_ref()
                .map_or("<unknown>".to_string(), |path| path.display().to_string())
        );
        configuration_files.extend(
            loaded
                .file_path
                .iter()
                .chain(&loaded.extended_files)
                .cloned(),
        );
        notices.extend(file_notices(&loaded));
        apply_root_configuration(
            workspace,
            fs,
            project_key,
            loaded,
            layers,
            vcs,
            &mut notices,
        )
    });
    Ok(ConfiguredProject {
        project_key,
        folder: folder.unwrap_or_default(),
        configuration_files,
        notices,
        result,
    })
}

/// What the user should know about a configuration file.
fn file_notices(loaded: &LoadedConfiguration) -> Vec<ConfigurationNotice> {
    let mut notices = Vec::new();
    if let Some(file_path) = &loaded.file_path
        && let Some(file_name) = file_path.file_name().and_then(|name| name.to_str())
        && ConfigName::is_deprecated(file_name)
    {
        notices.push(ConfigurationNotice::DeprecatedFileName(file_path.clone()));
    }
    if loaded.is_user_configuration
        && let (Some(file_path), Some(directory)) = (&loaded.file_path, &loaded.directory_path)
    {
        notices.push(ConfigurationNotice::UserConfiguration {
            file_path: file_path.clone(),
            directory: directory.clone(),
        });
    }
    let deprecations = loaded.configuration.deprecations();
    if !deprecations.is_empty() {
        notices.push(ConfigurationNotice::DeprecatedSettings(deprecations));
    }
    notices
}

/// Lays `layers` over a loaded configuration file and applies it to the root settings of a
/// project.
fn apply_root_configuration(
    workspace: &dyn Workspace,
    fs: &DynRef<'_, dyn FileSystem>,
    project_key: ProjectKey,
    loaded: LoadedConfiguration,
    layers: &ConfigurationLayers,
    vcs: VcsIntegration,
    notices: &mut Vec<ConfigurationNotice>,
) -> Result<AppliedConfiguration, WorkspaceError> {
    let configuration = layers.apply(loaded.configuration);
    let (vcs_base_path, gitignore_matches) = match vcs {
        VcsIntegration::Enabled => {
            configuration.retrieve_gitignore_matches(fs, loaded.directory_path.as_deref())?
        }
        VcsIntegration::FromWorkingDirectory => {
            configuration.retrieve_gitignore_matches(fs, fs.working_directory().as_deref())?
        }
        VcsIntegration::Disabled => (None, Vec::new()),
    };

    // An invalid connection string only disables the database features, so the user would not
    // notice it otherwise.
    if let Some(error) = configuration.db.as_ref().and_then(connection_string_error) {
        notices.push(ConfigurationNotice::InvalidConnectionString(error));
    }

    let workspace_directory = fs.working_directory();
    workspace.update_settings(UpdateSettingsParams {
        project_key,
        workspace_directory: workspace_directory.clone(),
        configuration: configuration.clone(),
        vcs_base_path: vcs_base_path.clone(),
        gitignore_matches: gitignore_matches.clone(),
    })?;
    let files = FilesSettings::from_configuration(
        &configuration,
        workspace_directory,
        vcs_base_path,
        &gitignore_matches,
    )?;
    Ok(AppliedConfiguration {
        configuration,
        files,
    })
}
