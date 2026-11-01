mod layers;
mod project;

pub use layers::{ConfigurationLayers, OverrideMode, SectionMask};
pub use project::{
    AppliedConfiguration, ConfigurationNotice, ConfigureProjectParams, ConfiguredProject,
    VcsIntegration, configure_project,
};

use std::{
    ffi::OsStr,
    io::ErrorKind,
    ops::Deref,
    path::{Path, PathBuf},
    sync::Arc,
};

use pgls_analyser::LinterRules;
use pgls_configuration::Merge;
use pgls_configuration::{
    ConfigurationDiagnostic, ConfigurationPathHint, ConfigurationPayload, PartialConfiguration,
    VERSION, database::PartialDatabaseConfiguration, diagnostics::CantLoadExtendFile,
};
use pgls_console::markup;
use pgls_env::PGLS_WEBSITE;
use pgls_fs::{
    AutoSearchResult, ConfigName, FileSystem, FindFiles, OpenOptions, SKIPPED_DIRECTORIES,
    SkipDirectory,
};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    DynRef, Workspace, WorkspaceError,
    settings::{FilesSettings, Settings},
    workspace::{
        NestedSettingsParams, ProjectKey, UpdateNestedSettingsParams, UpdateNestedSettingsResult,
    },
};

/// Information regarding the configuration that was found.
///
/// This contains the expanded configuration including default values where no
/// configuration was present.
#[derive(Default, Debug)]
pub struct LoadedConfiguration {
    /// If present, the path of the directory the configuration applies to: where it was found,
    /// or the project directory for the user configuration
    pub directory_path: Option<PathBuf>,
    /// If present, the path of the file where it was found
    pub file_path: Option<PathBuf>,
    /// The Deserialized configuration
    pub configuration: PartialConfiguration,
    /// The `db` section of the configuration file itself, without what it extends
    pub file_database: Option<PartialDatabaseConfiguration>,
    /// The files it extends
    pub extended_files: Vec<PathBuf>,
    /// Whether it is the user configuration, which applies because the project has none
    pub is_user_configuration: bool,
}

impl LoadedConfiguration {
    fn try_from_payload(
        value: ConfigurationPayload,
        fs: &DynRef<'_, dyn FileSystem>,
    ) -> Result<Self, WorkspaceError> {
        let ConfigurationPayload {
            external_resolution_base_path,
            configuration_file_path,
            deserialized: mut partial_configuration,
        } = value;

        let file_database = partial_configuration.db.clone();
        let extended_files = partial_configuration.apply_extends(
            fs,
            &configuration_file_path,
            &external_resolution_base_path,
        )?;

        Ok(Self {
            configuration: partial_configuration,
            file_database,
            extended_files,
            is_user_configuration: false,
            directory_path: configuration_file_path.parent().map(PathBuf::from),
            file_path: Some(configuration_file_path),
        })
    }

    /// Return the path of the **directory** where the configuration is
    pub fn directory_path(&self) -> Option<&Path> {
        self.directory_path.as_deref()
    }

    /// Return the path of the **file** where the configuration is
    pub fn file_path(&self) -> Option<&Path> {
        self.file_path.as_deref()
    }
}

/// Load the partial configuration for this session of the CLI.
///
/// If no project configuration is found and the user didn't provide a configuration path, the
/// user configuration is loaded instead. It applies to the project as if it were placed in the
/// project directory, and is never merged with a project configuration.
pub fn load_configuration(
    fs: &DynRef<'_, dyn FileSystem>,
    config_path: ConfigurationPathHint,
) -> Result<LoadedConfiguration, WorkspaceError> {
    let project_directory = match &config_path {
        ConfigurationPathHint::FromUser(_) => None,
        ConfigurationPathHint::FromLsp(path) | ConfigurationPathHint::FromWorkspace(path) => {
            Some(path.clone())
        }
        ConfigurationPathHint::None => Some(fs.working_directory().unwrap_or_default()),
    };

    if let Some(config) = load_config(fs, config_path)? {
        return LoadedConfiguration::try_from_payload(config, fs);
    }

    let Some(project_directory) = project_directory else {
        return Ok(LoadedConfiguration::default());
    };
    let Some(config) = load_user_config(fs, &project_directory)? else {
        return Ok(LoadedConfiguration::default());
    };

    let mut loaded = LoadedConfiguration::try_from_payload(config, fs)?;
    loaded.directory_path = Some(project_directory);
    loaded.is_user_configuration = true;
    Ok(loaded)
}

