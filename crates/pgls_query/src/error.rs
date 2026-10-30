use thiserror::Error;

/// Error structure representing the basic error scenarios for `pg_query`.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum Error {
    #[error("Invalid statement format: {0}")]
    Conversion(#[from] std::ffi::NulError),
    #[error("Error decoding result: {0}")]
    Decode(#[from] prost::DecodeError),
    #[error("Invalid statement: {0}")]
    Parse(String),
    #[error("Error parsing JSON: {0}")]
    InvalidJson(String),
    #[error("Invalid pointer")]
    InvalidPointer,
    #[error("Error scanning: {0}")]
    Scan(String),
    #[error("Error splitting: {0}")]
    Split(String),
    #[error(
        "The statement is nested too deeply: its parse tree may be at most {} levels deep",
        crate::MAX_NESTING
    )]
    TooDeep,
}

/// Convenient Result alias for returning `pg_query::Error`.
pub type Result<T> = core::result::Result<T, Error>;
