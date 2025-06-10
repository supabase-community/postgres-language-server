use biome_deserialize::StringSet;
use globset::Glob;
use pgt_diagnostics::Category;
use std::{
    borrow::Cow,
    num::NonZeroU64,
    path::{Path, PathBuf},
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
    time::Duration,
};
use tracing::trace;

use ignore::gitignore::{Gitignore, GitignoreBuilder};
use pgt_configuration::{
    ConfigurationDiagnostic, LinterConfiguration, PartialConfiguration,
    database::PartialDatabaseConfiguration,
    diagnostics::InvalidIgnorePattern,
    files::FilesConfiguration,
    migrations::{MigrationsConfiguration, PartialMigrationsConfiguration},
};
use pgt_fs::PgTPath;

use crate::{
    WorkspaceError,
    matcher::Matcher,
    workspace::{ProjectKey, WorkspaceData},
};

#[derive(Debug, Default)]
/// The information tracked for each project
pub struct ProjectData {
    /// The root path of the project. This path should be **absolute**.
    path: PgTPath,
    /// The settings of the project, usually inferred from the configuration file e.g. `biome.json`.
    settings: Settings,
}

#[derive(Debug, Default)]
/// Type that manages different projects inside the workspace.
pub struct WorkspaceSettings {
    /// The data of the projects
    data: WorkspaceData<ProjectData>,
    /// The ID of the current project.
    current_project: ProjectKey,
}

impl WorkspaceSettings {
    pub fn get_current_project_key(&self) -> ProjectKey {
        self.current_project
    }

    pub fn get_current_project_path(&self) -> Option<&PgTPath> {
        trace!("Current key {:?}", self.current_project);
        self.data
            .get(self.current_project)
            .as_ref()
            .map(|d| &d.path)
    }

    pub fn get_current_project_data_mut(&mut self) -> &mut ProjectData {
        self.data
            .get_mut(self.current_project)
            .expect("Current project not configured")
    }

    /// Retrieves the settings of the current workspace folder
    pub fn get_current_settings(&self) -> Option<&Settings> {
        trace!("Current key {:?}", self.current_project);
        let data = self.data.get(self.current_project);
        if let Some(data) = data {
            Some(&data.settings)
        } else {
            None
        }
    }

    /// Retrieves a mutable reference of the settings of the current project
    pub fn get_current_settings_mut(&mut self) -> &mut Settings {
        &mut self
            .data
            .get_mut(self.current_project)
            .expect("You must have at least one workspace.")
            .settings
    }

    /// Register the current project using its unique key
    pub fn register_current_project(&mut self, key: ProjectKey) {
        self.current_project = key;
    }

    /// Insert a new project using its folder. Use [WorkspaceSettings::get_current_settings_mut] to retrieve
    /// a mutable reference to its [Settings] and manipulate them.
    pub fn insert_project(&mut self, workspace_path: impl Into<PathBuf>) -> ProjectKey {
        let path = PgTPath::new(workspace_path.into());
        trace!("Insert workspace folder: {:?}", path);
        self.data.insert(ProjectData {
            path,
            settings: Settings::default(),
        })
    }

    /// Remove a project using its folder.
    pub fn remove_project(&mut self, workspace_path: &Path) {
        let keys_to_remove = {
            let mut data = vec![];
            let iter = self.data.iter();

            for (key, path_to_settings) in iter {
                if path_to_settings.path.as_path() == workspace_path {
                    data.push(key)
                }
            }

            data
        };

        for key in keys_to_remove {
            self.data.remove(key)
        }
    }

    /// Checks if the current path belongs to a registered project.
    ///
    /// If there's a match, and the match **isn't** the current project, it returns the new key.
    pub fn path_belongs_to_current_workspace(&self, path: &PgTPath) -> Option<ProjectKey> {
        if self.data.is_empty() {
            return None;
        }
        trace!("Current key: {:?}", self.current_project);
        let iter = self.data.iter();
        for (key, path_to_settings) in iter {
            trace!(
                "Workspace path {:?}, file path {:?}",
                path_to_settings.path, path
            );
            trace!("Iter key: {:?}", key);
            if key == self.current_project {
                continue;
            }
            if path.strip_prefix(path_to_settings.path.as_path()).is_ok() {
                trace!("Update workspace to {:?}", key);
                return Some(key);
            }
        }
        None
    }

