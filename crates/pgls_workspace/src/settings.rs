use globset::Glob;
use pgls_configuration::StringSet;
use pgls_diagnostics::Category;
#[cfg(feature = "db")]
use std::str::FromStr;
use std::{
    borrow::Cow,
    num::NonZeroU64,
    path::{Path, PathBuf},
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
    time::Duration,
};
use tracing::trace;

use ignore::gitignore::{Gitignore, GitignoreBuilder};
use pgls_configuration::{
    ConfigurationDiagnostic, LinterConfiguration, PartialConfiguration, TypecheckConfiguration,
    database::PartialDatabaseConfiguration,
    diagnostics::InvalidIgnorePattern,
    files::FilesConfiguration,
    format::{
        CastStyle, ClauseBodyStyle, CommaStyle, FormatConfiguration, IndentStyle, KeywordCase,
        Layout, LogicalOperatorPlacement,
    },
    migrations::{MigrationsConfiguration, PartialMigrationsConfiguration, SectionMarkers},
    pglinter::PglinterConfiguration,
    plpgsql_check::PlPgSqlCheckConfiguration,
    splinter::SplinterConfiguration,
};
use pgls_fs::PgLSPath;
#[cfg(feature = "db")]
use sqlx::postgres::PgConnectOptions;

use crate::{
    WorkspaceError,
    matcher::Matcher,
    workspace::{ProjectKey, WorkspaceData},
};

#[derive(Debug, Default)]
/// The information tracked for each project
pub struct ProjectData {
    /// The root path of the project. This path should be **absolute**.
    path: PgLSPath,
    /// The settings of the project, usually inferred from the configuration file at or above its
    /// root.
    settings: Settings,
    /// The settings of the configuration files below the root, by their folder, deepest first.
    nested: Vec<(PathBuf, Settings)>,
}

impl ProjectData {
    /// The settings that apply to `path`: those of the deepest nested configuration file that
    /// contains it, or else the root settings. A relative path is relative to the root.
    fn settings_for(&self, path: &Path) -> &Settings {
        let path = if path.is_relative() {
            Cow::Owned(self.path.join(path))
        } else {
            Cow::Borrowed(path)
        };
        self.nested
            .iter()
            .find(|(directory, _)| path.starts_with(directory))
            .map_or(&self.settings, |(_, settings)| settings)
    }
}

#[derive(Debug, Default)]
/// Type that manages different projects inside the workspace.
pub struct WorkspaceSettings {
    /// The data of the projects
    data: WorkspaceData<ProjectData>,
}

impl WorkspaceSettings {
    /// Insert a new project using its folder. Use [WorkspaceSettings::get_settings_mut] to
    /// retrieve a mutable reference to its [Settings] and manipulate them.
    pub fn insert_project(&mut self, workspace_path: impl Into<PathBuf>) -> ProjectKey {
        let path = PgLSPath::new(workspace_path.into());
        trace!("Insert workspace folder: {:?}", path);
        self.data.insert(ProjectData {
            path,
            settings: Settings::default(),
            nested: Vec::new(),
        })
    }

    /// Remove a project, including its nested settings, using its folder.
    pub fn remove_project(&mut self, workspace_path: &Path) {
        if let Some(key) = self.get_project_key(workspace_path) {
            self.data.remove(key);
        }
    }

    /// The project registered at exactly `path`.
    pub fn get_project_key(&self, path: &Path) -> Option<ProjectKey> {
        self.data
            .iter()
            .find(|(_, project)| project.path.as_path() == path)
            .map(|(key, _)| key)
    }

    /// The folder of a project.
    pub fn get_project_path(&self, key: ProjectKey) -> Option<&PgLSPath> {
        self.data.get(key).map(|project| &project.path)
    }

    /// The root settings of a project.
    pub fn get_settings(&self, key: ProjectKey) -> Option<&Settings> {
        self.data.get(key).map(|project| &project.settings)
    }

