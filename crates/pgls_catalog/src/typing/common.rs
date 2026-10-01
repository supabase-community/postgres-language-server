use crate::{
    CatalogView,
    typing::{Selection, Type},
};
/// Selects Postgres' common type for a set of expressions.
pub fn select_common_type(_: &dyn CatalogView, _: &[Type]) -> Selection<Type> {
    Selection::Unknown
}