    /// Checks if the current path belongs to a registered project.
    ///
    /// If there's a match, and the match **isn't** the current project, the function will mark the match as the current project.
    pub fn set_current_project(&mut self, new_key: ProjectKey) {
        self.current_project = new_key;
    }
}

#[derive(Debug)]
pub struct WorkspaceSettingsHandle<'a> {
    inner: RwLockReadGuard<'a, WorkspaceSettings>,
}

impl<'a> WorkspaceSettingsHandle<'a> {
    pub(crate) fn new(settings: &'a RwLock<WorkspaceSettings>) -> Self {
        Self {
            inner: settings.read().unwrap(),
        }
    }

    pub(crate) fn settings(&self) -> Option<&Settings> {
        self.inner.get_current_settings()
    }

    pub(crate) fn path(&self) -> Option<&PgTPath> {
        self.inner.get_current_project_path()
    }
}

impl AsRef<WorkspaceSettings> for WorkspaceSettingsHandle<'_> {
    fn as_ref(&self) -> &WorkspaceSettings {
        &self.inner
    }
}

pub struct WorkspaceSettingsHandleMut<'a> {
    inner: RwLockWriteGuard<'a, WorkspaceSettings>,
}

impl<'a> WorkspaceSettingsHandleMut<'a> {
    pub(crate) fn new(settings: &'a RwLock<WorkspaceSettings>) -> Self {
        Self {
            inner: settings.write().unwrap(),
        }
    }
}

impl AsMut<WorkspaceSettings> for WorkspaceSettingsHandleMut<'_> {
    fn as_mut(&mut self) -> &mut WorkspaceSettings {
        &mut self.inner
    }
}

/// Global settings for the entire workspace
#[derive(Debug, Default)]
pub struct Settings {
    /// Filesystem settings for the workspace
    pub files: FilesSettings,

    /// Database settings for the workspace
    pub db: DatabaseSettings,

    /// Linter settings applied to all files in the workspace
    pub linter: LinterSettings,

    /// Migrations settings
    pub migrations: Option<MigrationSettings>,
}

impl Settings {
    /// The [PartialConfiguration] is merged into the workspace
    #[tracing::instrument(level = "trace", skip(self), err)]
    pub fn merge_with_configuration(
        &mut self,
        configuration: PartialConfiguration,
        working_directory: Option<PathBuf>,
        vcs_path: Option<PathBuf>,
        gitignore_matches: &[String],
    ) -> Result<(), WorkspaceError> {
        // Filesystem settings
        if let Some(files) = to_file_settings(
            working_directory.clone(),
            configuration.files.map(FilesConfiguration::from),
            vcs_path,
            gitignore_matches,
        )? {
            self.files = files;
        }

        // db settings
        if let Some(db) = configuration.db {
            self.db = db.into()
        }

        // linter part
        if let Some(linter) = configuration.linter {
            self.linter =
                to_linter_settings(working_directory.clone(), LinterConfiguration::from(linter))?;
        }

        // Migrations settings
        if let Some(migrations) = configuration.migrations {
            self.migrations = to_migration_settings(
                working_directory.clone(),
                MigrationsConfiguration::from(migrations),
            );
        }

        Ok(())
    }

    /// Retrieves the settings of the linter
    pub fn linter(&self) -> &LinterSettings {
        &self.linter
    }

    /// Returns linter rules.
    pub fn as_linter_rules(&self) -> Option<Cow<pgt_configuration::analyser::linter::Rules>> {
        self.linter.rules.as_ref().map(Cow::Borrowed)
    }

    /// It retrieves the severity based on the `code` of the rule and the current configuration.
    ///
    /// The code of the has the following pattern: `{group}/{rule_name}`.
    ///
    /// It returns [None] if the `code` doesn't match any rule.
    pub fn get_severity_from_rule_code(
        &self,
        code: &Category,
    ) -> Option<pgt_diagnostics::Severity> {
        let rules = self.linter.rules.as_ref();
        if let Some(rules) = rules {
            rules.get_severity_from_code(code)
        } else {
            None
        }
    }
}

