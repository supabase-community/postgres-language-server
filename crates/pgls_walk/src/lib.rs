//! The walk over the statements of a file. Lint and the workspace's editor features (hover,
//! completions, inlay hints, semantic tokens, code actions) share it, so that they see the same
//! catalog and session at each statement.

mod file_context;
mod migration;
mod statement;
mod walk;

pub use file_context::AnalysedFileContext;
pub use migration::{Direction, MigrationSection};
pub use statement::{AnalysableStatement, UnparsableStatement};
pub use walk::{WalkParams, is_plain_query, walk_statements};
