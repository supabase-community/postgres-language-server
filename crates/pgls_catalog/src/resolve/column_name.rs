//! The names Postgres gives to unnamed select list entries.

use pgls_query::{NodeEnum, protobuf};

const UNNAMED: &str = "?column?";

/// The column name Postgres chooses for an unnamed select list entry (`FigureColname`).
///
/// Returns `None` if we can't tell.
pub(crate) fn figure_column_name(value: &NodeEnum) -> Option<String> {
    figure_column_name_with_strength(value).map(|(name, _)| name)
}

/// The column name together with how strong that choice is. A type cast uses the name of its
/// argument only if that name is strong (2), and its type name otherwise.
fn figure_column_name_with_strength(value: &NodeEnum) -> Option<(String, u8)> {
    let named = |name: &str| Some((name.to_owned(), 2));
    match value {
        NodeEnum::ColumnRef(column) => match column.fields.last()?.node.as_ref()? {
            NodeEnum::String(name) => named(&name.sval),
            _ => Some((UNNAMED.into(), 0)),
        },
        NodeEnum::AIndirection(indirection) => {
            match indirection.indirection.last()?.node.as_ref()? {
                NodeEnum::String(name) => named(&name.sval),
                NodeEnum::AStar(_) => None,
                _ => figure_column_name_with_strength(indirection.arg.as_deref()?.node.as_ref()?),
            }
        }
        NodeEnum::FuncCall(call) => match call.funcname.last()?.node.as_ref()? {
            NodeEnum::String(name) => named(&name.sval),
            _ => None,
        },
        NodeEnum::AExpr(expr) if expr.kind() == protobuf::AExprKind::AexprNullif => named("nullif"),
        NodeEnum::TypeCast(cast) => {
            let inner = figure_column_name_with_strength(cast.arg.as_deref()?.node.as_ref()?)?;
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
        NodeEnum::CollateClause(collate) => {
            figure_column_name_with_strength(collate.arg.as_deref()?.node.as_ref()?)
        }
        NodeEnum::GroupingFunc(_) => named("grouping"),
        NodeEnum::SubLink(link) => match link.sub_link_type() {
            protobuf::SubLinkType::ExistsSublink => named("exists"),
            protobuf::SubLinkType::ArraySublink => named("array"),
            protobuf::SubLinkType::ExprSublink | protobuf::SubLinkType::MultiexprSublink => {
                let NodeEnum::SelectStmt(select) = link.subselect.as_deref()?.node.as_ref()? else {
                    return None;
                };
                let NodeEnum::ResTarget(target) = select.target_list.first()?.node.as_ref()? else {
                    return None;
                };
                if target.name.is_empty() {
                    Some((UNNAMED.into(), 0))
                } else {
                    named(&target.name)
                }
            }
            _ => Some((UNNAMED.into(), 0)),
        },
        NodeEnum::CaseExpr(_) => Some(("case".into(), 1)),
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
        NodeEnum::AConst(_)
        | NodeEnum::AExpr(_)
        | NodeEnum::BoolExpr(_)
        | NodeEnum::NullTest(_)
        | NodeEnum::BooleanTest(_)
        | NodeEnum::ParamRef(_) => Some((UNNAMED.into(), 0)),
        _ => None,
    }
}
