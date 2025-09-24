use pgls_console::fmt::Display;
use pgls_console::{MarkupBuf, markup};
use pgls_diagnostics::adapters::ResolveError;

use pgls_diagnostics::{Advices, Diagnostic, Error, LogCategory, MessageAndDescription, Visit};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};

/// Series of errors that can be thrown while computing the configuration.
#[derive(Deserialize, Diagnostic, Serialize)]
pub enum ConfigurationDiagnostic {
    /// Thrown when the program can't serialize the configuration, while saving it
    SerializationError(SerializationError),

    /// Error thrown when de-serialising the configuration from file
    DeserializationError(DeserializationError),

    /// Thrown when trying to **create** a new configuration file, but it exists already
    ConfigAlreadyExists(ConfigAlreadyExists),

    /// When something is wrong with the configuration
    InvalidConfiguration(InvalidConfiguration),

    /// Thrown when the pattern inside the `ignore` field errors
    InvalidIgnorePattern(InvalidIgnorePattern),

    /// Thrown when there's something wrong with the files specified inside `"extends"`
    CantLoadExtendFile(CantLoadExtendFile),

    /// Thrown when a configuration file can't be resolved from `node_modules`
    CantResolve(CantResolve),
}

impl ConfigurationDiagnostic {
    pub fn new_deserialization_error(error: serde_json::Error) -> Self {
        Self::DeserializationError(DeserializationError {
            message: error.to_string(),
        })
    }

    pub fn new_serialization_error() -> Self {
        Self::SerializationError(SerializationError)
    }

    pub fn new_invalid_ignore_pattern(
        pattern: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::InvalidIgnorePattern(InvalidIgnorePattern {
            message: format!(
                "Couldn't parse the pattern \"{}\". Reason: {}",
                pattern.into(),
                reason.into()
            ),
            file_path: None,
        })
    }

    pub fn new_invalid_ignore_pattern_with_path(
        pattern: impl Into<String>,
        reason: impl Into<String>,
        file_path: Option<impl Into<String>>,
    ) -> Self {
        Self::InvalidIgnorePattern(InvalidIgnorePattern {
            message: format!(
                "Couldn't parse the pattern \"{}\". Reason: {}",
                pattern.into(),
                reason.into()
            ),
            file_path: file_path.map(|f| f.into()),
        })
    }

    pub fn new_already_exists() -> Self {
        Self::ConfigAlreadyExists(ConfigAlreadyExists {})
    }

    pub fn invalid_configuration(message: impl Display) -> Self {
        Self::InvalidConfiguration(InvalidConfiguration {
            message: MessageAndDescription::from(markup! {{message}}.to_owned()),
        })
    }

    pub fn cant_resolve(path: impl Display, source: oxc_resolver::ResolveError) -> Self {
        Self::CantResolve(CantResolve {
            message: MessageAndDescription::from(
                markup! {
                   "Failed to resolve the configuration from "{{path}}
                }
                .to_owned(),
            ),
            source: Some(Error::from(ResolveError::from(source))),
        })
    }
}

impl Debug for ConfigurationDiagnostic {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl std::fmt::Display for ConfigurationDiagnostic {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.description(f)
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ConfigurationAdvices {
    messages: Vec<MarkupBuf>,
}

impl Advices for ConfigurationAdvices {
    fn record(&self, visitor: &mut dyn Visit) -> std::io::Result<()> {
        for message in &self.messages {
            visitor.record_log(LogCategory::Info, message)?;
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Diagnostic)]
#[diagnostic(
    message = "Failed to deserialize",
    category = "configuration",
    severity = Error
)]
pub struct DeserializationError {
    #[message]
    #[description]
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Diagnostic)]
#[diagnostic(
    message = "Failed to serialize",
    category = "configuration",
    severity = Error
)]
pub struct SerializationError;

#[derive(Debug, Serialize, Deserialize, Diagnostic)]
#[diagnostic(
    message = "It seems that a configuration file already exists",
    category = "configuration",
    severity = Error
)]
pub struct ConfigAlreadyExists {}

#[derive(Debug, Serialize, Deserialize, Diagnostic)]
#[diagnostic(
    category = "configuration",
    severity = Error,
)]
pub struct InvalidIgnorePattern {
    #[message]
    #[description]
    pub message: String,

    #[location(resource)]
    pub file_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Diagnostic)]
#[diagnostic(
	category = "configuration",
	severity = Error,
)]
pub struct InvalidConfiguration {
    #[message]
    #[description]
    message: MessageAndDescription,
}

#[derive(Debug, Serialize, Deserialize, Diagnostic)]
#[diagnostic(
    category = "configuration",
    severity = Error,
)]
pub struct CantResolve {
    #[message]
    #[description]
    message: MessageAndDescription,

    #[serde(skip)]
    #[source]
    source: Option<Error>,
}

#[derive(Debug, Serialize, Deserialize, Diagnostic)]
#[diagnostic(
	category = "configuration",
	severity = Error,
)]
pub struct CantLoadExtendFile {
    #[location(resource)]
    file_path: String,
    #[message]
    #[description]
    message: MessageAndDescription,

    #[verbose_advice]
    verbose_advice: ConfigurationAdvices,
}

impl CantLoadExtendFile {
    pub fn new(file_path: impl Into<String>, message: impl Display) -> Self {
        Self {
            file_path: file_path.into(),
            message: MessageAndDescription::from(markup! {{message}}.to_owned()),
            verbose_advice: ConfigurationAdvices::default(),
        }
    }

    pub fn with_verbose_advice(mut self, messsage: impl Display) -> Self {
        self.verbose_advice
            .messages
            .push(markup! {{messsage}}.to_owned());
        self
    }
}
