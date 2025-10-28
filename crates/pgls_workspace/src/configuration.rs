use std::{
    ffi::OsStr,
    io::ErrorKind,
    ops::Deref,
    path::{Path, PathBuf},
};

use biome_deserialize::Merge;
use pgls_analyse::AnalyserRules;
use pgls_configuration::{
    ConfigurationDiagnostic, ConfigurationPathHint, ConfigurationPayload, PartialConfiguration,
    VERSION, diagnostics::CantLoadExtendFile, push_to_analyser_rules,
};
use pgls_console::markup;
use pgls_env::PGLS_WEBSITE;
use pgls_fs::{AutoSearchResult, ConfigName, FileSystem, OpenOptions};

use crate::{DynRef, WorkspaceError, settings::Settings};

/// Information regarding the configuration that was found.
///
/// This contains the expanded configuration including default values where no
/// configuration was present.
#[derive(Default, Debug)]
pub struct LoadedConfiguration {
    /// If present, the path of the directory where it was found
    pub directory_path: Option<PathBuf>,
    /// If present, the path of the file where it was found
    pub file_path: Option<PathBuf>,
    /// The Deserialized configuration
    pub configuration: PartialConfiguration,
}

impl LoadedConfiguration {
    fn try_from_payload(
        value: Option<ConfigurationPayload>,
        fs: &DynRef<'_, dyn FileSystem>,
    ) -> Result<Self, WorkspaceError> {
        let Some(value) = value else {
            return Ok(LoadedConfiguration::default());
        };

        let ConfigurationPayload {
            external_resolution_base_path,
            configuration_file_path,
            deserialized: mut partial_configuration,
        } = value;

        partial_configuration.apply_extends(
            fs,
            &configuration_file_path,
            &external_resolution_base_path,
        )?;

        Ok(Self {
            configuration: partial_configuration,
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
pub fn load_configuration(
    fs: &DynRef<'_, dyn FileSystem>,
    config_path: ConfigurationPathHint,
) -> Result<LoadedConfiguration, WorkspaceError> {
    let config = load_config(fs, config_path)?;
    LoadedConfiguration::try_from_payload(config, fs)
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
        ConfigurationPathHint::FromUser(_) | ConfigurationPathHint::None => file_system
            .working_directory()
            .map_or(PathBuf::new(), |working_directory| working_directory),
    };

    // If the configuration path hint is from user and is a file path,
    // we'll load it directly
    if let ConfigurationPathHint::FromUser(ref config_file_path) = base_path {
        if file_system.path_is_file(config_file_path) {
            let content = strip_jsonc_comments(&file_system.read_file_from_path(config_file_path)?);

            let deserialized = serde_json::from_str::<PartialConfiguration>(&content)
                .map_err(ConfigurationDiagnostic::new_deserialization_error)?;

            return Ok(Some(ConfigurationPayload {
                deserialized,
                configuration_file_path: PathBuf::from(config_file_path),
                external_resolution_base_path,
            }));
        }
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

        let deserialized =
            serde_json::from_str::<PartialConfiguration>(&strip_jsonc_comments(&content))
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

/// Creates a new configuration on file system
///
/// ## Errors
///
/// It fails if:
/// - the configuration file already exists
/// - the program doesn't have the write rights
pub fn create_config(
    fs: &mut DynRef<dyn FileSystem>,
    configuration: &mut PartialConfiguration,
) -> Result<(), WorkspaceError> {
    let path = PathBuf::from(ConfigName::pgls_jsonc());

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
    if fs
        .open_with_options(pgls_node_schema_path, OpenOptions::default().read(true))
        .is_ok()
    {
        configuration.schema = pgls_node_schema_path.to_str().map(String::from);
    } else if fs
        .open_with_options(
            postgrestools_node_schema_path,
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
pub fn to_analyser_rules(settings: &Settings) -> AnalyserRules {
    let mut analyser_rules = AnalyserRules::default();
    if let Some(rules) = settings.linter.rules.as_ref() {
        push_to_analyser_rules(rules, pgls_analyser::METADATA.deref(), &mut analyser_rules);
    }
    analyser_rules
}

/// Takes a string of jsonc content and returns a comment free version
/// which should parse fine as regular json.
/// Nested block comments are supported.
pub fn strip_jsonc_comments(jsonc_input: &str) -> String {
    let mut json_output = String::new();

    let mut block_comment_depth: u8 = 0;
    let mut is_in_string: bool = false; // Comments cannot be in strings

    for line in jsonc_input.split('\n') {
        let mut last_char: Option<char> = None;
        for cur_char in line.chars() {
            // Check whether we're in a string
            if block_comment_depth == 0 && last_char != Some('\\') && cur_char == '"' {
                is_in_string = !is_in_string;
            }

            // Check for line comment start
            if !is_in_string && last_char == Some('/') && cur_char == '/' {
                last_char = None;
                json_output.push_str("  ");
                break; // Stop outputting or parsing this line
            }
            // Check for block comment start
            if !is_in_string && last_char == Some('/') && cur_char == '*' {
                block_comment_depth += 1;
                last_char = None;
                json_output.push_str("  ");
            // Check for block comment end
            } else if !is_in_string && last_char == Some('*') && cur_char == '/' {
                block_comment_depth = block_comment_depth.saturating_sub(1);
                last_char = None;
                json_output.push_str("  ");
            // Output last char if not in any block comment
            } else {
                if block_comment_depth == 0 {
                    if let Some(last_char) = last_char {
                        json_output.push(last_char);
                    }
                } else {
                    json_output.push(' ');
                }
                last_char = Some(cur_char);
            }
        }

        // Add last char and newline if not in any block comment
        if let Some(last_char) = last_char {
            if block_comment_depth == 0 {
                json_output.push(last_char);
            } else {
                json_output.push(' ');
            }
        }

        // Remove trailing whitespace from line
        while json_output.ends_with(' ') {
            json_output.pop();
        }
        json_output.push('\n');
    }

    json_output
}

pub trait PartialConfigurationExt {
    fn apply_extends(
        &mut self,
        fs: &DynRef<'_, dyn FileSystem>,
        file_path: &Path,
        external_resolution_base_path: &Path,
    ) -> Result<(), WorkspaceError>;

    fn deserialize_extends(
        &mut self,
        fs: &DynRef<'_, dyn FileSystem>,
        relative_resolution_base_path: &Path,
        external_resolution_base_path: &Path,
    ) -> Result<Vec<PartialConfiguration>, WorkspaceError>;

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
    ) -> Result<(), WorkspaceError> {
        let configurations = self.deserialize_extends(
            fs,
            file_path.parent().expect("file path should have a parent"),
            external_resolution_base_path,
        )?;

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

        Ok(())
    }

    /// It attempts to deserialize all the configuration files that were specified in the `extends` property
    fn deserialize_extends(
        &mut self,
        fs: &DynRef<'_, dyn FileSystem>,
        relative_resolution_base_path: &Path,
        external_resolution_base_path: &Path,
    ) -> Result<Vec<PartialConfiguration>, WorkspaceError> {
        let Some(extends) = &self.extends else {
            return Ok(Vec::new());
        };

        let mut deserialized_configurations = vec![];
        for extend_entry in extends.iter() {
            let extend_entry_as_path = Path::new(extend_entry);

            let extend_configuration_file_path = if extend_entry_as_path.starts_with(".")
                || matches!(
                    extend_entry_as_path
                        .extension()
                        .map(OsStr::as_encoded_bytes),
                    Some(b"jsonc")
                ) {
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
                        "Postgres Tools tried to load the configuration file \""<Emphasis>{
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

            let deserialized = serde_json::from_str::<PartialConfiguration>(&content)
                .map_err(ConfigurationDiagnostic::new_deserialization_error)?;
            deserialized_configurations.push(deserialized)
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
            if let Some(client_kind) = &vcs.client_kind {
                if !vcs.ignore_file_disabled() {
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
        }
        Ok((None, vec![]))
    }
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

    #[test]
    fn test_strip_jsonc_comments_line_comments() {
        let input = r#"{
  "name": "test", // This is a line comment
  "value": 42 // Another comment
}"#;

        let expected = r#"{
  "name": "test",
  "value": 42
}
"#;

        assert_eq!(strip_jsonc_comments(input), expected);
    }

    #[test]
    fn test_strip_jsonc_comments_block_comments() {
        let input = r#"{
  /* This is a block comment */
  "name": "test",
  "value": /* inline comment */ 42
}"#;

        let expected = r#"{

  "name": "test",
  "value":                       42
}
"#;

        assert_eq!(strip_jsonc_comments(input), expected);
    }

    #[test]
    fn test_strip_jsonc_comments_nested_block_comments() {
        let input = r#"{
  /* Outer comment /* Nested comment */ still outer */
  "name": "test"
}"#;

        let expected = r#"{

  "name": "test"
}
"#;

        assert_eq!(strip_jsonc_comments(input), expected);
    }

    #[test]
    fn test_strip_jsonc_comments_in_strings() {
        let input = r#"{
  "comment_like": "This is not a // comment",
  "another": "This is not a /* block comment */ either"
}"#;

        let expected = r#"{
  "comment_like": "This is not a // comment",
  "another": "This is not a /* block comment */ either"
}
"#;

        assert_eq!(strip_jsonc_comments(input), expected);
    }

    #[test]
    fn test_strip_jsonc_comments_escaped_quotes() {
        let input = r#"{
  "escaped\": \"quote": "value", // Comment after escaped quotes
  "normal": "value" // Normal comment
}"#;

        let expected = r#"{
  "escaped\": \"quote": "value",
  "normal": "value"
}
"#;

        assert_eq!(strip_jsonc_comments(input), expected);
    }

    #[test]
    fn test_strip_jsonc_comments_multiline_block() {
        let input = r#"{
  /* This is a
     multiline block
     comment */
  "name": "test"
}"#;

        let expected = r#"{



  "name": "test"
}
"#;

        assert_eq!(strip_jsonc_comments(input), expected);
    }
}