    /// The root settings of a project.
    pub fn get_settings_mut(&mut self, key: ProjectKey) -> Option<&mut Settings> {
        self.data.get_mut(key).map(|project| &mut project.settings)
    }

    /// The settings of a project that apply to `path`, so that a file uses the nearest
    /// configuration file.
    pub fn get_settings_for_path(&self, key: ProjectKey, path: &Path) -> Option<&Settings> {
        self.data.get(key).map(|project| project.settings_for(path))
    }

    /// The root settings and the nested settings of a project.
    pub fn get_all_settings(&self, key: ProjectKey) -> impl Iterator<Item = &Settings> {
        self.data.get(key).into_iter().flat_map(|project| {
            std::iter::once(&project.settings)
                .chain(project.nested.iter().map(|(_, settings)| settings))
        })
    }

    /// Replaces the settings of the configuration files below the root of a project.
    pub fn set_nested_settings(&mut self, key: ProjectKey, mut nested: Vec<(PathBuf, Settings)>) {
        nested.sort_by(|(a, _), (b, _)| {
            b.components()
                .count()
                .cmp(&a.components().count())
                .then_with(|| a.cmp(b))
        });
        if let Some(project) = self.data.get_mut(key) {
            project.nested = nested;
        }
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

    /// The root settings of a project.
    pub(crate) fn settings(&self, key: ProjectKey) -> Option<&Settings> {
        self.inner.get_settings(key)
    }

    /// The settings of a project that apply to the file at `path`.
    pub(crate) fn settings_for(&self, key: ProjectKey, path: &Path) -> Option<&Settings> {
        self.inner.get_settings_for_path(key, path)
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

    /// Splinter (database linter) settings for the workspace
    pub splinter: SplinterSettings,

    /// Formatter settings applied to all files in the workspace
    pub formatter: FormatterSettings,

    /// Pglinter (database linter via pglinter extension) settings for the workspace
    pub pglinter: PglinterSettings,

    /// Type checking settings for the workspace
    pub typecheck: TypecheckSettings,

    /// plpgsql_check settings for the workspace
    pub plpgsql_check: PlPgSqlCheckSettings,

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

        // splinter part
        if let Some(splinter) = configuration.splinter {
            self.splinter = to_splinter_settings(SplinterConfiguration::from(splinter));
        }

        // formatter part
        if let Some(format) = configuration.format {
            self.formatter = to_formatter_settings(
                working_directory.clone(),
                FormatConfiguration::from(format),
            )?;
        }

        // pglinter part
        if let Some(pglinter) = configuration.pglinter {
            self.pglinter = to_pglinter_settings(PglinterConfiguration::from(pglinter));
        }

        // typecheck part
        if let Some(typecheck) = configuration.typecheck {
            self.typecheck = to_typecheck_settings(TypecheckConfiguration::from(typecheck));
        }

        // plpgsql_check part
        if let Some(plpgsql_check) = configuration.plpgsql_check {
            self.plpgsql_check =
                to_plpgsql_check_settings(PlPgSqlCheckConfiguration::from(plpgsql_check));
        }

        // Migrations settings
        if let Some(migrations) = configuration.migrations {
            self.migrations = Some(to_migration_settings(
                working_directory.clone(),
                MigrationsConfiguration::from(migrations),
            ));
        }

        Ok(())
    }

    /// Retrieves the settings of the linter
    pub fn linter(&self) -> &LinterSettings {
        &self.linter
    }

    /// Returns the rule settings of the linter.
    pub fn as_linter_rules(&self) -> Option<pgls_configuration::LinterRuleSettings<'_>> {
        self.linter
            .rules
            .as_ref()
            .map(|rules| pgls_configuration::LinterRuleSettings::new(rules, &self.linter.groups))
    }