fn to_linter_settings(
    working_directory: Option<PathBuf>,
    conf: LinterConfiguration,
) -> Result<LinterSettings, WorkspaceError> {
    Ok(LinterSettings {
        enabled: conf.enabled,
        rules: Some(conf.rules),
        ignored_files: to_matcher(working_directory.clone(), Some(&conf.ignore))?,
        included_files: to_matcher(working_directory.clone(), Some(&conf.include))?,
    })
}

fn to_file_settings(
    working_directory: Option<PathBuf>,
    config: Option<FilesConfiguration>,
    vcs_config_path: Option<PathBuf>,
    gitignore_matches: &[String],
) -> Result<Option<FilesSettings>, WorkspaceError> {
    let config = match config {
        Some(config) => Some(config),
        _ => {
            if vcs_config_path.is_some() {
                Some(FilesConfiguration::default())
            } else {
                None
            }
        }
    };
    let git_ignore = if let Some(vcs_config_path) = vcs_config_path {
        Some(to_git_ignore(vcs_config_path, gitignore_matches)?)
    } else {
        None
    };
    Ok(match config {
        Some(config) => Some(FilesSettings {
            max_size: config.max_size,
            git_ignore,
            ignored_files: to_matcher(working_directory.clone(), Some(&config.ignore))?,
            included_files: to_matcher(working_directory, Some(&config.include))?,
        }),
        _ => None,
    })
}

fn to_git_ignore(path: PathBuf, matches: &[String]) -> Result<Gitignore, WorkspaceError> {
    let mut gitignore_builder = GitignoreBuilder::new(path.clone());

    for the_match in matches {
        gitignore_builder
            .add_line(Some(path.clone()), the_match)
            .map_err(|err| {
                ConfigurationDiagnostic::InvalidIgnorePattern(InvalidIgnorePattern {
                    message: err.to_string(),
                    file_path: path.to_str().map(|s| s.to_string()),
                })
            })?;
    }
    let gitignore = gitignore_builder.build().map_err(|err| {
        ConfigurationDiagnostic::InvalidIgnorePattern(InvalidIgnorePattern {
            message: err.to_string(),
            file_path: path.to_str().map(|s| s.to_string()),
        })
    })?;
    Ok(gitignore)
}

/// Creates a [Matcher] from a [StringSet]
///
/// ## Errors
///
/// It can raise an error if the patterns aren't valid
pub fn to_matcher(
    working_directory: Option<PathBuf>,
    string_set: Option<&StringSet>,
) -> Result<Matcher, WorkspaceError> {
    let mut matcher = Matcher::empty();
    if let Some(working_directory) = working_directory {
        matcher.set_root(working_directory)
    }
    if let Some(string_set) = string_set {
        for pattern in string_set.iter() {
            matcher.add_pattern(pattern).map_err(|err| {
                ConfigurationDiagnostic::new_invalid_ignore_pattern(
                    pattern.to_string(),
                    err.msg.to_string(),
                )
            })?;
        }
    }
    Ok(matcher)
}

/// Linter settings for the entire workspace
#[derive(Debug)]
pub struct LinterSettings {
    /// Enabled by default
    pub enabled: bool,

    /// List of rules
    pub rules: Option<pgt_configuration::analyser::linter::Rules>,

    /// List of ignored paths/files to match
    pub ignored_files: Matcher,

    /// List of included paths/files to match
    pub included_files: Matcher,
}

impl Default for LinterSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            rules: Some(pgt_configuration::analyser::linter::Rules::default()),
            ignored_files: Matcher::empty(),
            included_files: Matcher::empty(),
        }
    }
}

/// Database settings for the entire workspace
#[derive(Debug)]
pub struct DatabaseSettings {
    pub enable_connection: bool,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub conn_timeout_secs: Duration,
    pub allow_statement_executions: bool,
}

impl Default for DatabaseSettings {
    fn default() -> Self {
        Self {
            enable_connection: false,
            host: "127.0.0.1".to_string(),
            port: 5432,
            username: "postgres".to_string(),
            password: "postgres".to_string(),
            database: "postgres".to_string(),
            conn_timeout_secs: Duration::from_secs(10),
            allow_statement_executions: true,
        }
    }
}

