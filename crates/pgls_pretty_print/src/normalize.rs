//! AST normalization for semantic comparison.
//!
//! This module provides functions to normalize ASTs so that semantically equivalent
//! but syntactically different forms compare as equal. This is used during beta
//! to verify that formatting preserves semantics.
//!
//! The normalization functions handle cases like:
//! - Clearing location fields (which differ between original and reparsed ASTs)
//! - Flattening nested structures that can be represented multiple ways
//! - Normalizing equivalent forms (e.g., CROSS JOIN vs INNER JOIN ON TRUE)
//! - Converting internal representations to their emitted forms

use pgls_query::{NodeEnum, NodeMut};

use crate::nodes::clauses::function_options::function_option_order;

/// Normalize an AST for semantic comparison.
///
/// Applies all normalization functions to handle differences between
/// the original AST and the AST of reparsed formatted output.
pub fn normalize_ast(node: &mut NodeEnum) {
    clear_location(node);
    normalize_bool_expr_associativity(node);
    normalize_foreign_table_partbound(node);
    normalize_merge_support_func(node);
    normalize_function_options(node);
}

/// Flatten nested boolean expressions which PostgreSQL represents differently
/// depending on redundant parentheses.
///
/// Only same-operator AND and OR expressions are associative. NOT and mixed
/// AND/OR expressions retain their original tree shape because parentheses
/// affect their semantics.
fn normalize_bool_expr_associativity(node: &mut NodeEnum) {
    let bool_exprs = node
        .iter_mut()
        .filter_map(|node| match node {
            NodeMut::BoolExpr(bool_expr) => Some(bool_expr),
            _ => None,
        })
        .collect::<Vec<_>>();

    // Process descendants first. Flattening a parent moves its child nodes, so
    // child pointers must not be used after their parent has been normalized.
    for bool_expr in bool_exprs.into_iter().rev() {
        unsafe {
            flatten_bool_expr_args(&mut *bool_expr);
        }
    }
}

fn flatten_bool_expr_args(bool_expr: &mut pgls_query::protobuf::BoolExpr) {
    use pgls_query::protobuf::BoolExprType;

    if !matches!(
        BoolExprType::try_from(bool_expr.boolop),
        Ok(BoolExprType::AndExpr | BoolExprType::OrExpr)
    ) {
        return;
    }

    let boolop = bool_expr.boolop;
    let mut flattened = Vec::with_capacity(bool_expr.args.len());

    for mut arg in std::mem::take(&mut bool_expr.args) {
        let nested_args = match arg.node.as_mut() {
            Some(NodeEnum::BoolExpr(nested)) if nested.boolop == boolop => {
                Some(std::mem::take(&mut nested.args))
            }
            _ => None,
        };

        if let Some(mut nested_args) = nested_args {
            flattened.append(&mut nested_args);
        } else {
            flattened.push(arg);
        }
    }

    bool_expr.args = flattened;
}