    /// Returns splinter rules.
    pub fn as_splinter_rules(&self) -> Option<Cow<'_, pgls_configuration::splinter::Rules>> {
        self.splinter.rules.as_ref().map(Cow::Borrowed)
    }

    /// Returns pglinter rules.
    pub fn as_pglinter_rules(&self) -> Option<Cow<'_, pgls_configuration::pglinter::Rules>> {
        self.pglinter.rules.as_ref().map(Cow::Borrowed)
    }

    /// It retrieves the severity based on the `code` of the rule and the current configuration.
    ///
    /// The code of the rule has the pattern `lint/{rule_name}`.
    ///
    /// It returns [None] if the `code` doesn't match any rule.
    pub fn get_severity_from_rule_code(
        &self,
        code: &Category,
    ) -> Option<pgls_diagnostics::Severity> {
        self.as_linter_rules()
            .and_then(|rules| rules.get_severity_from_code(code))
    }
}

fn to_linter_settings(
    working_directory: Option<PathBuf>,
    conf: LinterConfiguration,
) -> Result<LinterSettings, WorkspaceError> {
    Ok(LinterSettings {
        enabled: conf.enabled,
        rules: Some(conf.rules),
        groups: conf.groups,
        ignored_files: to_matcher(working_directory.clone(), Some(&conf.ignore))?,
        included_files: to_matcher(working_directory.clone(), Some(&conf.include))?,
    })
}

fn to_splinter_settings(conf: SplinterConfiguration) -> SplinterSettings {
    SplinterSettings {
        enabled: conf.enabled,
        ignore: conf.ignore,
        rules: Some(conf.rules),
    }
}

fn to_formatter_settings(
    working_directory: Option<PathBuf>,
    conf: FormatConfiguration,
) -> Result<FormatterSettings, WorkspaceError> {
    Ok(FormatterSettings {
        enabled: conf.enabled,
        line_width: conf.line_width,
        indent_size: conf.indent_size,
        indent_style: conf.indent_style,
        keyword_case: conf.keyword_case,
        constant_case: conf.constant_case,
        type_case: conf.type_case,
        comma_style: conf.comma_style,
        logical_operator_placement: conf.logical_operator_placement,
        layout: conf.layout,
        values_row_layout: conf.values_row_layout,
        cast_style: conf.cast_style,
        clause_body_style: conf.clause_body_style,
        isolate_semicolon: conf.isolate_semicolon,
        function_argument_groups: conf.function_argument_groups,
        skip_fn_bodies: conf.skip_fn_bodies,
        ignored_files: to_matcher(working_directory.clone(), Some(&conf.ignore))?,
        included_files: to_matcher(working_directory.clone(), Some(&conf.include))?,
    })
}

fn to_pglinter_settings(conf: PglinterConfiguration) -> PglinterSettings {
    PglinterSettings {
        enabled: conf.enabled,
        rules: Some(conf.rules),
    }
}

fn to_typecheck_settings(conf: TypecheckConfiguration) -> TypecheckSettings {
    TypecheckSettings {
        search_path: conf.search_path.into_iter().collect(),
        enabled: conf.enabled,
    }
}

fn to_plpgsql_check_settings(conf: PlPgSqlCheckConfiguration) -> PlPgSqlCheckSettings {
    PlPgSqlCheckSettings {
        enabled: conf.enabled,
        fatal_errors: conf.fatal_errors,
        other_warnings: conf.other_warnings,
        extra_warnings: conf.extra_warnings,
        performance_warnings: conf.performance_warnings,
        security_warnings: conf.security_warnings,
        compatibility_warnings: conf.compatibility_warnings,
        without_warnings: conf.without_warnings,
        all_warnings: conf.all_warnings,
        use_incomment_options: conf.use_incomment_options,
        incomment_options_usage_warning: conf.incomment_options_usage_warning,
        constant_tracing: conf.constant_tracing,
    }
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
    pub rules: Option<pgls_configuration::linter::Rules>,

    /// The level of all rules of a group
    pub groups: pgls_configuration::linter::Groups,

    /// List of ignored paths/files to match
    pub ignored_files: Matcher,

    /// List of included paths/files to match
    pub included_files: Matcher,
}

