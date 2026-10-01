use super::*;

pub(super) fn infer_column_ref(
    r: &mut Resolver<'_>,
    n: &pgls_query::protobuf::ColumnRef,
) -> Option<Type> {
    {
        let names = string_values(&n.fields)?;
        match names.as_slice() {
            // A column takes precedence over a whole-row reference to a table.
            [column] => find_column_type(&r.levels, column)
                .or_else(|| {
                    find_item(&r.levels, column)
                        .and_then(|item| item.typed_columns.clone().map(Type::Record))
                })
                .or_else(|| {
                    let param = r.function_param(column)?;
                    function_param_type(r, param)
                }),
            [table, column] => {
                find_item(&r.levels, table).and_then(|item| item.type_of(column).flatten())
            }
            _ => None,
        }
    }
}