/// Clear location fields in AST nodes.
///
/// Location fields record the byte offset in the original source,
/// which will differ between the original and reparsed AST.
fn clear_location(node: &mut NodeEnum) {
    let mut join_quals_to_clear = Vec::new();

    // SAFETY: The iterator provides mutable access to AST node fields
    unsafe {
        node.iter_mut().for_each(|n| match n {
            NodeMut::ColumnRef(n) => {
                (*n).location = 0;
            }
            NodeMut::WithClause(n) => {
                (*n).location = 0;
            }
            NodeMut::ParamRef(n) => {
                (*n).location = 0;
            }
            NodeMut::AExpr(n) => {
                (*n).location = 0;
            }
            NodeMut::JoinExpr(n) => {
                (*n).rtindex = 0;
                // Lives in the generic pass so that no parent node kind can hide a join from it:
                // the printer spells a CROSS JOIN as INNER JOIN ON TRUE, and both must compare equal.
                if (*n).jointype == pgls_query::protobuf::JoinType::JoinInner as i32 {
                    let is_true_qual = (*n)
                        .quals
                        .as_ref()
                        .and_then(|q| q.node.as_ref())
                        .map(|node| {
                            matches!(
                                node,
                                NodeEnum::AConst(c)
                                    if matches!(c.val.as_ref(), Some(pgls_query::protobuf::a_const::Val::Boolval(b)) if b.boolval)
                            )
                        })
                        .unwrap_or(false);
                    if is_true_qual {
                        // Defer structural mutations until the iterator has finished. It keeps
                        // pointers to descendants, which clearing `quals` would invalidate.
                        join_quals_to_clear.push(n);
                    }
                }
            }
            NodeMut::TypeCast(n) => {
                (*n).location = 0;
            }
            NodeMut::CollateClause(n) => {
                (*n).location = 0;
            }
            NodeMut::FuncCall(n) => {
                (*n).location = 0;
                // SQL-standard syntax the printer does not know is written as a call to the same
                // `pg_catalog` function, which only differs in this flag.
                (*n).funcformat = pgls_query::protobuf::CoercionForm::CoerceExplicitCall.into();
            }
            NodeMut::NamedArgExpr(n) => {
                (*n).location = 0;
            }
            NodeMut::SetToDefault(n) => {
                (*n).location = 0;
            }
            NodeMut::AArrayExpr(n) => {
                (*n).location = 0;
            }
            NodeMut::ResTarget(n) => {
                (*n).location = 0;
            }
            NodeMut::SortBy(n) => {
                (*n).location = 0;
            }
            NodeMut::CoalesceExpr(n) => {
                (*n).location = 0;
            }
            NodeMut::WindowDef(n) => {
                (*n).location = 0;
            }
            NodeMut::PartitionSpec(n) => {
                (*n).location = 0;
            }
            NodeMut::PartitionElem(n) => {
                (*n).location = 0;
            }
            NodeMut::PartitionBoundSpec(n) => {
                (*n).location = 0;
            }
            NodeMut::SqlvalueFunction(n) => {
                (*n).location = 0;
            }
            NodeMut::ColumnDef(n) => {
                (*n).location = 0;
            }
            NodeMut::DefElem(n) => {
                (*n).location = 0;
                // Normalize defname to lowercase for case-insensitive options
                (*n).defname = (*n).defname.to_lowercase();
            }
            NodeMut::DeclareCursorStmt(n) => {
                // Mask out internal optimization flags (CURSOR_OPT_PARALLEL_OK = 0x100 etc.)
                // Keep only syntax-affecting flags: BINARY, SCROLL, NO_SCROLL, INSENSITIVE, ASENSITIVE, HOLD
                const CURSOR_SYNTAX_MASK: i32 = 0x3F; // First 6 bits
                (*n).options &= CURSOR_SYNTAX_MASK;
            }
            NodeMut::XmlSerialize(n) => {
                (*n).location = 0;
            }
            NodeMut::JsonArrayConstructor(n) => {
                (*n).location = 0;
                if let Some(output) = (*n).output.as_mut()
                    && let Some(returning) = output.returning.as_mut()
                    && let Some(format) = returning.format.as_mut()
                {
                    format.location = 0;
                }
            }
            NodeMut::JsonObjectConstructor(n) => {
                (*n).location = 0;
                if let Some(output) = (*n).output.as_mut()
                    && let Some(returning) = output.returning.as_mut()
                    && let Some(format) = returning.format.as_mut()
                {
                    format.location = 0;
                }
            }
            NodeMut::JsonAggConstructor(n) => {
                (*n).location = 0;
            }
            NodeMut::JsonArrayQueryConstructor(n) => {
                (*n).location = 0;
                if let Some(format) = (*n).format.as_mut() {
                    format.location = 0;
                }
            }
            NodeMut::JsonIsPredicate(n) => {
                (*n).location = 0;
                if let Some(format) = (*n).format.as_mut() {
                    format.location = 0;
                }
            }
            NodeMut::JsonFuncExpr(n) => {
                (*n).location = 0;
                // Normalize locations for behaviors
                if let Some(on_error) = (*n).on_error.as_mut() {
                    on_error.location = 0;
                }
                if let Some(on_empty) = (*n).on_empty.as_mut() {
                    on_empty.location = 0;
                }
                // Normalize output.returning.format.location
                if let Some(output) = (*n).output.as_mut()
                    && let Some(returning) = output.returning.as_mut()
                    && let Some(format) = returning.format.as_mut()
                {
                    format.location = 0;
                }
                // Normalize wrapper: JswUnspec (1) and JswNone (2) are semantically equivalent
                // When no wrapper clause is specified, PostgreSQL defaults to WITHOUT WRAPPER
                if (*n).wrapper == 1 {
                    (*n).wrapper = 2;
                }
            }
            NodeMut::JsonTable(n) => {
                (*n).location = 0;
                if let Some(context) = (*n).context_item.as_mut()
                    && let Some(format) = context.format.as_mut()
                {
                    format.location = 0;
                }

                for column in &mut (*n).columns {
                    if let Some(NodeEnum::JsonTableColumn(col)) = column.node.as_mut() {
                        col.location = 0;
                        if let Some(format) = col.format.as_mut() {
                            format.location = 0;
                        }
                    }
                }
            }
            NodeMut::JsonTableColumn(n) => {
                (*n).location = 0;
                if let Some(format) = (*n).format.as_mut() {
                    format.location = 0;
                }
            }
            NodeMut::JsonTablePathSpec(n) => {
                (*n).location = 0;
                (*n).name_location = 0;
            }
            NodeMut::JsonValueExpr(n) => {
                if let Some(format) = (*n).format.as_mut() {
                    format.location = 0;
                }
            }
            NodeMut::OnConflictClause(n) => {
                (*n).location = 0;
                if let Some(infer) = (*n).infer.as_mut() {
                    infer.location = 0;
                }
            }
            NodeMut::InferClause(n) => {
                (*n).location = 0;
            }
            NodeMut::TypeName(n) => {
                (*n).location = 0;
            }
            NodeMut::JsonBehavior(n) => {
                (*n).location = 0;
            }
            NodeMut::AConst(n) => {
                (*n).location = 0;
                // NOTE: We do NOT lowercase string values here - they are case-sensitive!
                // Only specific contexts (like SET statements) are case-insensitive,
                // and those should be handled in context-specific normalization.
            }
            NodeMut::RangeVar(n) => {
                (*n).location = 0;
            }
            NodeMut::RoleSpec(n) => {
                (*n).location = 0;
            }
            NodeMut::RangeTableFunc(n) => {
                (*n).location = 0;
            }
            NodeMut::RangeTableFuncCol(n) => {
                (*n).location = 0;
            }
            NodeMut::RangeTableSample(n) => {
                (*n).location = 0;
            }
            NodeMut::RowExpr(n) => {
                (*n).location = 0;
            }
            NodeMut::BoolExpr(n) => {
                (*n).location = 0;
            }
            NodeMut::GroupingFunc(n) => {
                (*n).location = 0;
            }
            NodeMut::GroupingSet(n) => {
                (*n).location = 0;
            }
            NodeMut::CommonTableExpr(n) => {
                (*n).location = 0;
            }
            NodeMut::SubLink(n) => {
                (*n).location = 0;
            }
            NodeMut::NullTest(n) => {
                (*n).location = 0;
            }
            NodeMut::BooleanTest(n) => {
                (*n).location = 0;
            }
            NodeMut::MinMaxExpr(n) => {
                (*n).location = 0;
            }
            NodeMut::Constraint(n) => {
                (*n).location = 0;
                // Normalize pg_default indexspace to empty (default tablespace)
                if (*n).indexspace == "pg_default" {
                    (*n).indexspace = String::new();
                }
            }
            NodeMut::CaseWhen(n) => {
                (*n).location = 0;
            }
            NodeMut::CaseExpr(n) => {
                (*n).location = 0;
            }
            NodeMut::TransactionStmt(n) => {
                (*n).location = 0;
            }
            NodeMut::JsonParseExpr(n) => {
                (*n).location = 0;
            }
            NodeMut::JsonScalarExpr(n) => {
                (*n).location = 0;
            }
            NodeMut::JsonSerializeExpr(n) => {
                (*n).location = 0;
            }
            NodeMut::CopyStmt(n) => {
                // Normalize boolean options in COPY statement options
                for opt in (*n).options.iter_mut() {
                    if let Some(NodeEnum::DefElem(def)) = opt.node.as_mut() {
                        def.location = 0;
                        // Normalize boolean true to None for COPY options
                        let should_clear = def
                            .arg
                            .as_ref()
                            .and_then(|arg| arg.node.as_ref())
                            .map(|node| matches!(node, NodeEnum::Boolean(b) if b.boolval))
                            .unwrap_or(false);
                        if should_clear {
                            def.arg = None;
                        }
                    }
                }
            }
            NodeMut::DefineStmt(n) => {
                // Normalize the Integer flag in args for ordered-set aggregates
                // The integer indicates the number of direct arguments, but this may differ
                // between syntactic forms (positive value vs -1)
                // Normalize all to 0 to ignore this difference
                for arg in (*n).args.iter_mut() {
                    if let Some(NodeEnum::Integer(int)) = arg.node.as_mut() {
                        int.ival = 0;
                    }
                }
            }
            NodeMut::IndexElem(n) => {
                // Normalize DefElem args in opclassopts from TypeName to String
                for opt in (*n).opclassopts.iter_mut() {
                    if let Some(NodeEnum::DefElem(def)) = opt.node.as_mut() {
                        def.location = 0;
                        // Normalize TypeName to String
                        if let Some(ref mut arg) = def.arg
                            && let Some(NodeEnum::TypeName(tn)) = arg.node.as_mut()
                            && tn.names.len() == 1
                            && tn.typmods.is_empty()
                            && tn.array_bounds.is_empty()
                            && !tn.setof
                            && !tn.pct_type
                            && let Some(first) = tn.names.first()
                        {
                            arg.node = first.node.clone();
                        }
                    }
                }
            }
            NodeMut::DeallocateStmt(n) => {
                (*n).location = 0;
            }
            NodeMut::PublicationObjSpec(n) => {
                (*n).location = 0;
                // Normalize TABLES IN SCHEMA CURRENT_SCHEMA to TABLES IN CURRENT_SCHEMA
                // The latter parses as PublicationobjTablesInCurSchema (3) with empty name,
                // while the former parses as PublicationobjTablesInSchema (2) with name "CURRENT_SCHEMA"
                if (*n).pubobjtype == 2 && (*n).name.eq_ignore_ascii_case("CURRENT_SCHEMA") {
                    (*n).pubobjtype = 3;
                    (*n).name.clear();
                }
            }
            NodeMut::XmlExpr(n) => {
                (*n).location = 0;
            }
            _ => {}
        });

        for join in join_quals_to_clear {
            (*join).quals = None;
        }
    }
}

