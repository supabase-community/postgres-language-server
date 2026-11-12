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
    if is_star(value) {
        return None;
    }
    figure_column_name_with_strength(value).map(|(name, _)| name)
}

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

/// How deeply [`figure_column_name`] follows casts, `COLLATE`, the `ELSE` of `CASE`, field
/// selections and scalar subqueries to the expression that names the column. Postgres follows
/// them recursively, until its stack depth limit; real queries nest a handful. Ten thousand
/// levels take well under a millisecond.
const COLUMN_NAME_DEPTH_MAX: usize = 10_000;

/// How many arms of a chain of set operations, like `select 1 union select 2 union ...`, are
/// followed to the left-most select. Postgres transforms the chain recursively, and fails with
/// its stack depth limit long before a hundred thousand arms.
const SET_OPERATION_ARMS_MAX: usize = 100_000;

/// The column name together with how strong that choice is. A type cast uses the name of its
/// argument only if that name is strong (2), and its type name otherwise. A CASE likewise uses the
/// name of its ELSE result only if it is strong, and `case` otherwise. Port of
/// [`FigureColnameInternal`], which recurses into these; this walks down to the expression that
/// names the column and then applies the casts and `CASE`s on the way back up.
///
/// [`FigureColnameInternal`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L1752
fn figure_column_name_with_strength(value: &NodeEnum) -> Option<(String, u8)> {
    let mut wrappers = Vec::new();
    let mut value = value;
    let named = loop {
        if wrappers.len() >= COLUMN_NAME_DEPTH_MAX {
            tracing::warn!(
                "stopped looking for the name of a column after {COLUMN_NAME_DEPTH_MAX} nested expressions"
            );
            return None;
        }
        match step(value) {
            Step::Named(named) => break named?,
            Step::Into(inner, wrapper) => {
                wrappers.push(wrapper);
                value = inner;
            }
        }
    };
    wrappers
        .iter()
        .rev()
        .try_fold(named, |inner, wrapper| wrapper.apply(inner))
}

/// One step of [`figure_column_name_with_strength`].
enum Step<'a> {
    /// The expression names the column itself, or we can't tell (`None`).
    Named(Option<(String, u8)>),
    /// The name comes from the inner expression, through the wrapper.
    Into(&'a NodeEnum, Wrapper<'a>),
}

/// An expression whose column name comes from an inner expression.
enum Wrapper<'a> {
    /// `COLLATE`, or a subscript of an expression: the inner name.
    Same,
    /// A cast: the inner name if it is strong, and the type name otherwise.
    Cast(&'a protobuf::TypeCast),
    /// A `CASE`: the name of its `ELSE` result if it is strong, and `case` otherwise.
    Case,
    /// A scalar subquery: the name of its first column, which is strong.
    Subquery,
}

impl Wrapper<'_> {
    fn apply(&self, inner: (String, u8)) -> Option<(String, u8)> {
        match self {
            Wrapper::Same => Some(inner),
            Wrapper::Cast(_) | Wrapper::Case if inner.1 > 1 => Some(inner),
            Wrapper::Cast(cast) => match cast.type_name.as_ref()?.names.last()?.node.as_ref()? {
                NodeEnum::String(name) => Some((name.sval.clone(), 1)),
                _ => None,
            },
            Wrapper::Case => Some(("case".into(), 1)),
            Wrapper::Subquery => Some((inner.0, 2)),
        }
    }
}

