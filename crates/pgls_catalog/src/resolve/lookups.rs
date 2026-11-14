//! The names that Postgres looks up through the search path, collected under
//! [`ResolveParams::collect_lookups`](super::ResolveParams::collect_lookups): relations, types,
//! functions and operators without a schema, and column references without a relation.
//!
//! A name is recorded where the resolver visits it, whether or not its object exists, so the
//! lookups don't need a database snapshot. Only two decisions use the objects: an unqualified
//! relation that is a temporary table isn't recorded, since `pg_temp` always comes first, and an
//! unqualified name in a query is a column unless it is a whole row, a parameter, or an output
//! column. The parts of a statement that the resolver doesn't resolve, like the expressions of
//! DDL, are walked by [`record_unresolved`].

use pgls_query::{
    NodeEnum, NodeRef,
    protobuf::{
        AExpr, AExprKind, AlterTableType, CoercionForm, CreateFunctionStmt, FuncCall, Node,
        RangeVar, SortBy, SubLink, SubLinkType, Token, TypeName,
    },
};
use pgls_query_ext::utils::string_value;
use pgls_text_size::TextRange;
use rustc_hash::FxHashSet;

use super::{nodes::record_column_ref_lookup, resolver::Resolver};
use crate::typing::serial_type;

/// A name that Postgres looks up through the search path, because it is not qualified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchPathLookup {
    pub kind: LookupKind,
    /// The name as Postgres reads it, e.g. `users` for `Users`, or the operator, e.g. `=`. For an
    /// operator construct, the first of [`OperatorConstruct::operators`].
    pub name: String,
    /// The location of the name in the statement, or of the first keyword of an operator
    /// construct.
    pub location: i32,
    /// The span of the name, or of the keywords of an operator construct, like `NOT IN`.
    pub span: Option<TextRange>,
}

/// What a [`SearchPathLookup`] looks up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LookupKind {
    /// A relation that a query, DML, `COPY`, `TRUNCATE` or `LOCK` reads or writes. Not the
    /// relations that DDL creates, changes or drops, nor CTEs and temporary tables.
    Relation,
    /// A column reference without a relation in a query that has relations. Postgres looks it up
    /// in every relation of the query and of the queries around it, and a parameter of the SQL
    /// function the statement is in takes its place if none has it.
    Column,
    /// A type name, except the serial pseudo-types of column definitions and `%TYPE`.
    Type,
    /// A function call, except the SQL syntax that calls a function of `pg_catalog`, like
    /// `EXTRACT`.
    Function,
    /// An operator: written out, like `=`, `OPERATOR(=)`, `= ANY`, and `USING <`, or used by a
    /// construct.
    Operator(Option<OperatorConstruct>),
}

/// A SQL construct that Postgres turns into operators, which it looks up through the search path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperatorConstruct {
    In,
    NotIn,
    Like,
    NotLike,
    Ilike,
    NotIlike,
    SimilarTo,
    NotSimilarTo,
    IsDistinctFrom,
    IsNotDistinctFrom,
    Nullif,
    Between,
    NotBetween,
}

impl OperatorConstruct {
    /// The keywords of the construct, e.g. `NOT IN`.
    pub fn keywords(self) -> &'static str {
        match self {
            Self::In => "IN",
            Self::NotIn => "NOT IN",
            Self::Like => "LIKE",
            Self::NotLike => "NOT LIKE",
            Self::Ilike => "ILIKE",
            Self::NotIlike => "NOT ILIKE",
            Self::SimilarTo => "SIMILAR TO",
            Self::NotSimilarTo => "NOT SIMILAR TO",
            Self::IsDistinctFrom => "IS DISTINCT FROM",
            Self::IsNotDistinctFrom => "IS NOT DISTINCT FROM",
            Self::Nullif => "NULLIF",
            Self::Between => "BETWEEN",
            Self::NotBetween => "NOT BETWEEN",
        }
    }

    /// The operators that the construct looks up, e.g. `>=` and `<=` for `BETWEEN`.
    pub fn operators(self) -> &'static [&'static str] {
        match self {
            Self::In | Self::IsDistinctFrom | Self::IsNotDistinctFrom | Self::Nullif => &["="],
            Self::NotIn => &["<>"],
            Self::Like => &["~~"],
            Self::NotLike => &["!~~"],
            Self::Ilike => &["~~*"],
            Self::NotIlike => &["!~~*"],
            Self::SimilarTo => &["~"],
            Self::NotSimilarTo => &["!~"],
            Self::Between => &[">=", "<="],
            Self::NotBetween => &["<", ">"],
        }
    }

    /// The last keyword of the construct, e.g. `FROM` of `IS DISTINCT FROM`.
    fn last_token(self) -> Token {
        match self {
            Self::In | Self::NotIn => Token::InP,
            Self::Like | Self::NotLike => Token::Like,
            Self::Ilike | Self::NotIlike => Token::Ilike,
            Self::SimilarTo | Self::NotSimilarTo => Token::To,
            Self::IsDistinctFrom | Self::IsNotDistinctFrom => Token::From,
            Self::Nullif => Token::Nullif,
            Self::Between | Self::NotBetween => Token::Between,
        }
    }
}

