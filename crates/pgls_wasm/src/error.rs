//! Error types for the WASM bindings.

use std::fmt;

/// Error type for WASM operations.
#[derive(Debug)]
pub enum WasmError {
    /// File not found in the virtual file system.
    FileNotFound(String),
    /// Invalid schema JSON.
    InvalidSchema(String),
    /// SQL parsing error.
    ParseError(String),
    /// Internal error.
    Internal(String),
}

impl fmt::Display for WasmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WasmError::FileNotFound(msg) => write!(f, "File not found: {msg}"),
            WasmError::InvalidSchema(msg) => write!(f, "Invalid schema: {msg}"),
            WasmError::ParseError(msg) => write!(f, "Parse error: {msg}"),
            WasmError::Internal(msg) => write!(f, "Internal error: {msg}"),
        }
    }
}

impl std::error::Error for WasmError {}
