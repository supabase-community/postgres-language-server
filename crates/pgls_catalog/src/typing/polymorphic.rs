use crate::typing::{Decision, Type, TypeId};
/// Resolves a polymorphic type against the supplied concrete type.
pub fn resolve_polymorphic(_: &TypeId, _: &[Type]) -> Decision<Type> {
    Decision::Unknown
}