/// Records the relation `n` if it is unqualified. Called where Postgres looks it up, with the
/// relation it finds, so that a temporary table isn't recorded. [`RangeVarGetRelidExtended`]
/// looks in `pg_temp` first, so a name that is a temporary table can't refer to anything else.
///
/// [`RangeVarGetRelidExtended`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/catalog/namespace.c#L441
pub(super) fn record_relation(r: &mut Resolver, n: &RangeVar, temporary: bool) {
    if !r.collects_lookups() || !n.schemaname.is_empty() || !n.catalogname.is_empty() || temporary {
        return;
    }
    let span = r.token_span(n.location);
    r.record_lookup(LookupKind::Relation, &n.relname, n.location, span);
}

/// Records the function of `call` if it is unqualified. SQL syntax like `EXTRACT` or `TRIM` calls
/// a function of `pg_catalog` ([`transformFuncCall`]).
///
/// [`transformFuncCall`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_expr.c#L1449
pub(super) fn record_function(r: &mut Resolver, call: &FuncCall) {
    if !r.collects_lookups() || call.funcformat() == CoercionForm::CoerceSqlSyntax {
        return;
    }
    let Some(name) = unqualified(&call.funcname) else {
        return;
    };
    let span = r.token_span(call.location);
    r.record_lookup(LookupKind::Function, name, call.location, span);
}

/// Records the type name `n` if it is unqualified ([`LookupTypeNameExtended`]). The types whose
/// names are keywords, like `integer`, are `pg_catalog` types in the parse tree. `%TYPE` looks up
/// a column.
///
/// [`LookupTypeNameExtended`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_type.c#L73
pub(super) fn record_type(r: &mut Resolver, n: &TypeName) {
    if !r.collects_lookups() || n.pct_type {
        return;
    }
    let Some(name) = unqualified(&n.names) else {
        return;
    };
    let span = r.token_span(n.location);
    r.record_lookup(LookupKind::Type, name, n.location, span);
}

/// Records the operator that an `A_Expr` looks up, by its kind ([`transformExprRecurse`]).
///
/// [`transformExprRecurse`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_expr.c#L137
pub(super) fn record_a_expr(r: &mut Resolver, n: &AExpr) {
    if !r.collects_lookups() {
        return;
    }
    let Some(name) = unqualified(&n.name) else {
        return;
    };
    // The negated constructs, like `NOT IN`, are named by the negated operator, like `<>`.
    let negated = matches!(name, "<>" | "!~~" | "!~~*" | "!~");
    let construct = match n.kind() {
        AExprKind::AexprOp | AExprKind::AexprOpAny | AExprKind::AexprOpAll => None,
        // `IS [NOT] DISTINCT FROM NULL` with a bare `NULL` is a `NullTest`, which uses no
        // operator ([`transformAExprDistinct`]).
        //
        // [`transformAExprDistinct`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_expr.c#L1031
        AExprKind::AexprDistinct | AExprKind::AexprNotDistinct
            if [&n.lexpr, &n.rexpr].into_iter().any(|operand| {
                matches!(
                    operand.as_deref().and_then(|operand| operand.node.as_ref()),
                    Some(NodeEnum::AConst(value)) if value.isnull
                )
            }) =>
        {
            return;
        }
        AExprKind::AexprDistinct => Some(OperatorConstruct::IsDistinctFrom),
        AExprKind::AexprNotDistinct => Some(OperatorConstruct::IsNotDistinctFrom),
        // [`transformAExprIn`] compares with `=`, or `<>` for `NOT IN`.
        //
        // [`transformAExprIn`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_expr.c#L1125
        AExprKind::AexprIn if negated => Some(OperatorConstruct::NotIn),
        AExprKind::AexprIn => Some(OperatorConstruct::In),
        // `LIKE`, `ILIKE` and `SIMILAR TO` are operators named by the parser, which
        // [`transformAExprOp`] looks up like any other.
        //
        // [`transformAExprOp`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_expr.c#L922
        AExprKind::AexprLike if negated => Some(OperatorConstruct::NotLike),
        AExprKind::AexprLike => Some(OperatorConstruct::Like),
        AExprKind::AexprIlike if negated => Some(OperatorConstruct::NotIlike),
        AExprKind::AexprIlike => Some(OperatorConstruct::Ilike),
        AExprKind::AexprSimilar if negated => Some(OperatorConstruct::NotSimilarTo),
        AExprKind::AexprSimilar => Some(OperatorConstruct::SimilarTo),
        // [`transformAExprNullIf`]
        //
        // [`transformAExprNullIf`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_expr.c#L1082
        AExprKind::AexprNullif => Some(OperatorConstruct::Nullif),
        // [`transformAExprBetween`] compares with `>=` and `<=`, or `<` and `>` for
        // `NOT BETWEEN`, whatever the name of the `A_Expr`.
        //
        // [`transformAExprBetween`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_expr.c#L1294
        AExprKind::AexprBetween | AExprKind::AexprBetweenSym => Some(OperatorConstruct::Between),
        AExprKind::AexprNotBetween | AExprKind::AexprNotBetweenSym => {
            Some(OperatorConstruct::NotBetween)
        }
        AExprKind::Undefined => return,
    };
    record_operator(r, name, construct, n.location);
}

