use super::*;

/// `CURRENT_DATE`, `CURRENT_USER`, and the other SQL value functions. Port of
/// [`transformSQLValueFunction`].
///
/// [`transformSQLValueFunction`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_expr.c#L2314
pub(super) fn infer_sql_value_function(
    r: &mut Resolver<'_>,
    n: &pgls_query::protobuf::SqlValueFunction,
) -> Option<Type> {
    use protobuf::SqlValueFunctionOp as Op;
    let name = match Op::try_from(n.op).ok()? {
        Op::SvfopCurrentDate => "date",
        Op::SvfopCurrentTime | Op::SvfopCurrentTimeN => "timetz",
        Op::SvfopLocaltime | Op::SvfopLocaltimeN => "time",
        Op::SvfopCurrentTimestamp | Op::SvfopCurrentTimestampN => "timestamptz",
        Op::SvfopLocaltimestamp | Op::SvfopLocaltimestampN => "timestamp",
        Op::SvfopCurrentRole
        | Op::SvfopCurrentUser
        | Op::SvfopUser
        | Op::SvfopSessionUser
        | Op::SvfopCurrentCatalog
        | Op::SvfopCurrentSchema => "name",
        _ => return None,
    };
    named(r, name)
}