/// A configuration file in a folder below the root of a project.
#[derive(Debug)]
pub struct NestedConfiguration {
    /// The folder of the configuration file. Its settings apply to the files below it.
    pub directory: PathBuf,
    /// Whether the configuration file was loaded and applied. The files below a configuration file
    /// that could not be applied use the default settings.
    pub result: Result<(), WorkspaceError>,
    /// The files it extends.
    pub extended_files: Vec<PathBuf>,
}

/// Where [apply_nested_configurations] looks for configuration files.
pub struct NestedConfigurationSearch {
    /// The folder of the project.
    pub root: PathBuf,
    /// Only the configuration files that apply to these paths are looked up: those in the folders
    /// between the root and each path, and those below the paths that are folders. `None` looks
    /// below the whole root. Relative paths are relative to the root.
    pub paths: Option<Vec<PathBuf>>,
    /// The files settings of the root configuration. The folders they ignore are not searched.
    pub root_files: Option<Arc<FilesSettings>>,
}

impl NestedConfigurationSearch {
    /// Looks for every configuration file below `root`.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            paths: None,
            root_files: None,
        }
    }
}

/// The result of [apply_nested_configurations].
#[derive(Debug, Default)]
pub struct NestedConfigurations {
    /// The configuration files that apply.
    pub configurations: Vec<NestedConfiguration>,
    /// Set if the search did not look at every folder it should have, so that some configuration
    /// files may be missing.
    pub incomplete: Option<IncompleteSearch>,
}

/// Why [apply_nested_configurations] did not look at every folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IncompleteSearch {
    /// The folder is the home directory or a filesystem root, which holds unrelated projects.
    NotAProjectFolder(PathBuf),
    /// The search stopped at the depth or entry limit of [FindFiles].
    LimitReached(PathBuf),
}

impl std::fmt::Display for IncompleteSearch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotAProjectFolder(folder) => write!(
                f,
                "{} is the home directory or a filesystem root, so the configuration files below \
                 it are not used. Open a project folder to use them.",
                folder.display()
            ),
            Self::LimitReached(folder) => write!(
                f,
                "Stopped looking for configuration files below {} after {} files or {} levels of \
                 folders, so some may not be used. Ignore the folders that hold no SQL files in \
                 `files.ignore` or `.gitignore`.",
                folder.display(),
                FindFiles::DEFAULT_MAX_ENTRIES,
                FindFiles::DEFAULT_MAX_DEPTH
            ),
        }
    }
}

/// Applies the configuration files below the root of a project to it as nested settings, so that
/// each file uses the settings of the nearest configuration file. They replace the nested settings
/// the project had before.
///
/// A nested configuration file inherits nothing from the configuration files above it unless it
/// `extends` them, and the user configuration is never merged into it: that one only applies, as
/// the root settings, to the files that have no project configuration at or above them. The user
/// configuration directories are skipped even when they are below the root, and so is everything
/// below the home directory or a filesystem root, which hold unrelated projects.
///
/// `layers` are laid over each configuration file like over the root configuration file, except
/// that the environment only fills in the connection options that a nested configuration file
/// doesn't set itself, so that each file keeps its own database (see
/// [ConfigurationLayers::apply_nested]). Folders that the settings above them ignore are skipped.
///
/// This walks the file system, so call it off the async runtime. Fails if the project is not
/// registered.
pub fn apply_nested_configurations(
    workspace: &dyn Workspace,
    fs: &DynRef<'_, dyn FileSystem>,
    project_key: ProjectKey,
    search: NestedConfigurationSearch,
    layers: &ConfigurationLayers,
) -> Result<NestedConfigurations, WorkspaceError> {
    let NestedConfigurationSearch {
        root,
        paths,
        root_files,
    } = search;
    let root = root.as_path();
    let (directories, incomplete) =
        find_nested_configurations(fs, root, paths.as_deref(), root_files);
    let user_config_directories = fs.user_config_directories();
    let mut directories: Vec<PathBuf> = directories
        .into_iter()
        .filter(|directory| directory != root && directory.starts_with(root))
        .filter(|directory| !user_config_directories.contains(directory))
        .collect();
    directories.sort();
    directories.dedup();

    let mut load_errors = Vec::new();
    let mut extended_files = FxHashMap::default();
    let nested = directories
        .into_iter()
        .map(
            |directory| match load_nested_configuration(fs, &directory, layers) {
                Ok((nested, extended)) => {
                    extended_files.insert(directory, extended);
                    nested
                }
                Err(error) => {
                    load_errors.push((directory.clone(), error));
                    // Neither the configuration above it nor a part of the broken one applies.
                    NestedSettingsParams {
                        directory,
                        configuration: PartialConfiguration::default(),
                        vcs_base_path: None,
                        gitignore_matches: Vec::new(),
                    }
                }
            },
        )
        .collect();

    let UpdateNestedSettingsResult { applied, errors } =
        workspace.update_nested_settings(UpdateNestedSettingsParams {
            project_key,
            nested,
        })?;

    let configurations = applied
        .into_iter()
        .map(|directory| {
            let load_error = load_errors
                .iter()
                .position(|(failed, _)| failed == &directory)
                .map(|index| load_errors.swap_remove(index).1);
            let apply_error = errors
                .iter()
                .find(|error| error.directory == directory)
                .map(|error| ConfigurationDiagnostic::invalid_configuration(&error.message).into());
            let result = match load_error.or(apply_error) {
                Some(error) => Err(error),
                None => Ok(()),
            };
            NestedConfiguration {
                extended_files: extended_files.remove(&directory).unwrap_or_default(),
                directory,
                result,
            }
        })
        .collect();
    Ok(NestedConfigurations {
        configurations,
        incomplete,
    })
}

