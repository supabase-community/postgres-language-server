//! The database catalog and session state for statically analysing SQL files.
//!
//! - [`Snapshot`] (`snapshot/`): the objects of the connected database, loaded with the `db`
//!   feature or from JSON.
//! - [`CatalogBase`] (`catalog/base.rs`): the snapshot, indexed for lookups and shared.
//! - [`Catalog`] (`catalog/`): the base plus the changes made by earlier statements of the file.
//!   `catalog/ddl/<stmt>.rs` applies one kind of statement, and [`Catalog::snapshot`] turns the
//!   result back into a [`Snapshot`] for completions and hover.
//! - [`Session`] (`session/`): the search path and role that names are resolved with, and the
//!   lock and timeout state that the safety rules check.
//! - `lookup.rs`: what the lookups of [`Catalog`] return.
//! - [`typing`] (`typing/`): Postgres' type rules, which the resolver of `pgls_resolve` asks.
//!
//! The analyser walks a file statement by statement. For each statement it resolves its names
//! (`pgls_resolve`) once against the current [`Catalog`] with the [`Session`]'s search path, runs the rules on
//! that resolution, and then applies the statement to both:
//! `catalog.apply(stmt, session.search_path(), output)`, where `output` is the output columns of
//! the statement's query from its resolution, then `session.apply(stmt, &catalog)`, which looks
//! up the names of the statement in the catalog.

mod catalog;
mod lookup;
mod session;
pub mod snapshot;
pub mod typing;

pub use catalog::{Catalog, CatalogBase, rename};
pub use lookup::{
    Candidates, CastInfo, CastMethod, ColumnInfo, FunctionArgument, FunctionArgumentMode,
    FunctionInfo, FunctionKind, FunctionSignature, IndexInfo, Lookup, OperatorInfo, OperatorKind,
    Origin, RelationInfo, RelationKind, TypeInfo, TypeKind,
};
pub use session::{
    LockMode, NonTransactional, Session, TableLock, TransactionMode, alter_table_cmd_lock,
    alter_table_lock, ends_transaction_block, prevents_transaction_block, relation_locks,
};
pub use snapshot::*;
pub use typing::{
    Decision, SerialType, Type, TypeId, TypedColumn, base_type, is_binary_coercible,
    normalize_type_name, serial_type,
};