impl From<PartialDatabaseConfiguration> for DatabaseSettings {
    fn from(value: PartialDatabaseConfiguration) -> Self {
        let d = DatabaseSettings::default();

        // "host" is the minimum required setting for database features
        // to be enabled.
        let enable_connection = value
            .host
            .as_ref()
            .is_some_and(|_| value.disable_connection.is_none_or(|disabled| !disabled));

        let database = value.database.unwrap_or(d.database);
        let host = value.host.unwrap_or(d.host);

        let allow_statement_executions = value
            .allow_statement_executions_against
            .map(|stringset| {
                stringset.iter().any(|pattern| {
                    let glob = Glob::new(pattern)
                        .unwrap_or_else(|_| panic!("Invalid pattern: {}", pattern))
                        .compile_matcher();

                    glob.is_match(format!("{}/{}", host, database))
                })
            })
            .unwrap_or(false);

        Self {
            enable_connection,

            port: value.port.unwrap_or(d.port),
            username: value.username.unwrap_or(d.username),
            password: value.password.unwrap_or(d.password),
            database,
            host,

            conn_timeout_secs: value
                .conn_timeout_secs
                .map(|s| Duration::from_secs(s.into()))
                .unwrap_or(d.conn_timeout_secs),

            allow_statement_executions,
        }
    }
}

/// Filesystem settings for the entire workspace
#[derive(Debug)]
pub struct FilesSettings {
    /// File size limit in bytes
    pub max_size: NonZeroU64,

    /// List of paths/files to matcher
    pub ignored_files: Matcher,

    /// List of paths/files to matcher
    pub included_files: Matcher,

    /// gitignore file patterns
    pub git_ignore: Option<Gitignore>,
}

/// Migration settings
#[derive(Debug, Default)]
pub struct MigrationSettings {
    pub path: Option<PathBuf>,
    pub after: Option<u64>,
}

impl From<PartialMigrationsConfiguration> for MigrationSettings {
    fn from(value: PartialMigrationsConfiguration) -> Self {
        Self {
            path: value.migrations_dir.map(PathBuf::from),
            after: value.after,
        }
    }
}

fn to_migration_settings(
    working_directory: Option<PathBuf>,
    conf: MigrationsConfiguration,
) -> Option<MigrationSettings> {
    working_directory.map(|working_directory| MigrationSettings {
        path: Some(working_directory.join(conf.migrations_dir)),
        after: Some(conf.after),
    })
}

/// Limit the size of files to 1.0 MiB by default
pub(crate) const DEFAULT_FILE_SIZE_LIMIT: NonZeroU64 =
    // SAFETY: This constant is initialized with a non-zero value
    NonZeroU64::new(1024 * 1024).unwrap();

impl Default for FilesSettings {
    fn default() -> Self {
        Self {
            max_size: DEFAULT_FILE_SIZE_LIMIT,
            ignored_files: Matcher::empty(),
            included_files: Matcher::empty(),
            git_ignore: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use biome_deserialize::StringSet;
    use pgt_configuration::database::PartialDatabaseConfiguration;

    use super::DatabaseSettings;

    #[test]
    fn should_identify_allowed_statement_executions() {
        let partial_config = PartialDatabaseConfiguration {
            allow_statement_executions_against: Some(StringSet::from_iter(vec![String::from(
                "localhost/*",
            )])),
            host: Some("localhost".into()),
            database: Some("test-db".into()),
            ..Default::default()
        };

        let config = DatabaseSettings::from(partial_config);

        assert!(config.allow_statement_executions)
    }

    #[test]
    fn should_identify_not_allowed_statement_executions() {
        let partial_config = PartialDatabaseConfiguration {
            allow_statement_executions_against: Some(StringSet::from_iter(vec![String::from(
                "localhost/*",
            )])),
            host: Some("production".into()),
            database: Some("test-db".into()),
            ..Default::default()
        };

        let config = DatabaseSettings::from(partial_config);

        assert!(!config.allow_statement_executions)
    }
}