/// The folders with a configuration file that [NestedConfigurationSearch] asks for.
fn find_nested_configurations(
    fs: &DynRef<'_, dyn FileSystem>,
    root: &Path,
    paths: Option<&[PathBuf]>,
    root_files: Option<Arc<FilesSettings>>,
) -> (Vec<PathBuf>, Option<IncompleteSearch>) {
    let file_names = ConfigName::file_names();
    let skip_directory = root_files.map(|root_files| {
        Arc::new(move |directory: &Path| root_files.is_ignored(directory)) as SkipDirectory
    });
    let mut incomplete = None;
    let mut find = |folder: &Path| -> Vec<PathBuf> {
        if is_not_a_project_folder(fs, folder) {
            incomplete.get_or_insert(IncompleteSearch::NotAProjectFolder(folder.to_path_buf()));
            return Vec::new();
        }
        let found = fs.find_files(
            folder,
            &FindFiles {
                skip_directory: skip_directory.clone(),
                ..FindFiles::new(&file_names)
            },
        );
        if found.truncated {
            incomplete.get_or_insert(IncompleteSearch::LimitReached(folder.to_path_buf()));
        }
        found
            .files
            .into_iter()
            .filter_map(|file_path| file_path.parent().map(PathBuf::from))
            .collect()
    };

    let Some(paths) = paths else {
        let directories = find(root);
        return (directories, incomplete);
    };

    let mut directories = Vec::new();
    // The folders between the root and a path that were looked at, with their ancestors.
    let mut looked_at = FxHashSet::default();
    for path in paths {
        let path = normalize_path(&root.join(path));
        if !path.starts_with(root) {
            continue;
        }
        if fs.path_is_dir(&path) {
            directories.extend(find(&path));
        }
        for directory in path
            .ancestors()
            .skip(1)
            .take_while(|directory| *directory != root && directory.starts_with(root))
        {
            if !looked_at.insert(directory.to_path_buf()) {
                break;
            }
            if file_names
                .iter()
                .any(|file_name| fs.path_is_file(&directory.join(file_name)))
            {
                directories.push(directory.to_path_buf());
            }
        }
    }
    (directories, incomplete)
}

/// Whether [apply_nested_configurations] looks for a configuration file in `directory` when it
/// searches the whole `root`, so that a change of the file there has to be reloaded.
/// `root_files` are the files settings of the root configuration.
pub fn is_searched_for_configuration(
    fs: &DynRef<'_, dyn FileSystem>,
    root: &Path,
    root_files: Option<&FilesSettings>,
    directory: &Path,
) -> bool {
    let Ok(relative) = directory.strip_prefix(root) else {
        return false;
    };
    if relative.as_os_str().is_empty() {
        return true;
    }
    let is_skipped = |name: &OsStr| {
        name.to_str()
            .is_none_or(|name| name.starts_with('.') || SKIPPED_DIRECTORIES.contains(&name))
    };
    if is_not_a_project_folder(fs, root)
        || relative.iter().any(is_skipped)
        || relative.iter().count() > FindFiles::DEFAULT_MAX_DEPTH
        || fs
            .user_config_directories()
            .iter()
            .any(|user| user == directory)
    {
        return false;
    }
    let Some(root_files) = root_files else {
        return true;
    };
    // Like the search: the root settings decide down to the first configuration file below it.
    let mut folder = root.to_path_buf();
    for component in relative {
        folder.push(component);
        if root_files.is_ignored(&folder) {
            return false;
        }
        if folder != directory
            && ConfigName::file_names()
                .iter()
                .any(|file_name| fs.path_is_file(&folder.join(file_name)))
        {
            return true;
        }
    }
    true
}

