//! Parse command: emit the libpg_query parse tree for consumption by other
//! tools.
//!
//! The output is the `ParseResult` protobuf message defined by libpg_query,
//! the same contract the official bindings of other languages expose, so a
//! consumer can decode it with any generated `pg_query` client. The parse
//! reflects how the rest of the CLI reads SQL: psql named parameters
//! (`:'name'`, `:name`) become parser-compatible placeholders of identical
//! byte length, keeping every location of the tree aligned with the input.
//! A template, see `templating.syntax`, is parsed with placeholders for its
//! directives, like `check` reads it.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use pgls_configuration::{
    ConfigurationPathHint, Merge,
    templating::{PartialTemplatingConfiguration, Template, TemplatingConfiguration},
};
use pgls_console::{ConsoleExt, EnvConsole, markup};
use pgls_fs::{FileSystem, normalize_path};
use pgls_workspace::{DynRef, configuration::load_configuration, settings::TemplatingSettings};
use prost::Message;

use crate::CliDiagnostic;

/// What decides how the SQL of `parse` is read.
pub(crate) struct ParseArgs {
    /// The path of the SQL file, also for the code from standard input.
    pub(crate) path: Option<PathBuf>,
    /// `--config-path`.
    pub(crate) config_path: Option<PathBuf>,
    /// `--template-syntax`, which replaces the `templating.syntax` of the configuration.
    pub(crate) templating: Option<PartialTemplatingConfiguration>,
}

impl ParseArgs {
    /// The template syntax of the file: that of the configuration file of its folder, or of
    /// `--config-path`, replaced by `--template-syntax`.
    pub(crate) fn template(
        &self,
        fs: &DynRef<'_, dyn FileSystem>,
    ) -> Result<Template, CliDiagnostic> {
        let working_directory = fs.working_directory().unwrap_or_default();
        let path = self
            .path
            .as_ref()
            .map(|path| normalize_path(&working_directory.join(path)));
        let hint = match (&self.config_path, &path) {
            (Some(config_path), _) => ConfigurationPathHint::FromUser(config_path.clone()),
            (None, Some(path)) => ConfigurationPathHint::FromWorkspace(
                path.parent()
                    .map_or(working_directory.clone(), Path::to_path_buf),
            ),
            (None, None) => ConfigurationPathHint::None,
        };
        let loaded = load_configuration(fs, hint)?;
        let directory = loaded.directory_path().map(Path::to_path_buf);
        let mut templating = loaded.configuration.templating.unwrap_or_default();
        if let Some(arguments) = self.templating.clone() {
            templating.merge_with(arguments);
        }
        let settings =
            TemplatingSettings::new(directory, TemplatingConfiguration::from(templating))?;
        Ok(path.map_or(settings.syntax, |path| settings.template_of(&path)))
    }
}

/// Parse a SQL file (or standard input) and write the parse tree to standard
/// output as a protobuf `ParseResult` message.
///
/// # Arguments
/// * `input` - SQL text to parse
/// * `path` - path of the SQL file, used for diagnostics only
/// * `template` - the template syntax of the SQL
pub(crate) fn run_parse(
    input: &str,
    path: Option<&Path>,
    template: Template,
) -> Result<(), CliDiagnostic> {
    let mut console = EnvConsole::default();
    let failure = |error: &dyn std::fmt::Display| {
        CliDiagnostic::parse_sql_error(format!(
            "Failed to parse {}: {}",
            path.map(|path| path.to_string_lossy().into_owned())
                .unwrap_or_else(|| "<stdin>".to_string()),
            error
        ))
    };

    let preprocessed = template.read(input);
    if let Some(ambiguity) = preprocessed
        .as_ref()
        .and_then(|preprocessed| preprocessed.ambiguous.first())
    {
        return Err(failure(&ambiguity.message(input)));
    }
    let text = preprocessed
        .as_ref()
        .map_or(input, |preprocessed| &preprocessed.text);
    let normalized = pgls_lexer::convert_to_positional_params(text);
    let result = pgls_query::parse(&normalized).map_err(|error| failure(&error))?;

    if !result.warnings.is_empty() {
        console.log(markup! {
            "Parser warnings:" <Emphasis>{result.warnings.join("\n")}</Emphasis>
        });
    }

    let bytes = result.protobuf.encode_to_vec();
    std::io::stdout()
        .write_all(&bytes)
        .map_err(CliDiagnostic::io_error)?;
    std::io::stdout().flush().map_err(CliDiagnostic::io_error)
}

/// Read the SQL to parse from a file or standard input.
pub(crate) fn read_input(path: Option<&Path>) -> Result<String, CliDiagnostic> {
    match path {
        Some(path) => std::fs::read_to_string(path).map_err(|error| {
            CliDiagnostic::parse_sql_error(format!(
                "Failed to read {}: {}",
                path.to_string_lossy(),
                error
            ))
        }),
        None => {
            let mut buffer = String::new();
            std::io::stdin()
                .read_to_string(&mut buffer)
                .map_err(CliDiagnostic::io_error)?;
            Ok(buffer)
        }
    }
}
