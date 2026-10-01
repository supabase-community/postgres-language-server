use crate::{
    CatalogView,
    typing::{CallArg, OperatorKind, ResolvedFunction, ResolvedOperator, Selection, Type},
};
/// Selects an operator overload.
pub fn select_operator(
    _: &dyn CatalogView,
    _: Option<&str>,
    _: &str,
    _: OperatorKind,
    _: Option<&Type>,
    _: &Type,
    _: &[String],
) -> Selection<ResolvedOperator> {
    Selection::Unknown
}
/// Selects a function overload.
pub fn select_function(
    _: &dyn CatalogView,
    _: Option<&str>,
    _: &str,
    _: &[CallArg],
    _: &[String],
) -> Selection<ResolvedFunction> {
    Selection::Unknown
}