/// The home directory and the filesystem roots hold unrelated projects, so looking for
/// configuration files below them would apply the ones of other projects and take long.
fn is_not_a_project_folder(fs: &DynRef<'_, dyn FileSystem>, folder: &Path) -> bool {
    folder.parent().is_none() || fs.home_directory().is_some_and(|home| home == folder)
}

fn load_nested_configuration(
    fs: &DynRef<'_, dyn FileSystem>,
    directory: &Path,
    layers: &ConfigurationLayers,
) -> Result<(NestedSettingsParams, Vec<PathBuf>), WorkspaceError> {
    let loaded = load_configuration(
        fs,
        ConfigurationPathHint::FromWorkspace(directory.to_path_buf()),
    )?;
    let configuration = layers.apply_nested(loaded.configuration, loaded.file_database.as_ref());
    let (vcs_base_path, gitignore_matches) =
        configuration.retrieve_gitignore_matches(fs, Some(directory))?;
    let nested = NestedSettingsParams {
        directory: directory.to_path_buf(),
        configuration,
        vcs_base_path,
        gitignore_matches,
    };
    Ok((nested, loaded.extended_files))
}

/// - [Result]: if an error occurred while loading the configuration file.
/// - [Option]: sometimes not having a configuration file should not be an error, so we need this type.
/// - [ConfigurationPayload]: The result of the operation
type LoadConfig = Result<Option<ConfigurationPayload>, WorkspaceError>;

/// Load the configuration from the file system.
///
/// The configuration file will be read from the `file_system`. A [path hint](ConfigurationPathHint) should be provided.
fn load_config(
    file_system: &DynRef<'_, dyn FileSystem>,
    base_path: ConfigurationPathHint,
) -> LoadConfig {
    // This path is used for configuration resolution from external packages.
    let external_resolution_base_path = match base_path {
        // Path hint from LSP is always the workspace root
        // we use it as the resolution base path.
        ConfigurationPathHint::FromLsp(ref path) => path.clone(),
        ConfigurationPathHint::FromWorkspace(ref path) => path.clone(),
        // Path hint from user means the command is invoked from the CLI
        // So we use the working directory (CWD) as the resolution base path
        ConfigurationPathHint::FromUser(_) | ConfigurationPathHint::None => {
            file_system.working_directory().unwrap_or_default()
        }
    };

    // If the configuration path hint is from user and is a file path,
    // we'll load it directly
    if let ConfigurationPathHint::FromUser(ref config_file_path) = base_path
        && file_system.path_is_file(config_file_path)
    {
        let content = strip_jsonc(&file_system.read_file_from_path(config_file_path)?)?;

        let deserialized = serde_json::from_str::<PartialConfiguration>(&content)
            .map_err(ConfigurationDiagnostic::new_deserialization_error)?;

        return Ok(Some(ConfigurationPayload {
            deserialized,
            configuration_file_path: PathBuf::from(config_file_path),
            external_resolution_base_path,
        }));
    }

    // If the configuration path hint is not a file path
    // we'll auto search for the configuration file
    let should_error = base_path.is_from_user();
    let configuration_directory = match base_path {
        ConfigurationPathHint::FromLsp(path) => path,
        ConfigurationPathHint::FromUser(path) => path,
        ConfigurationPathHint::FromWorkspace(path) => path,
        ConfigurationPathHint::None => file_system.working_directory().unwrap_or_default(),
    };

    // We first search for `postgres-language-server.jsonc` files
    if let Some(auto_search_result) = file_system.auto_search(
        &configuration_directory,
        ConfigName::file_names().as_slice(),
        should_error,
    )? {
        let AutoSearchResult { content, file_path } = auto_search_result;

        let deserialized = serde_json::from_str::<PartialConfiguration>(&strip_jsonc(&content)?)
            .map_err(ConfigurationDiagnostic::new_deserialization_error)?;

        Ok(Some(ConfigurationPayload {
            deserialized,
            configuration_file_path: file_path,
            external_resolution_base_path,
        }))
    } else {
        Ok(None)
    }
}

/// Load the user configuration from the first user configuration directory that has one.
fn load_user_config(
    file_system: &DynRef<'_, dyn FileSystem>,
    project_directory: &Path,
) -> LoadConfig {
    for directory in file_system.user_config_directories() {
        let file_path = directory.join(ConfigName::pgls_jsonc());
        if !file_system.path_is_file(&file_path) {
            continue;
        }

        let content = strip_jsonc(&file_system.read_file_from_path(&file_path)?)?;
        let deserialized = serde_json::from_str::<PartialConfiguration>(&content)
            .map_err(ConfigurationDiagnostic::new_deserialization_error)?;

        return Ok(Some(ConfigurationPayload {
            deserialized,
            configuration_file_path: file_path,
            external_resolution_base_path: project_directory.to_path_buf(),
        }));
    }

    Ok(None)
}