/// Records the operator of `op ANY (subquery)`, `op ALL (subquery)`, and `IN (subquery)`, which
/// compares with `=` ([`transformSubLink`]).
///
/// [`transformSubLink`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_expr.c#L1782
pub(super) fn record_sub_link(r: &mut Resolver, n: &SubLink) {
    if !r.collects_lookups() {
        return;
    }
    match n.sub_link_type() {
        SubLinkType::AnySublink if n.oper_name.is_empty() => {
            record_operator(r, "=", Some(OperatorConstruct::In), n.location);
        }
        SubLinkType::AnySublink | SubLinkType::AllSublink => {
            if let Some(name) = unqualified(&n.oper_name) {
                record_operator(r, name, None, n.location);
            }
        }
        _ => {}
    }
}

/// Records the operator of `ORDER BY ... USING op` ([`addTargetToSortList`]).
///
/// [`addTargetToSortList`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_clause.c#L3393
pub(super) fn record_sort_by(r: &mut Resolver, n: &SortBy) {
    if !r.collects_lookups() {
        return;
    }
    if let Some(name) = unqualified(&n.use_op) {
        record_operator(r, name, None, n.location);
    }
}

fn record_operator(
    r: &mut Resolver,
    name: &str,
    construct: Option<OperatorConstruct>,
    location: i32,
) {
    let span = match construct {
        Some(construct) => r.construct_span(location, construct.last_token()),
        None => r.token_span(location),
    };
    let name = construct.map_or(name, |construct| construct.operators()[0]);
    r.record_lookup(LookupKind::Operator(construct), name, location, span);
}

/// Records the lookups in `node`, a part of the statement that the resolver doesn't resolve,
/// like a constraint, an index expression or the arguments of `CALL`: its functions, types and
/// operators, and, if it is part of a query (`in_query`), its relations and columns, as seen from
/// the query.
pub(super) fn record_unresolved(r: &mut Resolver, node: NodeRef, in_query: bool) {
    if !r.collects_lookups() {
        return;
    }
    // The serial columns of `CREATE TABLE` and `ALTER TABLE ... ADD COLUMN`, by the address of
    // their type, which Postgres doesn't look up ([`transformColumnDefinition`]). A node is
    // visited after the statement or command it is in.
    //
    // [`transformColumnDefinition`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_utilcmd.c#L591
    let mut serial_columns: FxHashSet<*const TypeName> = FxHashSet::default();
    for node in node.iter() {
        match node {
            NodeRef::CreateStmt(n) => serial_columns.extend(serial_column_types(&n.table_elts)),
            NodeRef::AlterTableCmd(n) if n.subtype() == AlterTableType::AtAddColumn => {
                serial_columns.extend(serial_column_types(n.def.as_deref()));
            }
            NodeRef::TypeName(n) if !serial_columns.contains(&std::ptr::from_ref(n)) => {
                record_type(r, n);
            }
            NodeRef::FuncCall(n) => record_function(r, n),
            NodeRef::AExpr(n) => record_a_expr(r, n),
            NodeRef::SubLink(n) => record_sub_link(r, n),
            NodeRef::SortBy(n) => record_sort_by(r, n),
            NodeRef::RangeVar(n)
                if in_query
                    && !r
                        .ctes
                        .iter()
                        .any(|cte| n.schemaname.is_empty() && cte.name == n.relname) =>
            {
                record_relation(r, n, false);
            }
            NodeRef::ColumnRef(n) if in_query => record_column_ref_lookup(r, n),
            _ => {}
        }
    }
}

/// Records the lookups of `CREATE FUNCTION` outside its body: of its parameters, their defaults,
/// its result and its options. The statements of a body written as `BEGIN ATOMIC ... END` or
/// `RETURN ...` are resolved on their own, and a body in a string is not parsed here.
pub(super) fn record_create_function(r: &mut Resolver, n: &CreateFunctionStmt) {
    let parts = n
        .parameters
        .iter()
        .chain(&n.options)
        .filter_map(|part| part.node.as_ref());
    for part in parts {
        record_unresolved(r, part.to_ref(), false);
    }
    if let Some(return_type) = &n.return_type {
        record_unresolved(r, return_type.to_ref(), false);
    }
}

/// The types of the serial columns among `column_defs`, by address.
fn serial_column_types<'a>(
    column_defs: impl IntoIterator<Item = &'a Node>,
) -> impl Iterator<Item = *const TypeName> {
    column_defs
        .into_iter()
        .filter_map(|column_def| match &column_def.node {
            Some(NodeEnum::ColumnDef(column_def)) => column_def.type_name.as_ref(),
            _ => None,
        })
        .filter(|type_name| serial_type(type_name).is_some())
        .map(std::ptr::from_ref)
}

/// The name of a name list without schema, like the name of an unqualified function.
fn unqualified(names: &[Node]) -> Option<&str> {
    match names {
        [name] => string_value(name),
        _ => None,
    }
}