fn step(value: &NodeEnum) -> Step<'_> {
    fn into<'a>(inner: Option<&'a protobuf::Node>, wrapper: Wrapper<'a>) -> Step<'a> {
        match inner.and_then(|inner| inner.node.as_ref()) {
            Some(inner) => Step::Into(inner, wrapper),
            None => Step::Named(None),
        }
    }
    let named = |name: &str| Step::Named(Some((name.to_owned(), 2)));
    let unnamed = || Step::Named(Some((UNNAMED.into(), 0)));
    match value {
        NodeEnum::ColumnRef(column) => match last_field_name(&column.fields) {
            Some(name) => named(name),
            None => unnamed(),
        },
        NodeEnum::AIndirection(indirection) => match last_field_name(&indirection.indirection) {
            Some(name) => named(name),
            None => into(indirection.arg.as_deref(), Wrapper::Same),
        },
        NodeEnum::FuncCall(call) => {
            match call.funcname.last().and_then(|name| name.node.as_ref()) {
                Some(NodeEnum::String(name)) => named(&name.sval),
                _ => Step::Named(None),
            }
        }
        NodeEnum::AExpr(expr) if expr.kind() == protobuf::AExprKind::AexprNullif => named("nullif"),
        NodeEnum::TypeCast(cast) => into(cast.arg.as_deref(), Wrapper::Cast(cast)),
        NodeEnum::CollateClause(collate) => into(collate.arg.as_deref(), Wrapper::Same),
        NodeEnum::GroupingFunc(_) => named("grouping"),
        NodeEnum::SubLink(link) => match link.sub_link_type() {
            protobuf::SubLinkType::ExistsSublink => named("exists"),
            protobuf::SubLinkType::ArraySublink => named("array"),
            // Postgres transforms the subquery first and takes the name of its column.
            protobuf::SubLinkType::ExprSublink => match link
                .subselect
                .as_deref()
                .and_then(|select| select.node.as_ref())
            {
                Some(NodeEnum::SelectStmt(select)) => first_column(select, Wrapper::Subquery),
                _ => Step::Named(None),
            },
            _ => unnamed(),
        },
        NodeEnum::CaseExpr(case) => match case.defresult.as_deref() {
            Some(default) => into(Some(default), Wrapper::Case),
            None => Step::Named(Some(("case".into(), 1))),
        },
        NodeEnum::AArrayExpr(_) => named("array"),
        NodeEnum::RowExpr(_) => named("row"),
        NodeEnum::CoalesceExpr(_) => named("coalesce"),
        NodeEnum::MinMaxExpr(expr) => match expr.op() {
            protobuf::MinMaxOp::IsGreatest => named("greatest"),
            protobuf::MinMaxOp::IsLeast => named("least"),
            _ => Step::Named(None),
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
                _ => return Step::Named(None),
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
                Op::IsDocument => unnamed(),
                Op::Undefined => Step::Named(None),
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
                Op::JsonTableOp | Op::Undefined => Step::Named(None),
            }
        }
        NodeEnum::AConst(_)
        | NodeEnum::AExpr(_)
        | NodeEnum::BoolExpr(_)
        | NodeEnum::NullTest(_)
        | NodeEnum::BooleanTest(_)
        | NodeEnum::ParamRef(_) => unnamed(),
        _ => Step::Named(None),
    }
}

/// The first output column of a subquery: a set operation takes the names of its left-most
/// select ([`transformSetOperationStmt`]), and VALUES names its columns `column1`, `column2`, ...
/// ([`addRangeTableEntryForValues`]). Its name is strong, and comes through `wrapper` from the
/// expression of the column unless the column has an alias.
///
/// [`transformSetOperationStmt`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/analyze.c#L1751
/// [`addRangeTableEntryForValues`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_relation.c#L2183
fn first_column<'a>(select: &'a protobuf::SelectStmt, wrapper: Wrapper<'a>) -> Step<'a> {
    let Some(select) = leftmost_select(select) else {
        return Step::Named(None);
    };
    if !select.values_lists.is_empty() {
        return Step::Named(Some(("column1".into(), 2)));
    }
    let Some(NodeEnum::ResTarget(target)) = select
        .target_list
        .first()
        .and_then(|target| target.node.as_ref())
    else {
        return Step::Named(None);
    };
    if !target.name.is_empty() {
        return Step::Named(Some((target.name.clone(), 2)));
    }
    match target.val.as_deref().and_then(|value| value.node.as_ref()) {
        Some(value) if !is_star(value) => Step::Into(value, wrapper),
        _ => Step::Named(None),
    }
}

/// The left-most select of a chain of set operations, whose columns name those of the chain, or
/// the select itself. `None` after [`SET_OPERATION_ARMS_MAX`] arms.
pub(crate) fn leftmost_select(select: &protobuf::SelectStmt) -> Option<&protobuf::SelectStmt> {
    let mut select = select;
    for _ in 0..SET_OPERATION_ARMS_MAX {
        if select.op() == protobuf::SetOperation::SetopNone {
            return Some(select);
        }
        select = select.larg.as_deref()?;
    }
    tracing::warn!(
        "stopped looking for the left-most select after {SET_OPERATION_ARMS_MAX} set operations"
    );
    None
}

#[cfg(test)]
mod tests {
    use pgls_query::protobuf::{
        CaseExpr, ColumnRef, Node, ResTarget, SelectStmt, SubLink, SubLinkType, TypeCast, TypeName,
    };

