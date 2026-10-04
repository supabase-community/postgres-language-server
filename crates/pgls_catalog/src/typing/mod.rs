//! Postgres' type rules, ported from the parser: what a type is, whether one type can become
//! another, which type several inputs agree on, and which function or operator a call picks.
//!
//! The functions here only answer questions about types. The facts they need come from
//! [`crate::CatalogView`], and [`crate::resolve`] walks statements and asks them. Every
//! answer can be unknown, and unknown never proves an error.
//!
//! The algorithms are ported from the latest supported Postgres, [`REL_18_6`], and each port
//! links the function it follows. Postgres 15 to 17 resolve types the same way.
//!
//! Not modelled yet, so always unknown: range and multirange polymorphism, ordered-set and
//! hypothetical aggregates, `VARIADIC` calls, custom subscripting (like `jsonb['key']`),
//! conversions between composite types through inheritance, and row comparisons other than
//! equality. Typmods, domain constraints and the contents of literals are checked at runtime by
//! Postgres and are ignored.
//!
//! [`REL_18_6`]: https://github.com/postgres/postgres/tree/REL_18_6/src/backend/parser

mod coerce;
mod model;
mod overload;
mod types;
mod unify;

pub use coerce::*;
pub use model::*;
pub use overload::*;
pub use types::*;
pub use unify::*;
