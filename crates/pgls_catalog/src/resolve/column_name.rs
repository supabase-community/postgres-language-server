//! The names Postgres gives to unnamed select list entries.

use pgls_query::{NodeEnum, protobuf};
use pgls_query_ext::utils::{is_star, string_value};

const UNNAMED: &str = "?column?";

/// The name of a select list entry: its alias, or else the name Postgres chooses for it
/// ([`figure_column_name`]).
pub fn target_name(target: &protobuf::ResTarget) -> Option<String> {
    if !target.name.is_empty() {
        return Some(target.name.clone());
    }
    figure_column_name(target.val.as_deref()?.node.as_ref()?)
}

/// The column name Postgres chooses for an unnamed select list entry. Port of [`FigureColname`].
///
/// Returns `None` if we can't tell. A `*` expands to the columns it stands for, so it has no name
/// of its own.
///
/// [`FigureColname`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L1713
pub fn figure_column_name(value: &NodeEnum) -> Option<String> {
    if is_star(value) {
        return None;
    }
    figure_column_name_with_strength(value).map(|(name, _)| name)
}

/// The last field name of a column reference or indirection, ignoring `*` and subscripts.
fn last_field_name(fields: &[pgls_query::protobuf::Node]) -> Option<&str> {
    fields.iter().rev().find_map(string_value)
}