impl Default for LinterSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            rules: Some(pgls_configuration::linter::Rules::default()),
            groups: Default::default(),
            ignored_files: Matcher::empty(),
            included_files: Matcher::empty(),
        }
    }
}

/// Splinter (database linter) settings for the entire workspace
#[derive(Debug)]
pub struct SplinterSettings {
    /// Enabled by default
    pub enabled: bool,

    /// Global ignore patterns for database objects (applies to all rules)
    pub ignore: StringSet,

    /// List of rules
    pub rules: Option<pgls_configuration::splinter::Rules>,
}

impl Default for SplinterSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            ignore: StringSet::default(),
            rules: Some(pgls_configuration::splinter::Rules::default()),
        }
    }
}

impl SplinterSettings {
    /// Convert settings back to SplinterConfiguration for use with splinter
    pub fn to_configuration(&self) -> pgls_configuration::splinter::SplinterConfiguration {
        pgls_configuration::splinter::SplinterConfiguration {
            enabled: self.enabled,
            ignore: self.ignore.clone(),
            rules: self.rules.clone().unwrap_or_default(),
        }
    }
}

/// Formatter settings for the entire workspace
#[derive(Debug)]
pub struct FormatterSettings {
    /// Disabled by default (beta)
    pub enabled: bool,

    /// Maximum line width before breaking. Default: 100.
    pub line_width: u16,

    /// Number of spaces (or tab width) for indentation. Default: 2.
    pub indent_size: u8,

    /// Indentation style: spaces or tabs. Default: spaces.
    pub indent_style: IndentStyle,

    /// Keyword casing: upper or lower. Default: lower.
    pub keyword_case: KeywordCase,

    /// Constant casing (NULL, TRUE, FALSE): upper or lower. Default: lower.
    pub constant_case: KeywordCase,

    /// Data type casing (text, varchar, int): upper or lower. Default: lower.
    pub type_case: KeywordCase,

    /// Where a comma sits when a list breaks: trailing or leading. Default: trailing.
    pub comma_style: CommaStyle,

    /// Where a boolean operator sits when a condition breaks: trailing or leading. Default: trailing.
    pub logical_operator_placement: LogicalOperatorPlacement,

    /// How a statement is laid out across lines: fit or expanded. Default: fit.
    pub layout: Layout,

    /// How VALUES rows are laid out: fit or expanded. Default: fit.
    pub values_row_layout: Layout,

    /// How an explicit cast is spelled: cast or operator. Default: cast.
    pub cast_style: CastStyle,

    /// Where the body of a clause starts: break or compact. Default: break.
    pub clause_body_style: ClauseBodyStyle,

    /// If true, the terminating semicolon goes on its own line when the statement spans several
    /// lines. Default: false.
    pub isolate_semicolon: bool,

    /// Function names mapped to the number of adjacent arguments in one logical group.
    pub function_argument_groups: pgls_configuration::FunctionArgumentGroups,

    /// If true, skip formatting of SQL function bodies (keep them verbatim). Default: false.
    pub skip_fn_bodies: bool,

    /// List of ignored paths/files to match
    pub ignored_files: Matcher,

    /// List of included paths/files to match
    pub included_files: Matcher,
}

impl Default for FormatterSettings {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default during beta
            line_width: 100,
            indent_size: 2,
            indent_style: IndentStyle::Spaces,
            keyword_case: KeywordCase::default(),
            constant_case: KeywordCase::default(),
            type_case: KeywordCase::default(),
            comma_style: CommaStyle::default(),
            logical_operator_placement: LogicalOperatorPlacement::default(),
            layout: Layout::default(),
            values_row_layout: Layout::default(),
            cast_style: CastStyle::default(),
            clause_body_style: ClauseBodyStyle::default(),
            isolate_semicolon: false,
            function_argument_groups: Default::default(),
            skip_fn_bodies: false,
            ignored_files: Matcher::empty(),
            included_files: Matcher::empty(),
        }
    }
}

