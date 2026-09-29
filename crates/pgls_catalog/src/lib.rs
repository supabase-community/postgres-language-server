//! In-memory catalog, session state, and name resolution for statically analysing SQL files.
//!
//! - [`Catalog`]: the database snapshot plus the changes made by earlier statements of the file.
//! - [`Session`]: search path, transaction, lock, and timeout state of the file's session.
//! - [`resolve`]: name resolution of a single statement against a [`CatalogView`].

mod catalog;
pub mod resolve;
mod session;
mod view;

pub use catalog::{Catalog, CatalogBase};
pub use session::{Session, is_reindex_concurrent, is_vacuum_full};
pub use view::{
    CatalogView, ColumnInfo, FunctionInfo, FunctionKind, Lookup, Origin, RelationInfo,
    RelationKind, TypeInfo,
};
