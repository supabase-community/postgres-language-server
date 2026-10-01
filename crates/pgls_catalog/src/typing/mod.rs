//! Static type metadata and resolution contracts.
mod coerce;
mod common;
mod model;
mod normalize;
mod overload;
mod polymorphic;

pub use coerce::*;
pub use common::*;
pub use model::*;
pub use normalize::*;
pub use overload::*;
pub use polymorphic::*;