/// Normalize CreateForeignTableStmt partbound.
///
/// When partbound is None but inh_relations is non-empty, set a default partition bound.
/// Also clear table_elts for partitions since they inherit from parent.
fn normalize_foreign_table_partbound(node: &mut NodeEnum) {
    if let NodeEnum::CreateForeignTableStmt(stmt) = node
        && let Some(ref mut base) = stmt.base_stmt
    {
        // If we have partition inheritance
        if !base.inh_relations.is_empty() {
            // Add a default partbound if missing
            if base.partbound.is_none() {
                base.partbound = Some(pgls_query::protobuf::PartitionBoundSpec {
                    strategy: String::new(),
                    is_default: true,
                    modulus: 0,
                    remainder: 0,
                    listdatums: vec![],
                    lowerdatums: vec![],
                    upperdatums: vec![],
                    location: 0,
                });
            }
            // Clear table_elts since partition columns come from parent
            base.table_elts.clear();
        }
    }
}

/// Normalize MergeSupportFunc nodes.
///
/// MergeSupportFunc is an internal executor node that gets emitted as `mergesupport#<oid>`.
/// When re-parsed, this becomes a ColumnRef. To match, we need to convert MergeSupportFunc
/// in the original AST to the equivalent ColumnRef.
fn normalize_merge_support_func(node: &mut NodeEnum) {
    normalize_merge_support_func_recursive(node);
}

