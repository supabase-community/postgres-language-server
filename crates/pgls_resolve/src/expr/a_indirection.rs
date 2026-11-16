use pgls_query::NodeEnum;

use super::infer_expr;
use crate::{resolver::Resolver, utils::field_type};
use pgls_catalog::{
    Lookup,
    typing::{Decision, Type},
};

/// Subscripts and field selection. Port of [`transformIndirection`] for true arrays
/// ([`transformContainerSubscripts`]) and composite fields ([`ParseComplexProjection`]).
///
/// [`transformIndirection`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_expr.c#L437
/// [`transformContainerSubscripts`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_node.c#L243
/// [`ParseComplexProjection`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_func.c#L1967
pub(super) fn infer_a_indirection(
    r: &mut Resolver<'_>,
    n: &pgls_query::protobuf::AIndirection,
) -> Option<Type> {
    infer_indirection(r, n.arg.as_deref(), &n.indirection)
}

/// The type of `arg` followed by the subscripts and field selections of `indirection`, like
/// [`infer_a_indirection`]. For `(expr).*`, `indirection` is the part before the `*`.
pub(crate) fn infer_indirection(
    r: &mut Resolver<'_>,
    arg: Option<&pgls_query::Node>,
    indirection: &[pgls_query::Node],
) -> Option<Type> {
    {
        let mut ty = arg
            .and_then(|x| x.node.as_ref())
            .and_then(|x| infer_expr(r, x))?;
        let mut items = indirection.iter().peekable();
        while let Some(item) = items.next() {
            ty = match item.node.as_ref()? {
                // Consecutive subscripts index one array: `a[1][2]` is an element of a
                // two-dimensional array, and any slice makes the result an array.
                NodeEnum::AIndices(indices) => {
                    let mut slice = indices.is_slice;
                    while let Some(NodeEnum::AIndices(next)) =
                        items.peek().and_then(|item| item.node.as_ref())
                    {
                        slice |= next.is_slice;
                        items.next();
                    }
                    let Type::Named(id) = &ty else { return None };
                    let Decision::Known(array) = pgls_catalog::typing::base_type(r.catalog, id)
                    else {
                        return None;
                    };
                    let Lookup::Found(info) = r.catalog.type_by_id(&array) else {
                        return None;
                    };
                    // Only true arrays: types like jsonb have their own subscripting.
                    let element = info.element?;
                    Type::Named(if slice { array } else { element })
                }
                NodeEnum::String(field) => field_type(r, &ty, &field.sval)?,
                _ => return None,
            };
        }
        Some(ty)
    }
}