/// Pglinter (database linter via pglinter extension) settings for the entire workspace
#[derive(Debug)]
pub struct PglinterSettings {
    /// Disabled by default (pglinter extension might not be installed)
    pub enabled: bool,

    /// List of rules
    pub rules: Option<pgls_configuration::pglinter::Rules>,
}

impl Default for PglinterSettings {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default since pglinter extension might not be installed
            rules: Some(pgls_configuration::pglinter::Rules::default()),
        }
    }
}

/// plpgsql_check settings for the entire workspace
#[derive(Debug, Clone)]
pub struct PlPgSqlCheckSettings {
    pub enabled: bool,
    pub fatal_errors: bool,
    pub other_warnings: bool,
    pub extra_warnings: bool,
    pub performance_warnings: bool,
    pub security_warnings: bool,
    pub compatibility_warnings: bool,
    pub without_warnings: bool,
    pub all_warnings: bool,
    pub use_incomment_options: bool,
    pub incomment_options_usage_warning: bool,
    pub constant_tracing: bool,
}

impl Default for PlPgSqlCheckSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            fatal_errors: true,
            other_warnings: true,
            extra_warnings: true,
            performance_warnings: false,
            security_warnings: false,
            compatibility_warnings: false,
            without_warnings: false,
            all_warnings: false,
            use_incomment_options: true,
            incomment_options_usage_warning: false,
            constant_tracing: true,
        }
    }
}

/// Type checking settings for the entire workspace
#[derive(Debug)]
pub struct TypecheckSettings {
    /// Enabled by default
    pub enabled: bool,
    /// Default search path schemas for type checking
    pub search_path: Vec<String>,
}

impl Default for TypecheckSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            search_path: vec!["public".to_string()],
        }
    }
}

/// The password used when neither the configuration, `PGPASSWORD`, nor the password file has one.
pub const DEFAULT_DATABASE_PASSWORD: &str = "postgres";

/// Database settings for the entire workspace
pub struct DatabaseSettings {
    pub enable_connection: bool,
    pub connection_string: Option<String>,
    pub host: String,
    pub port: u16,
    pub username: String,
    /// The password from the configuration or `PGPASSWORD`. Without one, the connection looks
    /// it up in the password file and falls back to [`DEFAULT_DATABASE_PASSWORD`].
    pub password: Option<String>,
    pub database: String,
    pub conn_timeout_secs: Duration,
    pub allow_statement_executions: bool,
}

impl std::fmt::Debug for DatabaseSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DatabaseSettings")
            .field("enable_connection", &self.enable_connection)
            .field(
                "connection_string",
                &self.connection_string.as_ref().map(|_| "[redacted]"),
            )
            .field("host", &self.host)
            .field("port", &self.port)
            .field("username", &self.username)
            .field("password", &"[redacted]")
            .field("database", &self.database)
            .field("conn_timeout_secs", &self.conn_timeout_secs)
            .field(
                "allow_statement_executions",
                &self.allow_statement_executions,
            )
            .finish()
    }
}

impl Default for DatabaseSettings {
    fn default() -> Self {
        Self {
            enable_connection: false,
            connection_string: None,
            host: "127.0.0.1".to_string(),
            port: 5432,
            username: "postgres".to_string(),
            password: None,
            database: "postgres".to_string(),
            conn_timeout_secs: Duration::from_secs(10),
            allow_statement_executions: true,
        }
    }
}