fn normalize_merge_support_func_recursive(node: &mut NodeEnum) {
    // First check if this node itself is a MergeSupportFunc and replace it
    if let NodeEnum::MergeSupportFunc(msf) = node {
        let ident = format!("mergesupport#{}", msf.msftype);
        *node = NodeEnum::ColumnRef(pgls_query::protobuf::ColumnRef {
            fields: vec![pgls_query::protobuf::Node {
                node: Some(NodeEnum::String(pgls_query::protobuf::String {
                    sval: ident,
                })),
            }],
            location: 0,
        });
        return;
    }

    // Manually handle types that can contain MergeSupportFunc
    match node {
        NodeEnum::SelectStmt(stmt) => {
            process_node_list(&mut stmt.target_list);
            process_node_list(&mut stmt.from_clause);
            process_optional_boxed_node(&mut stmt.where_clause);
            if let Some(ref mut wc) = stmt.with_clause {
                wc.location = 0;
                process_node_list(&mut wc.ctes);
            }
            // Handle set operations (UNION/INTERSECT/EXCEPT)
            if let Some(ref mut larg) = stmt.larg {
                process_select_stmt_box(larg.as_mut());
            }
            if let Some(ref mut rarg) = stmt.rarg {
                process_select_stmt_box(rarg.as_mut());
            }
        }
        NodeEnum::ResTarget(rt) => {
            process_optional_boxed_node(&mut rt.val);
        }
        NodeEnum::CommonTableExpr(cte) => {
            process_optional_boxed_node(&mut cte.ctequery);
        }
        NodeEnum::MergeStmt(stmt) => {
            process_node_list(&mut stmt.returning_list);
            process_node_list(&mut stmt.merge_when_clauses);
        }
        NodeEnum::MergeWhenClause(mwc) => {
            process_node_list(&mut mwc.target_list);
            process_node_list(&mut mwc.values);
            process_optional_boxed_node(&mut mwc.condition);
        }
        NodeEnum::UpdateStmt(stmt) => {
            process_node_list(&mut stmt.target_list);
            process_node_list(&mut stmt.returning_list);
        }
        NodeEnum::InsertStmt(stmt) => {
            process_node_list(&mut stmt.returning_list);
            process_optional_boxed_node(&mut stmt.select_stmt);
            if let Some(ref mut wc) = stmt.with_clause {
                process_node_list(&mut wc.ctes);
            }
        }
        NodeEnum::DeleteStmt(stmt) => {
            process_node_list(&mut stmt.returning_list);
        }
        NodeEnum::CaseExpr(ce) => {
            process_optional_boxed_node(&mut ce.arg);
            process_node_list(&mut ce.args);
            process_optional_boxed_node(&mut ce.defresult);
        }
        NodeEnum::CaseWhen(cw) => {
            process_optional_boxed_node(&mut cw.expr);
            process_optional_boxed_node(&mut cw.result);
        }
        NodeEnum::FuncCall(fc) => {
            process_node_list(&mut fc.args);
        }
        NodeEnum::AExpr(expr) => {
            process_optional_boxed_node(&mut expr.lexpr);
            process_optional_boxed_node(&mut expr.rexpr);
        }
        NodeEnum::SubLink(sl) => {
            process_optional_boxed_node(&mut sl.subselect);
            process_optional_boxed_node(&mut sl.testexpr);
        }
        NodeEnum::RangeSubselect(rs) => {
            process_optional_boxed_node(&mut rs.subquery);
        }
        NodeEnum::JoinExpr(je) => {
            process_optional_boxed_node(&mut je.larg);
            process_optional_boxed_node(&mut je.rarg);
        }
        NodeEnum::ViewStmt(vs) => {
            process_optional_boxed_node(&mut vs.query);
        }
        NodeEnum::WithClause(wc) => {
            wc.location = 0;
            process_node_list(&mut wc.ctes);
        }
        NodeEnum::CopyStmt(cs) => {
            if let Some(query) = &mut cs.query
                && let Some(n) = &mut query.node
            {
                normalize_merge_support_func_recursive(n);
            }
        }
        _ => {}
    }
}

