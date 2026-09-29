//! The database catalog, session state, and name resolution for statically analysing SQL files.
//!
//! - [`Snapshot`]: the objects of the connected database, loaded with the `db` feature or from
//!   JSON.
//! - [`CatalogBase`]: the snapshot, indexed for lookups.
//! - [`Catalog`]: the database snapshot plus the changes made by earlier statements of the file.
//! - [`Session`]: search path, transaction, lock, and timeout state of the file's session.
//! - [`resolve`]: name resolution of a single statement against a [`CatalogView`].

mod catalog;
mod column_name;
pub mod resolve;
mod search_path;
mod session;
pub mod snapshot;
mod view;

pub use catalog::{Catalog, CatalogBase};
pub use search_path::expand_search_path;
pub use session::{Session, is_reindex_concurrent, is_vacuum_full};
pub use snapshot::*;
pub use view::{
    CatalogView, ColumnInfo, FunctionInfo, FunctionKind, Lookup, Origin, RelationInfo,
    RelationKind, TypeInfo,
};
