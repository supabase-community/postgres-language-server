//! The database catalog, session state, and name resolution for statically analysing SQL files.
//!
//! - [`Snapshot`] (`snapshot/`): the objects of the connected database, loaded with the `db`
//!   feature or from JSON.
//! - [`CatalogBase`] (`catalog/base.rs`): the snapshot, indexed for lookups and shared.
//! - [`Catalog`] (`catalog/`): the base plus the changes made by earlier statements of the file.
//!   `catalog/ddl/<stmt>.rs` applies one kind of statement, and [`Catalog::snapshot`] turns the
//!   result back into a [`Snapshot`] for completions and hover.
//! - [`Session`] (`session/`): the search path and role that names are resolved with, and the
//!   lock and timeout state that the safety rules check.
//! - [`CatalogView`] (`lookup.rs`): the lookups the resolver makes. [`Catalog`] implements it.
//! - [`resolve`] (`resolve/`): name resolution of a single statement against a [`CatalogView`],
//!   with one module per parse node in `resolve/nodes/`.
//!
//! The analyser walks a file statement by statement. For each statement it runs the rules,
//! which resolve names against the current [`Catalog`] with the [`Session`]'s search path, and
//! then applies the statement to both: `catalog.apply(stmt, session.search_path())` and
//! `session.apply(stmt)`.

mod catalog;
mod lookup;
pub mod resolve;
mod session;
pub mod snapshot;
pub mod typing;

pub use catalog::{Catalog, CatalogBase};
pub use lookup::{
    CatalogView, ColumnInfo, FunctionInfo, FunctionKind, Lookup, Origin, RelationInfo,
    RelationKind, TypeInfo,
};
pub use session::{Session, is_reindex_concurrent, is_vacuum_full};
pub use snapshot::*;
pub use typing::*;
