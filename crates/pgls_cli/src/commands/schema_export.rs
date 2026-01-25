//! Schema export command for WASM bindings.
//!
//! This command connects to a PostgreSQL database and exports the schema cache
//! as JSON that can be used with the WASM bindings.

use pgls_console::{ConsoleExt, EnvConsole, markup};
use pgls_schema_cache::SchemaCache;
use sqlx::postgres::PgPoolOptions;
use std::io::Write;
use std::path::Path;

use crate::CliDiagnostic;

/// Export the database schema to JSON.
///
/// # Arguments
/// * `connection_string` - PostgreSQL connection string
/// * `output_path` - Optional path to write the JSON output (stdout if None)
pub async fn run_schema_export(
    connection_string: &str,
    output_path: Option<&Path>,
) -> Result<(), CliDiagnostic> {
    let mut console = EnvConsole::default();

    // Only print progress to stderr if we're writing to stdout,
    // so the JSON output is clean and can be piped
    let write_to_stdout = output_path.is_none();

    if !write_to_stdout {
        console.log(markup! {
            "Connecting to database..."
        });
    }

    // Connect to the database
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(connection_string)
        .await
        .map_err(|e| {
            CliDiagnostic::io_error(std::io::Error::other(format!(
                "Failed to connect to database: {e}"
            )))
        })?;

    if !write_to_stdout {
        console.log(markup! {
            "Loading schema cache..."
        });
    }

    // Load the schema cache
    let schema_cache = SchemaCache::load(&pool).await.map_err(|e| {
        CliDiagnostic::io_error(std::io::Error::other(format!(
            "Failed to load schema cache: {e}"
        )))
    })?;

    if !write_to_stdout {
        console.log(markup! {
            "Serializing schema..."
        });
    }

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&schema_cache).map_err(|e| {
        CliDiagnostic::io_error(std::io::Error::other(format!(
            "Failed to serialize schema: {e}"
        )))
    })?;

    // Write output
    if let Some(path) = output_path {
        std::fs::write(path, &json).map_err(|e| {
            CliDiagnostic::io_error(std::io::Error::other(format!(
                "Failed to write output file: {e}"
            )))
        })?;

        console.log(markup! {
            "Schema exported to "<Emphasis>{path.display().to_string()}</Emphasis>
        });

        // Print summary
        console.log(markup! {
            "\nSchema summary:"
        });
        console.log(markup! {
            "  Schemas: "{schema_cache.schemas.len().to_string()}
        });
        console.log(markup! {
            "  Tables: "{schema_cache.tables.len().to_string()}
        });
        console.log(markup! {
            "  Functions: "{schema_cache.functions.len().to_string()}
        });
        console.log(markup! {
            "  Types: "{schema_cache.types.len().to_string()}
        });
    } else {
        // Write to stdout
        std::io::stdout().write_all(json.as_bytes()).map_err(|e| {
            CliDiagnostic::io_error(std::io::Error::other(format!(
                "Failed to write to stdout: {e}"
            )))
        })?;
        std::io::stdout().write_all(b"\n").map_err(|e| {
            CliDiagnostic::io_error(std::io::Error::other(format!(
                "Failed to write to stdout: {e}"
            )))
        })?;
    }

    Ok(())
}
