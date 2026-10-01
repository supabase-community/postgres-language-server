use crate::{
    CatalogView,
    typing::{CoercionContext, Decision, Type, TypeId},
};
/// Determines whether a coercion is known to be allowed.
pub fn can_coerce(_: &dyn CatalogView, _: &Type, _: &TypeId, _: CoercionContext) -> Decision<bool> {
    Decision::Unknown
}
