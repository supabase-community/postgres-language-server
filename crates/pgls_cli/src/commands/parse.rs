//! Parse command: emit the libpg_query parse tree for consumption by other
//! tools.
//!
//! The output is the `ParseResult` protobuf message defined by libpg_query,
//! the same contract the official bindings of other languages expose, so a
//! consumer can decode it with any generated `pg_query` client. The parse
//! reflects how the rest of the CLI reads SQL: psql named parameters
//! (`:'name'`, `:name`) become parser-compatible placeholders of identical
//! byte length, keeping every location of the tree aligned with the input.

use std::io::{Read, Write};
use std::path::Path;

use pgls_console::{ConsoleExt, EnvConsole, markup};
use prost::Message;

use crate::CliDiagnostic;

/// Parse a SQL file (or standard input) and write the parse tree to standard
/// output as a protobuf `ParseResult` message.
///
/// # Arguments
/// * `input` - SQL text to parse
/// * `path` - path of the SQL file, used for diagnostics only
pub fn run_parse(input: &str, path: Option<&Path>) -> Result<(), CliDiagnostic> {
    let mut console = EnvConsole::default();

    let normalized = pgls_lexer::convert_to_positional_params(input);
    let result = pgls_query::parse(&normalized).map_err(|error| {
        CliDiagnostic::parse_sql_error(format!(
            "Failed to parse {}: {}",
            path.map(|path| path.to_string_lossy().into_owned())
                .unwrap_or_else(|| "<stdin>".to_string()),
            error
        ))
    })?;

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
pub fn read_input(path: Option<&Path>) -> Result<String, CliDiagnostic> {
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
