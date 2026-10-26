//! The names Postgres gives to unnamed select list entries.

use pgls_query::{NodeEnum, protobuf};

const UNNAMED: &str = "?column?";

/// The column name Postgres chooses for an unnamed select list entry. Port of [`FigureColname`].
///
/// Returns `None` if we can't tell. A `*` expands to the columns it stands for, so it has no name
/// of its own.
///
/// [`FigureColname`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L1713
pub fn figure_column_name(value: &NodeEnum) -> Option<String> {
    figure_column_name_with(value, &|_| None)
}

/// Like [`figure_column_name`], but `subquery_column` names the first output column of a scalar
/// subquery whose name can't be read from its select list alone, like the first column of the
/// relation of `(select * from t)`.
pub(super) fn figure_column_name_with(
    value: &NodeEnum,
    subquery_column: SubqueryColumn<'_>,
) -> Option<String> {
    if is_star(value) {
        return None;
    }
    figure_column_name_with_strength(value, subquery_column).map(|(name, _)| name)
}

/// The name of the first output column of a scalar subquery, if it is known.
pub(super) type SubqueryColumn<'a> = &'a dyn Fn(&protobuf::SubLink) -> Option<String>;

/// Whether a select list entry is a `*` that expands to several columns, like `*`, `t.*` or
/// `(t.c).*`.
fn is_star(value: &NodeEnum) -> bool {
    let last = match value {
        NodeEnum::ColumnRef(column) => column.fields.last(),
        NodeEnum::AIndirection(indirection) => indirection.indirection.last(),
        _ => None,
    };
    matches!(
        last.and_then(|last| last.node.as_ref()),
        Some(NodeEnum::AStar(_))
    )
}

/// The last field name of a column reference or indirection, ignoring `*` and subscripts.
fn last_field_name(fields: &[pgls_query::protobuf::Node]) -> Option<&str> {
    fields
        .iter()
        .rev()
        .find_map(|field| match field.node.as_ref()? {
            NodeEnum::String(name) => Some(name.sval.as_str()),
            _ => None,
        })
}

/// The column name together with how strong that choice is. A type cast uses the name of its
/// argument only if that name is strong (2), and its type name otherwise. A CASE likewise uses the
/// name of its ELSE result only if it is strong, and `case` otherwise. Port of
/// [`FigureColnameInternal`].
///
/// [`FigureColnameInternal`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L1752
fn figure_column_name_with_strength(
    value: &NodeEnum,
    subquery_column: SubqueryColumn<'_>,
) -> Option<(String, u8)> {
    let recurse = |node: &NodeEnum| figure_column_name_with_strength(node, subquery_column);
    let named = |name: &str| Some((name.to_owned(), 2));
    match value {
        NodeEnum::ColumnRef(column) => match last_field_name(&column.fields) {
            Some(name) => named(name),
            None => Some((UNNAMED.into(), 0)),
        },
        NodeEnum::AIndirection(indirection) => match last_field_name(&indirection.indirection) {
            Some(name) => named(name),
            None => recurse(indirection.arg.as_deref()?.node.as_ref()?),
        },
        NodeEnum::FuncCall(call) => match call.funcname.last()?.node.as_ref()? {
            NodeEnum::String(name) => named(&name.sval),
            _ => None,
        },
        NodeEnum::AExpr(expr) if expr.kind() == protobuf::AExprKind::AexprNullif => named("nullif"),
        NodeEnum::TypeCast(cast) => {
            let inner = recurse(cast.arg.as_deref()?.node.as_ref()?)?;
            if inner.1 > 1 {
                Some(inner)
            } else {
                let type_name = cast.type_name.as_ref()?;
                match type_name.names.last()?.node.as_ref()? {
                    NodeEnum::String(name) => Some((name.sval.clone(), 1)),
                    _ => None,
                }
            }
        }
        NodeEnum::CollateClause(collate) => recurse(collate.arg.as_deref()?.node.as_ref()?),
        NodeEnum::GroupingFunc(_) => named("grouping"),
        NodeEnum::SubLink(link) => match link.sub_link_type() {
            protobuf::SubLinkType::ExistsSublink => named("exists"),
            protobuf::SubLinkType::ArraySublink => named("array"),
            // Postgres transforms the subquery first and takes the name of its column.
            protobuf::SubLinkType::ExprSublink => {
                let NodeEnum::SelectStmt(select) = link.subselect.as_deref()?.node.as_ref()? else {
                    return None;
                };
                let name =
                    first_column_name(select, subquery_column).or_else(|| subquery_column(link))?;
                Some((name, 2))
            }
            _ => Some((UNNAMED.into(), 0)),
        },
        NodeEnum::CaseExpr(case) => {
            let default = match case.defresult.as_deref() {
                Some(default) => recurse(default.node.as_ref()?)?,
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

/// The name of the first output column of a subquery, from its select list. A set operation takes
/// the names of its left-most select ([`transformSetOperationStmt`]), and VALUES names its columns
/// `column1`, `column2`, ... ([`addRangeTableEntryForValues`]). `None` for a `*`, which expands to
/// the columns of its FROM items.
///
/// [`transformSetOperationStmt`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/analyze.c#L1751
/// [`addRangeTableEntryForValues`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_relation.c#L2183
fn first_column_name(
    select: &protobuf::SelectStmt,
    subquery_column: SubqueryColumn<'_>,
) -> Option<String> {
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
    if !target.name.is_empty() {
        return Some(target.name.clone());
    }
    figure_column_name_with(target.val.as_deref()?.node.as_ref()?, subquery_column)
}