/// Creates a new configuration file in `directory`, relative to the working directory.
///
/// ## Errors
///
/// It fails if:
/// - the configuration file already exists
/// - the program doesn't have the write rights
pub fn create_config(
    fs: &mut DynRef<dyn FileSystem>,
    directory: &Path,
    configuration: &mut PartialConfiguration,
) -> Result<(), WorkspaceError> {
    let path = directory.join(ConfigName::pgls_jsonc());

    if fs.path_exists(&path) {
        return Err(ConfigurationDiagnostic::new_already_exists().into());
    }

    let options = OpenOptions::default().write(true).create_new(true);

    let mut config_file = fs.open_with_options(&path, options).map_err(|err| {
        if err.kind() == ErrorKind::AlreadyExists {
            ConfigurationDiagnostic::new_already_exists().into()
        } else {
            WorkspaceError::cant_read_file(format!("{}", path.display()))
        }
    })?;

    // we now check if postgres-language-server or postgrestools is installed inside `node_modules` and if so, we use the schema from there
    let postgrestools_node_schema_path =
        Path::new("./node_modules/@postgrestools/postgrestools/schema.json");
    let pgls_node_schema_path =
        Path::new("./node_modules/@postgres-language-server/cli/schema.json");
    // The schema path is relative to the configuration file.
    if fs
        .open_with_options(
            &directory.join(pgls_node_schema_path),
            OpenOptions::default().read(true),
        )
        .is_ok()
    {
        configuration.schema = pgls_node_schema_path.to_str().map(String::from);
    } else if fs
        .open_with_options(
            &directory.join(postgrestools_node_schema_path),
            OpenOptions::default().read(true),
        )
        .is_ok()
    {
        configuration.schema = postgrestools_node_schema_path.to_str().map(String::from);
    } else if VERSION == "0.0.0" {
        // VERSION is 0.0.0 if it has not been explicitly set (e.g local dev, as fallback)
        configuration.schema = Some(format!("{PGLS_WEBSITE}/latest/schema.json"));
    } else {
        configuration.schema = Some(format!("{PGLS_WEBSITE}/{VERSION}/schema.json"));
    }

    let contents = serde_json::to_string_pretty(&configuration)
        .map_err(|_| ConfigurationDiagnostic::new_serialization_error())?;

    config_file
        .set_content(contents.as_bytes())
        .map_err(|_| WorkspaceError::cant_read_file(format!("{}", path.display())))?;

    Ok(())
}

/// Returns the rules applied to a specific [Path], given the [Settings]
pub fn to_analyser_rules(settings: &Settings) -> LinterRules {
    let mut analyser_rules = LinterRules::default();
    if let Some(rules) = settings.as_linter_rules() {
        rules.push_to_analyser_rules(pgls_analyser::METADATA.deref(), &mut analyser_rules);
    }
    analyser_rules
}

/// Strips JSONC comments and trailing commas from input, returning valid JSON.
fn strip_jsonc(content: &str) -> Result<String, ConfigurationDiagnostic> {
    let mut data = content.to_string();
    json_strip_comments::strip(&mut data).map_err(|e| {
        ConfigurationDiagnostic::DeserializationError(
            pgls_configuration::diagnostics::DeserializationError {
                message: format!("Failed to strip JSONC comments: {e}"),
            },
        )
    })?;
    Ok(data)
}

pub trait PartialConfigurationExt {
    /// Returns the files it extended.
    fn apply_extends(
        &mut self,
        fs: &DynRef<'_, dyn FileSystem>,
        file_path: &Path,
        external_resolution_base_path: &Path,
    ) -> Result<Vec<PathBuf>, WorkspaceError>;

    /// Returns each configuration file in `extends` with its path.
    fn deserialize_extends(
        &mut self,
        fs: &DynRef<'_, dyn FileSystem>,
        relative_resolution_base_path: &Path,
        external_resolution_base_path: &Path,
    ) -> Result<Vec<(PathBuf, PartialConfiguration)>, WorkspaceError>;

    fn retrieve_gitignore_matches(
        &self,
        file_system: &DynRef<'_, dyn FileSystem>,
        vcs_base_path: Option<&Path>,
    ) -> Result<(Option<PathBuf>, Vec<String>), WorkspaceError>;
}