    use super::*;

    /// How deeply the generated expressions nest. Recursing once per level overflowed the stack
    /// below.
    const DEPTH: usize = 5_000;
    const SMALL_STACK_BYTES: usize = 256 * 1024;

    fn node(node: NodeEnum) -> Option<Box<Node>> {
        Some(Box::new(Node { node: Some(node) }))
    }

    fn string(sval: &str) -> Node {
        Node {
            node: Some(NodeEnum::String(protobuf::String { sval: sval.into() })),
        }
    }

    fn constant() -> NodeEnum {
        NodeEnum::AConst(protobuf::AConst::default())
    }

    fn column(name: &str) -> NodeEnum {
        NodeEnum::ColumnRef(ColumnRef {
            fields: vec![string(name)],
            ..Default::default()
        })
    }

    fn cast(arg: NodeEnum, type_name: &str) -> NodeEnum {
        NodeEnum::TypeCast(Box::new(TypeCast {
            arg: node(arg),
            type_name: Some(TypeName {
                names: vec![string(type_name)],
                ..Default::default()
            }),
            ..Default::default()
        }))
    }

    fn case(default: NodeEnum) -> NodeEnum {
        NodeEnum::CaseExpr(Box::new(CaseExpr {
            defresult: node(default),
            ..Default::default()
        }))
    }

    fn subquery(value: NodeEnum) -> NodeEnum {
        NodeEnum::SubLink(Box::new(SubLink {
            sub_link_type: SubLinkType::ExprSublink.into(),
            subselect: node(NodeEnum::SelectStmt(Box::new(SelectStmt {
                target_list: vec![Node {
                    node: Some(NodeEnum::ResTarget(Box::new(ResTarget {
                        val: node(value),
                        ..Default::default()
                    }))),
                }],
                op: protobuf::SetOperation::SetopNone.into(),
                ..Default::default()
            }))),
            ..Default::default()
        }))
    }

    /// Runs `f` on a thread with a small stack, and builds and drops the deep expressions on one
    /// with a large stack, since dropping them recurses.
    fn on_small_stack<T: Sync>(
        build: impl FnOnce() -> T + Send + 'static,
        f: impl Fn(&T) + Send + Sync + 'static,
    ) {
        std::thread::Builder::new()
            .stack_size(256 * 1024 * 1024)
            .spawn(move || {
                let value = build();
                std::thread::scope(|scope| {
                    std::thread::Builder::new()
                        .stack_size(SMALL_STACK_BYTES)
                        .spawn_scoped(scope, || f(&value))
                        .unwrap()
                        .join()
                        .unwrap();
                });
            })
            .unwrap()
            .join()
            .unwrap();
    }

    fn name(value: &NodeEnum) -> Option<String> {
        figure_column_name(value)
    }

    #[test]
    fn casts_case_and_subqueries_pass_on_strong_names() {
        assert_eq!(name(&cast(column("a"), "int8")).as_deref(), Some("a"));
        assert_eq!(
            name(&cast(cast(constant(), "int4"), "text")).as_deref(),
            Some("text")
        );
        assert_eq!(name(&case(column("a"))).as_deref(), Some("a"));
        assert_eq!(
            name(&case(cast(constant(), "int4"))).as_deref(),
            Some("case")
        );
        assert_eq!(
            name(&subquery(cast(constant(), "int4"))).as_deref(),
            Some("int4")
        );
    }

    #[test]
    fn deeply_nested_expressions_dont_overflow_the_stack() {
        on_small_stack(
            || {
                let mut casts = column("a");
                let mut cases = column("b");
                let mut subqueries = column("c");
                for _ in 0..DEPTH {
                    casts = cast(casts, "text");
                    cases = case(cases);
                    subqueries = subquery(subqueries);
                }
                (casts, cases, subqueries)
            },
            |(casts, cases, subqueries)| {
                assert_eq!(name(casts).as_deref(), Some("a"));
                assert_eq!(name(cases).as_deref(), Some("b"));
                assert_eq!(name(subqueries).as_deref(), Some("c"));
            },
        );
    }

    #[test]
    fn stops_after_the_depth_limit() {
        on_small_stack(
            || {
                let mut casts = column("a");
                for _ in 0..COLUMN_NAME_DEPTH_MAX {
                    casts = cast(casts, "text");
                }
                casts
            },
            |casts| assert_eq!(name(casts), None),
        );
    }
}