impl From<PartialDatabaseConfiguration> for DatabaseSettings {
    fn from(value: PartialDatabaseConfiguration) -> Self {
        let d = DatabaseSettings::default();

        let connection_string = value.connection_string.and_then(|uri| {
            let trimmed = uri.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        });

        // "host" OR "connectionString" is the minimum required setting for database features
        // to be enabled.
        let disable_connection = value.disable_connection.is_some_and(|disabled| disabled);
        let enable_connection =
            (connection_string.is_some() || value.host.as_ref().is_some()) && !disable_connection;

        #[cfg_attr(not(feature = "db"), allow(unused_mut))]
        let mut database = value.database.unwrap_or(d.database);
        #[cfg_attr(not(feature = "db"), allow(unused_mut))]
        let mut host = value.host.unwrap_or(d.host);
        #[cfg_attr(not(feature = "db"), allow(unused_mut))]
        let mut port = value.port.unwrap_or(d.port);
        #[cfg_attr(not(feature = "db"), allow(unused_mut))]
        let mut username = value.username.unwrap_or(d.username);

        // Parse connection string to extract host/port/username/database (DB mode only)
        #[cfg(feature = "db")]
        if let Some(uri) = connection_string.as_ref() {
            let opts = PgConnectOptions::from_str(uri)
                .unwrap_or_else(|err| panic!("Invalid db.connectionString provided: {err}"));

            // we only extract the values we need for statement execution checks and connection key
            host = opts.get_host().to_string();
            port = opts.get_port();
            username = opts.get_username().to_string();
            if let Some(db) = opts.get_database() {
                database = db.to_string();
            }
        }

        let allow_statement_executions = value
            .allow_statement_executions_against
            .map(|stringset| {
                stringset.iter().any(|pattern| {
                    let glob = Glob::new(pattern)
                        .unwrap_or_else(|_| panic!("Invalid pattern: {pattern}"))
                        .compile_matcher();

                    glob.is_match(format!("{host}/{database}"))
                })
            })
            .unwrap_or(false);

        Self {
            enable_connection,
            connection_string,

            port,
            username,
            // An empty password counts as none, as in libpq.
            password: value.password.filter(|password| !password.is_empty()),
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
#[derive(Debug)]
pub struct MigrationSettings {
    pub path: Option<PathBuf>,
    pub after: Option<u64>,
    pub section_markers: SectionMarkers,
    pub lint_down: bool,
}

impl Default for MigrationSettings {
    fn default() -> Self {
        Self {
            path: None,
            after: None,
            section_markers: SectionMarkers::None,
            lint_down: true,
        }
    }
}

impl From<PartialMigrationsConfiguration> for MigrationSettings {
    fn from(value: PartialMigrationsConfiguration) -> Self {
        Self {
            path: value.migrations_dir.map(PathBuf::from),
            after: value.after,
            section_markers: value.section_markers.unwrap_or_default(),
            lint_down: value.lint_down.unwrap_or(true),
        }
    }
}

fn to_migration_settings(
    working_directory: Option<PathBuf>,
    conf: MigrationsConfiguration,
) -> MigrationSettings {
    MigrationSettings {
        // The migrations directory is relative to the working directory.
        path: working_directory
            .map(|working_directory| working_directory.join(conf.migrations_dir)),
        after: Some(conf.after),
        section_markers: conf.section_markers,
        lint_down: conf.lint_down,
    }
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
    use std::path::{Path, PathBuf};

    use pgls_configuration::StringSet;
    use pgls_configuration::database::PartialDatabaseConfiguration;

    use super::{DatabaseSettings, Settings, WorkspaceSettings};
    use crate::workspace::ProjectKey;

    fn with_linter(enabled: bool) -> Settings {
        let mut settings = Settings::default();
        settings.linter.enabled = enabled;
        settings
    }

    /// The nested settings of the folder that contains the path, or else the root settings.
    fn linter_enabled(settings: &WorkspaceSettings, key: ProjectKey, path: &str) -> bool {
        settings
            .get_settings_for_path(key, Path::new(path))
            .unwrap()
            .linter
            .enabled
    }

    /// A file uses the deepest nested settings that contain it.
    #[test]
    fn a_path_uses_the_nearest_settings() {
        let mut settings = WorkspaceSettings::default();
        let key = settings.insert_project("/repo");
        settings.get_settings_mut(key).unwrap().linter.enabled = false;
        settings.set_nested_settings(
            key,
            vec![
                (PathBuf::from("/repo/db1"), with_linter(true)),
                (PathBuf::from("/repo/db1/legacy"), with_linter(false)),
                (PathBuf::from("/repo/db10"), with_linter(true)),
            ],
        );

        assert!(!linter_enabled(&settings, key, "/repo/a.sql"));
        assert!(linter_enabled(&settings, key, "/repo/db1/a.sql"));
        assert!(!linter_enabled(&settings, key, "/repo/db1/legacy/x/a.sql"));
        assert!(linter_enabled(&settings, key, "/repo/db10/a.sql"));
        assert!(!linter_enabled(&settings, key, "/repo/db2/a.sql"));
        // A path outside of the project uses its root settings.
        assert!(!linter_enabled(&settings, key, "/elsewhere/a.sql"));
    }

    #[test]
    fn a_relative_path_is_relative_to_the_root_of_the_project() {
        let mut settings = WorkspaceSettings::default();
        let key = settings.insert_project("/repo");
        settings.set_nested_settings(key, vec![(PathBuf::from("/repo/db"), with_linter(false))]);

        assert!(!linter_enabled(&settings, key, "db/a.sql"));
        assert!(linter_enabled(&settings, key, "a.sql"));
    }

    /// Projects are independent: each one has its own nested settings.
    #[test]
    fn projects_have_their_own_nested_settings() {
        let mut settings = WorkspaceSettings::default();
        let one = settings.insert_project("/one");
        let two = settings.insert_project("/two");
        settings.set_nested_settings(one, vec![(PathBuf::from("/one/db"), with_linter(false))]);

        assert!(!linter_enabled(&settings, one, "/one/db/a.sql"));
        assert!(linter_enabled(&settings, two, "/one/db/a.sql"));
        assert_eq!(settings.get_all_settings(one).count(), 2);
        assert_eq!(settings.get_all_settings(two).count(), 1);
    }

    #[test]
    fn removing_a_project_removes_its_nested_settings() {
        let mut settings = WorkspaceSettings::default();
        let key = settings.insert_project("/repo");
        settings.set_nested_settings(key, vec![(PathBuf::from("/repo/db"), with_linter(false))]);

        settings.remove_project(Path::new("/repo"));

        assert_eq!(settings.get_project_key(Path::new("/repo")), None);
        assert_eq!(settings.get_all_settings(key).count(), 0);
        assert!(
            settings
                .get_settings_for_path(key, Path::new("/repo/db/a.sql"))
                .is_none()
        );
    }

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

    #[cfg(feature = "db")]
    #[test]
    fn connection_string_enables_statement_executions_matching_host() {
        let partial_config = PartialDatabaseConfiguration {
            connection_string: Some(
                "postgres://postgres:postgres@localhost:5432/test-db".to_string(),
            ),
            allow_statement_executions_against: Some(StringSet::from_iter(vec![String::from(
                "localhost/*",
            )])),
            ..Default::default()
        };

        let config = DatabaseSettings::from(partial_config);

        assert!(config.enable_connection);
        assert!(config.allow_statement_executions)
    }

    #[cfg(feature = "db")]
    #[test]
    fn connection_string_respects_statement_execution_filters() {
        let partial_config = PartialDatabaseConfiguration {
            connection_string: Some(
                "postgres://postgres:postgres@prod-host:5432/prod-db".to_string(),
            ),
            allow_statement_executions_against: Some(StringSet::from_iter(vec![String::from(
                "localhost/*",
            )])),
            ..Default::default()
        };

        let config = DatabaseSettings::from(partial_config);

        assert!(!config.allow_statement_executions)
    }
}
