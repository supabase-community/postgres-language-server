//! Schema export command for WASM bindings.
//!
//! This command connects to a PostgreSQL database and exports the schema cache
//! as a JSON file that can be used with the WASM bindings.

use pgls_console::{ConsoleExt, EnvConsole, markup};
use pgls_schema_cache::SchemaCache;
use sqlx::postgres::PgPoolOptions;
use std::path::Path;

use crate::CliDiagnostic;

/// Export the database schema to a JSON file.
///
/// # Arguments
/// * `connection_string` - PostgreSQL connection string
/// * `output_path` - Path to write the JSON output
pub async fn run_schema_export(
    connection_string: &str,
    output_path: &Path,
) -> Result<(), CliDiagnostic> {
    let mut console = EnvConsole::default();

    console.log(markup! {
        "Connecting to database..."
    });

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

    console.log(markup! {
        "Loading schema cache..."
    });

    // Load the schema cache
    let schema_cache = SchemaCache::load(&pool).await.map_err(|e| {
        CliDiagnostic::io_error(std::io::Error::other(format!(
            "Failed to load schema cache: {e}"
        )))
    })?;

    console.log(markup! {
        "Serializing schema..."
    });

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&schema_cache).map_err(|e| {
        CliDiagnostic::io_error(std::io::Error::other(format!(
            "Failed to serialize schema: {e}"
        )))
    })?;

    // Write to file
    std::fs::write(output_path, json)
        .map_err(|e| CliDiagnostic::io_error(std::io::Error::other(format!(
            "Failed to write output file: {e}"
        ))))?;

    console.log(markup! {
        "Schema exported to "<Emphasis>{output_path.display().to_string()}</Emphasis>
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

    Ok(())
}