impl PartialConfigurationExt for PartialConfiguration {
    /// Mutates the configuration so that any fields that have not been configured explicitly are
    /// filled in with their values from configs listed in the `extends` field.
    ///
    /// The `extends` configs are applied from left to right.
    ///
    /// If a configuration can't be resolved from the file system, the operation will fail.
    fn apply_extends(
        &mut self,
        fs: &DynRef<'_, dyn FileSystem>,
        file_path: &Path,
        external_resolution_base_path: &Path,
    ) -> Result<Vec<PathBuf>, WorkspaceError> {
        let (extended_files, configurations): (Vec<_>, Vec<_>) = self
            .deserialize_extends(
                fs,
                file_path.parent().expect("file path should have a parent"),
                external_resolution_base_path,
            )?
            .into_iter()
            .unzip();

        let extended_configuration = configurations.into_iter().reduce(
            |mut previous_configuration, current_configuration| {
                previous_configuration.merge_with(current_configuration);
                previous_configuration
            },
        );
        if let Some(mut extended_configuration) = extended_configuration {
            // We swap them to avoid having to clone `self.configuration` to merge it.
            std::mem::swap(self, &mut extended_configuration);
            self.merge_with(extended_configuration)
        }

        Ok(extended_files)
    }

    /// It attempts to deserialize all the configuration files that were specified in the `extends` property
    fn deserialize_extends(
        &mut self,
        fs: &DynRef<'_, dyn FileSystem>,
        relative_resolution_base_path: &Path,
        external_resolution_base_path: &Path,
    ) -> Result<Vec<(PathBuf, PartialConfiguration)>, WorkspaceError> {
        let Some(extends) = &self.extends else {
            return Ok(Vec::new());
        };

        let home_directory = fs.home_directory();
        let mut deserialized_configurations = vec![];
        for extend_entry in extends.iter() {
            let extend_entry_as_path = Path::new(extend_entry);

            let extend_configuration_file_path =
                if let Some(path) = expand_home(extend_entry, home_directory.as_deref()) {
                    normalize_path(&path)
                } else if extend_entry_as_path.starts_with(".")
                    || matches!(
                        extend_entry_as_path
                            .extension()
                            .map(OsStr::as_encoded_bytes),
                        Some(b"jsonc")
                    )
                {
                    // Normalize the path to handle relative segments like "../"
                    normalize_path(&relative_resolution_base_path.join(extend_entry))
                } else {
                    fs.resolve_configuration(extend_entry.as_str(), external_resolution_base_path)
                        .map_err(|error| {
                            ConfigurationDiagnostic::cant_resolve(
                                external_resolution_base_path.display().to_string(),
                                error,
                            )
                        })?
                        .into_path_buf()
                };

            let mut file = fs
                .open_with_options(
                    extend_configuration_file_path.as_path(),
                    OpenOptions::default().read(true),
                )
                .map_err(|err| {
                    CantLoadExtendFile::new(
                        extend_configuration_file_path.display().to_string(),
                        err.to_string(),
                    )
                    .with_verbose_advice(markup! {
                        "Postgres Language Server tried to load the configuration file \""<Emphasis>{
                            extend_configuration_file_path.display().to_string()
                        }</Emphasis>"\" in \"extends\" using \""<Emphasis>{
                            external_resolution_base_path.display().to_string()
                        }</Emphasis>"\" as the base path."
                    })
                })?;

            let mut content = String::new();
            file.read_to_string(&mut content).map_err(|err| {
                CantLoadExtendFile::new(extend_configuration_file_path.display().to_string(), err.to_string()).with_verbose_advice(
                    markup!{
                        "It's possible that the file was created with a different user/group. Make sure you have the rights to read the file."
                    }
                )

            })?;

            let deserialized =
                serde_json::from_str::<PartialConfiguration>(&strip_jsonc(&content)?)
                    .map_err(ConfigurationDiagnostic::new_deserialization_error)?;
            deserialized_configurations.push((extend_configuration_file_path, deserialized))
        }
        Ok(deserialized_configurations)
    }

    /// This function checks if the VCS integration is enabled, and if so, it will attempts to resolve the
    /// VCS root directory and the `.gitignore` file.
    ///
    /// ## Returns
    ///
    /// A tuple with VCS root folder and the contents of the `.gitignore` file
    fn retrieve_gitignore_matches(
        &self,
        file_system: &DynRef<'_, dyn FileSystem>,
        vcs_base_path: Option<&Path>,
    ) -> Result<(Option<PathBuf>, Vec<String>), WorkspaceError> {
        let Some(vcs) = &self.vcs else {
            return Ok((None, vec![]));
        };
        if vcs.is_enabled() {
            let vcs_base_path = match (vcs_base_path, &vcs.root) {
                (Some(vcs_base_path), Some(root)) => vcs_base_path.join(root),
                (None, Some(root)) => PathBuf::from(root),
                (Some(vcs_base_path), None) => PathBuf::from(vcs_base_path),
                (None, None) => return Err(WorkspaceError::vcs_disabled()),
            };
            if let Some(client_kind) = &vcs.client_kind
                && !vcs.ignore_file_disabled()
            {
                let result = file_system
                    .auto_search(&vcs_base_path, &[client_kind.ignore_file()], false)
                    .map_err(WorkspaceError::from)?;

                if let Some(result) = result {
                    return Ok((
                        result.file_path.parent().map(PathBuf::from),
                        result
                            .content
                            .lines()
                            .map(String::from)
                            .collect::<Vec<String>>(),
                    ));
                }
            }
        }
        Ok((None, vec![]))
    }
}

