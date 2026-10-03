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