/// The column name together with how strong that choice is. A type cast uses the name of its
/// argument only if that name is strong (2), and its type name otherwise. A CASE likewise uses the
/// name of its ELSE result only if it is strong, and `case` otherwise. Port of
/// [`FigureColnameInternal`].
///
/// [`FigureColnameInternal`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L1752
fn figure_column_name_with_strength(value: &NodeEnum) -> Option<(String, u8)> {
    let named = |name: &str| Some((name.to_owned(), 2));
    match value {
        NodeEnum::ColumnRef(column) => match last_field_name(&column.fields) {
            Some(name) => named(name),
            None => Some((UNNAMED.into(), 0)),
        },
        NodeEnum::AIndirection(indirection) => match last_field_name(&indirection.indirection) {
            Some(name) => named(name),
            None => figure_column_name_with_strength(indirection.arg.as_deref()?.node.as_ref()?),
        },
        NodeEnum::FuncCall(call) => named(string_value(call.funcname.last()?)?),
        NodeEnum::AExpr(expr) if expr.kind() == protobuf::AExprKind::AexprNullif => named("nullif"),
        NodeEnum::TypeCast(cast) => {
            let inner = figure_column_name_with_strength(cast.arg.as_deref()?.node.as_ref()?)?;
            if inner.1 > 1 {
                Some(inner)
            } else {
                let type_name = cast.type_name.as_ref()?;
                Some((string_value(type_name.names.last()?)?.to_owned(), 1))
            }
        }
        NodeEnum::CollateClause(collate) => {
            figure_column_name_with_strength(collate.arg.as_deref()?.node.as_ref()?)
        }
        NodeEnum::GroupingFunc(_) => named("grouping"),
        NodeEnum::SubLink(link) => match link.sub_link_type() {
            protobuf::SubLinkType::ExistsSublink => named("exists"),
            protobuf::SubLinkType::ArraySublink => named("array"),
            // Postgres transforms the subquery first and takes the name of its column.
            protobuf::SubLinkType::ExprSublink => {
                let NodeEnum::SelectStmt(select) = link.subselect.as_deref()?.node.as_ref()? else {
                    return None;
                };
                Some((first_column_name(select)?, 2))
            }
            _ => Some((UNNAMED.into(), 0)),
        },
        NodeEnum::CaseExpr(case) => {
            let default = match case.defresult.as_deref() {
                Some(default) => figure_column_name_with_strength(default.node.as_ref()?)?,
                None => (UNNAMED.into(), 0),
            };
            if default.1 > 1 {
                Some(default)
            } else {
                Some(("case".into(), 1))
            }
        }
        NodeEnum::AArrayExpr(_) => named("array"),
        NodeEnum::RowExpr(_) => named("row"),
        NodeEnum::CoalesceExpr(_) => named("coalesce"),
        NodeEnum::MinMaxExpr(expr) => match expr.op() {
            protobuf::MinMaxOp::IsGreatest => named("greatest"),
            protobuf::MinMaxOp::IsLeast => named("least"),
            _ => None,
        },
        NodeEnum::SqlvalueFunction(function) => {
            use protobuf::SqlValueFunctionOp as Op;
            named(match function.op() {
                Op::SvfopCurrentDate => "current_date",
                Op::SvfopCurrentTime | Op::SvfopCurrentTimeN => "current_time",
                Op::SvfopCurrentTimestamp | Op::SvfopCurrentTimestampN => "current_timestamp",
                Op::SvfopLocaltime | Op::SvfopLocaltimeN => "localtime",
                Op::SvfopLocaltimestamp | Op::SvfopLocaltimestampN => "localtimestamp",
                Op::SvfopCurrentRole => "current_role",
                Op::SvfopCurrentUser => "current_user",
                Op::SvfopUser => "user",
                Op::SvfopSessionUser => "session_user",
                Op::SvfopCurrentCatalog => "current_catalog",
                Op::SvfopCurrentSchema => "current_schema",
                _ => return None,
            })
        }
        NodeEnum::MergeSupportFunc(_) => named("merge_action"),
        NodeEnum::XmlExpr(expr) => {
            use protobuf::XmlExprOp as Op;
            match expr.op() {
                Op::IsXmlconcat => named("xmlconcat"),
                Op::IsXmlelement => named("xmlelement"),
                Op::IsXmlforest => named("xmlforest"),
                Op::IsXmlparse => named("xmlparse"),
                Op::IsXmlpi => named("xmlpi"),
                Op::IsXmlroot => named("xmlroot"),
                Op::IsXmlserialize => named("xmlserialize"),
                Op::IsDocument => Some((UNNAMED.into(), 0)),
                Op::Undefined => None,
            }
        }
        NodeEnum::XmlSerialize(_) => named("xmlserialize"),
        NodeEnum::JsonParseExpr(_) => named("json"),
        NodeEnum::JsonScalarExpr(_) => named("json_scalar"),
        NodeEnum::JsonSerializeExpr(_) => named("json_serialize"),
        NodeEnum::JsonObjectConstructor(_) => named("json_object"),
        NodeEnum::JsonArrayConstructor(_) | NodeEnum::JsonArrayQueryConstructor(_) => {
            named("json_array")
        }
        NodeEnum::JsonObjectAgg(_) => named("json_objectagg"),
        NodeEnum::JsonArrayAgg(_) => named("json_arrayagg"),
        NodeEnum::JsonFuncExpr(expr) => {
            use protobuf::JsonExprOp as Op;
            match expr.op() {
                Op::JsonExistsOp => named("json_exists"),
                Op::JsonQueryOp => named("json_query"),
                Op::JsonValueOp => named("json_value"),
                Op::JsonTableOp | Op::Undefined => None,
            }
        }
        NodeEnum::AConst(_)
        | NodeEnum::AExpr(_)
        | NodeEnum::BoolExpr(_)
        | NodeEnum::NullTest(_)
        | NodeEnum::BooleanTest(_)
        | NodeEnum::ParamRef(_) => Some((UNNAMED.into(), 0)),
        _ => None,
    }
}

/// The name of the first output column of a subquery. A set operation takes the names of its
/// left-most select ([`transformSetOperationStmt`]), and VALUES names its columns `column1`,
/// `column2`, ... ([`addRangeTableEntryForValues`]).
///
/// [`transformSetOperationStmt`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/analyze.c#L1751
/// [`addRangeTableEntryForValues`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_relation.c#L2183
fn first_column_name(select: &protobuf::SelectStmt) -> Option<String> {
    let mut select = select;
    while select.op() != protobuf::SetOperation::SetopNone {
        select = select.larg.as_deref()?;
    }
    if !select.values_lists.is_empty() {
        return Some("column1".into());
    }
    let NodeEnum::ResTarget(target) = select.target_list.first()?.node.as_ref()? else {
        return None;
    };
    target_name(target)
}