fn process_node_list(list: &mut [pgls_query::protobuf::Node]) {
    for node in list.iter_mut() {
        if let Some(ref mut n) = node.node {
            normalize_merge_support_func_recursive(n);
        }
    }
}

fn process_optional_boxed_node(opt: &mut Option<Box<pgls_query::protobuf::Node>>) {
    if let Some(node) = opt
        && let Some(n) = &mut node.node
    {
        normalize_merge_support_func_recursive(n);
    }
}

fn process_select_stmt_box(stmt: &mut pgls_query::protobuf::SelectStmt) {
    // Recursively process the SelectStmt
    let inner = std::mem::take(stmt);
    let mut node_enum = NodeEnum::SelectStmt(Box::new(inner));
    normalize_merge_support_func_recursive(&mut node_enum);
    if let NodeEnum::SelectStmt(updated) = node_enum {
        *stmt = *updated;
    }
}

/// Sort function options according to Postgres's canonical order.
fn sort_function_options(options: &mut [pgls_query::protobuf::Node]) {
    options.sort_by_key(|node| {
        if let Some(NodeEnum::DefElem(def_elem)) = &node.node {
            function_option_order(&def_elem.defname)
        } else {
            100 // Non-DefElem nodes go last
        }
    });
}

/// Sorts function options to the canonical order the printer writes them in.
fn normalize_function_options(node: &mut NodeEnum) {
    match node {
        NodeEnum::CreateFunctionStmt(stmt) => sort_function_options(&mut stmt.options),
        NodeEnum::AlterFunctionStmt(stmt) => sort_function_options(&mut stmt.actions),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_clears_location() {
        let mut node = NodeEnum::ColumnRef(pgls_query::protobuf::ColumnRef {
            fields: vec![],
            location: 42,
        });
        normalize_ast(&mut node);
        if let NodeEnum::ColumnRef(col) = node {
            assert_eq!(col.location, 0);
        } else {
            panic!("Expected ColumnRef");
        }
    }
}
