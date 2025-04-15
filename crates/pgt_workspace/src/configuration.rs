use std::{
    io::ErrorKind,
    ops::Deref,
    path::{Path, PathBuf},
};

use pgt_analyse::AnalyserRules;
use pgt_configuration::{
    ConfigurationDiagnostic, ConfigurationPathHint, ConfigurationPayload, PartialConfiguration,
    VERSION, push_to_analyser_rules,
};
use pgt_fs::{AutoSearchResult, ConfigName, FileSystem, OpenOptions};

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
    /// Return the path of the **directory** where the configuration is
    pub fn directory_path(&self) -> Option<&Path> {
        self.directory_path.as_deref()
    }

    /// Return the path of the **file** where the configuration is
    pub fn file_path(&self) -> Option<&Path> {
        self.file_path.as_deref()
    }
}

impl From<Option<ConfigurationPayload>> for LoadedConfiguration {
    fn from(value: Option<ConfigurationPayload>) -> Self {
        let Some(value) = value else {
            return LoadedConfiguration::default();
        };

        let ConfigurationPayload {
            configuration_file_path,
            deserialized: partial_configuration,
            ..
        } = value;

        LoadedConfiguration {
            configuration: partial_configuration,
            directory_path: configuration_file_path.parent().map(PathBuf::from),
            file_path: Some(configuration_file_path),
        }
    }
}

/// Load the partial configuration for this session of the CLI.
pub fn load_configuration(
    fs: &DynRef<'_, dyn FileSystem>,
    config_path: ConfigurationPathHint,
) -> Result<LoadedConfiguration, WorkspaceError> {
    let config = load_config(fs, config_path)?;
    Ok(LoadedConfiguration::from(config))
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

    // We first search for `postgrestools.jsonc`
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
    let path = PathBuf::from(ConfigName::pgt_jsonc());

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

    // we now check if postgrestools is installed inside `node_modules` and if so, we use the schema from there
    let node_schema_path = Path::new("./node_modules/@postgrestools/postgrestools/schema.json");
    let options = OpenOptions::default().read(true);
    if fs.open_with_options(node_schema_path, options).is_ok() {
        configuration.schema = node_schema_path.to_str().map(String::from);
    } else if VERSION == "0.0.0" {
        // VERSION is 0.0.0 if it has not been explicitly set (e.g local dev, as fallback)
        configuration.schema = Some("https://pgtools.dev/schemas/latest/schema.json".to_string());
    } else {
        configuration.schema = Some(format!("https://pgtools.dev/schemas/{VERSION}/schema.json"));
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
        push_to_analyser_rules(rules, pgt_analyser::METADATA.deref(), &mut analyser_rules);
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

#[cfg(test)]
mod tests {
    use super::*;

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