/// Expands a leading `~` in a path to the home directory.
///
/// Returns `None` if the path doesn't start with `~` or the home directory is unknown.
fn expand_home(path: &str, home_directory: Option<&Path>) -> Option<PathBuf> {
    let rest = if path == "~" {
        ""
    } else {
        path.strip_prefix("~/")
            .or_else(|| path.strip_prefix("~\\").filter(|_| cfg!(windows)))?
    };
    Some(home_directory?.join(rest))
}

/// Normalizes a path, resolving '..' and '.' segments without requiring the path to exist
fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    let mut prefix_component = None;
    let mut is_absolute = false;

    for component in path.components() {
        match component {
            std::path::Component::Prefix(_prefix) => {
                prefix_component = Some(component);
                components.clear();
            }
            std::path::Component::RootDir => {
                is_absolute = true;
                components.clear();
            }
            std::path::Component::ParentDir => {
                if !components.is_empty() {
                    components.pop();
                } else if !is_absolute && prefix_component.is_none() {
                    // Only keep parent dir if we're not absolute and have no prefix
                    components.push(component.as_os_str());
                }
            }
            std::path::Component::Normal(c) => {
                components.push(c);
            }
            std::path::Component::CurDir => {
                // Skip current directory components
            }
        }
    }

    let mut result = PathBuf::new();

    // Add prefix component (like C: on Windows)
    if let Some(prefix) = prefix_component {
        result.push(prefix.as_os_str());
    }

    // Add root directory if path is absolute
    if is_absolute {
        result.push(std::path::Component::RootDir.as_os_str());
    }

    // Add normalized components
    for component in components {
        result.push(component);
    }

    // Handle edge cases
    if result.as_os_str().is_empty() {
        if prefix_component.is_some() || is_absolute {
            // This shouldn't happen with proper input, but fallback to original path's root
            return path
                .ancestors()
                .last()
                .unwrap_or(Path::new(""))
                .to_path_buf();
        } else {
            return PathBuf::from(".");
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use pgls_fs::MemoryFileSystem;

    fn project_directory() -> PathBuf {
        PathBuf::from("/project")
    }

    fn home_directory() -> PathBuf {
        PathBuf::from("/home/alice")
    }

    fn user_config_directory() -> PathBuf {
        home_directory()
            .join(".config")
            .join("postgres-language-server")
    }

    fn file_system() -> MemoryFileSystem {
        let mut fs = MemoryFileSystem::default();
        fs.set_home_directory(home_directory());
        fs.set_user_config_directories(vec![user_config_directory()]);
        fs
    }

    fn load(fs: MemoryFileSystem, hint: ConfigurationPathHint) -> LoadedConfiguration {
        load_configuration(&DynRef::Owned(Box::new(fs)), hint).expect("configuration should load")
    }

    fn max_file_size(loaded: &LoadedConfiguration) -> Option<u64> {
        loaded
            .configuration
            .files
            .as_ref()?
            .max_size
            .map(|size| size.get())
    }

    fn files_config(max_size: u32) -> String {
        format!(r#"{{ "files": {{ "maxSize": {max_size} }} }}"#)
    }

    #[test]
    fn extends_expands_the_home_directory() {
        let mut fs = file_system();
        fs.insert(
            project_directory().join(ConfigName::pgls_jsonc()),
            r#"{ "extends": ["~/.config/shared.jsonc"] }"#,
        );
        fs.insert(
            home_directory().join(".config").join("shared.jsonc"),
            files_config(1),
        );

        let loaded = load(
            fs,
            ConfigurationPathHint::FromWorkspace(project_directory()),
        );

        assert_eq!(max_file_size(&loaded), Some(1));
    }

    #[test]
    fn extends_expands_the_home_directory_for_any_extension() {
        let mut fs = file_system();
        fs.insert(
            project_directory().join(ConfigName::pgls_jsonc()),
            r#"{ "extends": ["~/shared.json"] }"#,
        );
        fs.insert(home_directory().join("shared.json"), files_config(1));

        let loaded = load(
            fs,
            ConfigurationPathHint::FromWorkspace(project_directory()),
        );

        assert_eq!(max_file_size(&loaded), Some(1));
    }

    #[test]
    fn expand_home_only_expands_a_leading_tilde() {
        let home = home_directory();
        assert_eq!(expand_home("~", Some(&home)), Some(home.join("")));
        assert_eq!(
            expand_home("~/a/b.jsonc", Some(&home)),
            Some(home.join("a/b.jsonc"))
        );
        assert_eq!(expand_home("~a/b.jsonc", Some(&home)), None);
        assert_eq!(expand_home("./~/b.jsonc", Some(&home)), None);
        assert_eq!(expand_home("~/b.jsonc", None), None);
        assert_eq!(
            expand_home("~\\b.jsonc", Some(&home)),
            cfg!(windows).then(|| home.join("b.jsonc"))
        );
    }

    #[test]
    fn user_config_is_loaded_without_a_project_config() {
        let mut fs = file_system();
        let user_config = user_config_directory().join(ConfigName::pgls_jsonc());
        fs.insert(user_config.clone(), files_config(1));

        let loaded = load(
            fs,
            ConfigurationPathHint::FromWorkspace(project_directory()),
        );

        assert_eq!(max_file_size(&loaded), Some(1));
        assert_eq!(loaded.file_path(), Some(user_config.as_path()));
        assert_eq!(loaded.directory_path(), Some(project_directory().as_path()));
    }

    #[test]
    fn user_config_resolves_extends_relative_to_itself() {
        let mut fs = file_system();
        fs.insert(
            user_config_directory().join(ConfigName::pgls_jsonc()),
            r#"{ "extends": ["./base.jsonc"] }"#,
        );
        fs.insert(user_config_directory().join("base.jsonc"), files_config(1));

        let loaded = load(fs, ConfigurationPathHint::FromLsp(project_directory()));

        assert_eq!(max_file_size(&loaded), Some(1));
    }

    #[test]
    fn user_config_is_looked_up_in_order() {
        let mut fs = file_system();
        let second_directory = home_directory().join("second");
        fs.set_user_config_directories(vec![
            home_directory().join("missing"),
            user_config_directory(),
            second_directory.clone(),
        ]);
        fs.insert(
            user_config_directory().join(ConfigName::pgls_jsonc()),
            files_config(1),
        );
        fs.insert(
            second_directory.join(ConfigName::pgls_jsonc()),
            files_config(2),
        );

        let loaded = load(fs, ConfigurationPathHint::None);

        assert_eq!(max_file_size(&loaded), Some(1));
    }

    #[test]
    fn project_config_is_not_merged_with_the_user_config() {
        let mut fs = file_system();
        fs.insert(
            user_config_directory().join(ConfigName::pgls_jsonc()),
            files_config(1),
        );
        let project_config = project_directory().join(ConfigName::pgls_jsonc());
        fs.insert(project_config.clone(), "{}");

        let loaded = load(
            fs,
            ConfigurationPathHint::FromWorkspace(project_directory()),
        );

        assert_eq!(max_file_size(&loaded), None);
        assert_eq!(loaded.file_path(), Some(project_config.as_path()));
    }

    #[test]
    fn user_config_is_not_used_for_an_explicit_config_path() {
        let mut fs = file_system();
        fs.insert(
            user_config_directory().join(ConfigName::pgls_jsonc()),
            files_config(1),
        );

        let result = load_configuration(
            &DynRef::Owned(Box::new(fs)),
            ConfigurationPathHint::FromUser(project_directory()),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_normalize_path_windows_drive() {
        if cfg!(windows) {
            let path = Path::new(r"z:\workspace\test_one\..\postgres-language-server.jsonc");
            let normalized = normalize_path(path);
            assert_eq!(
                normalized,
                PathBuf::from(r"z:\workspace\postgres-language-server.jsonc")
            );
        }
    }

    #[test]
    fn test_normalize_path_relative() {
        let path = Path::new("workspace/test_one/../postgres-language-server.jsonc");
        let normalized = normalize_path(path);
        assert_eq!(
            normalized,
            PathBuf::from("workspace/postgres-language-server.jsonc")
        );
    }

    #[test]
    fn test_normalize_path_multiple_parent_dirs() {
        if cfg!(windows) {
            let path = Path::new(r"c:\a\b\c\..\..\d");
            let normalized = normalize_path(path);
            assert_eq!(normalized, PathBuf::from(r"c:\a\d"));
        }
    }
}
