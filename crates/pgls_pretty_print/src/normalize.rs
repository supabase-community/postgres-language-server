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

/// Normalize an AST for semantic comparison.
///
/// Applies all normalization functions to handle differences between
/// the original AST and the AST of reparsed formatted output.
pub fn normalize_ast(node: &mut NodeEnum) {
    clear_location(node);
    normalize_a_indirection(node);
    normalize_object_with_args(node);
    normalize_join_expr(node);
    normalize_foreign_table_partbound(node);
    normalize_merge_support_func(node);
    normalize_sql_value_function(node);
}

/// Clear location fields in AST nodes.
///
/// Location fields record the byte offset in the original source,
/// which will differ between the original and reparsed AST.
fn clear_location(node: &mut NodeEnum) {
    // SAFETY: The iterator provides mutable access to AST node fields
    unsafe {
        node.iter_mut().for_each(|n| match n {
            NodeMut::ColumnRef(n) => {
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
            }
            NodeMut::TypeCast(n) => {
                (*n).location = 0;
            }
            NodeMut::CollateClause(n) => {
                (*n).location = 0;
            }
            NodeMut::FuncCall(n) => {
                (*n).location = 0;
                // Normalize funcformat to CoerceExplicitCall since we emit as regular function call
                (*n).funcformat = pgls_query::protobuf::CoercionForm::CoerceExplicitCall.into();
                // Remove pg_catalog prefix from function names
                if (*n).funcname.len() == 2 {
                    if let Some(NodeEnum::String(schema)) =
                        (*n).funcname.first().and_then(|node| node.node.as_ref())
                    {
                        if schema.sval.eq_ignore_ascii_case("pg_catalog") {
                            (*n).funcname.remove(0);
                        }
                    }
                }
                // Normalize function names to lowercase for case-insensitive comparison
                for func_name_node in &mut (*n).funcname {
                    if let Some(NodeEnum::String(s)) = func_name_node.node.as_mut() {
                        s.sval = s.sval.to_lowercase();
                    }
                }
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
                if let Some(output) = (*n).output.as_mut() {
                    if let Some(returning) = output.returning.as_mut() {
                        if let Some(format) = returning.format.as_mut() {
                            format.location = 0;
                        }
                    }
                }
            }
            NodeMut::JsonObjectConstructor(n) => {
                (*n).location = 0;
                if let Some(output) = (*n).output.as_mut() {
                    if let Some(returning) = output.returning.as_mut() {
                        if let Some(format) = returning.format.as_mut() {
                            format.location = 0;
                        }
                    }
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
                if let Some(output) = (*n).output.as_mut() {
                    if let Some(returning) = output.returning.as_mut() {
                        if let Some(format) = returning.format.as_mut() {
                            format.location = 0;
                        }
                    }
                }
                // Normalize wrapper: JswUnspec (1) and JswNone (2) are semantically equivalent
                // When no wrapper clause is specified, PostgreSQL defaults to WITHOUT WRAPPER
                if (*n).wrapper == 1 {
                    (*n).wrapper = 2;
                }
            }
            NodeMut::JsonTable(n) => {
                (*n).location = 0;
                if let Some(context) = (*n).context_item.as_mut() {
                    if let Some(format) = context.format.as_mut() {
                        format.location = 0;
                    }
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

                if (*n).names.len() == 2 {
                    if let Some(NodeEnum::String(schema)) =
                        (*n).names.first().and_then(|node| node.node.as_ref())
                    {
                        if schema.sval.eq_ignore_ascii_case("pg_catalog") {
                            (*n).names.remove(0);
                        }
                    }
                }

                // Normalize char to bpchar(1) and bpchar to bpchar(1)
                if (*n).names.len() == 1 {
                    if let Some(NodeEnum::String(type_name)) =
                        (*n).names.first().and_then(|node| node.node.as_ref())
                    {
                        let is_char = type_name.sval.eq_ignore_ascii_case("char");
                        let is_bpchar = type_name.sval.eq_ignore_ascii_case("bpchar");
                        if (is_char || is_bpchar) && (*n).typmods.is_empty() {
                            // char/bpchar without size is char(1) = bpchar(1)
                            (&mut (*n).names)[0] = pgls_query::protobuf::Node {
                                node: Some(NodeEnum::String(pgls_query::protobuf::String {
                                    sval: "bpchar".to_string(),
                                })),
                            };
                            (*n).typmods.push(pgls_query::protobuf::Node {
                                node: Some(NodeEnum::AConst(pgls_query::protobuf::AConst {
                                    isnull: false,
                                    location: 0,
                                    val: Some(pgls_query::protobuf::a_const::Val::Ival(
                                        pgls_query::protobuf::Integer { ival: 1 },
                                    )),
                                })),
                            });
                        }
                    }
                }
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
            NodeMut::FunctionParameter(n) => {
                // Normalize FunctionParameter mode to FuncParamDefault
                // When emitting objargs (TypeName), reparsing creates objfuncargs (FunctionParameter)
                // with potentially different modes (e.g., FuncParamVariadic vs FuncParamDefault)
                (*n).mode = pgls_query::protobuf::FunctionParameterMode::FuncParamDefault as i32;
                // Clear name as DROP FUNCTION can be parsed with or without param names
                (*n).name.clear();
            }
            NodeMut::IndexElem(n) => {
                // Normalize DefElem args in opclassopts from TypeName to String
                for opt in (*n).opclassopts.iter_mut() {
                    if let Some(NodeEnum::DefElem(def)) = opt.node.as_mut() {
                        def.location = 0;
                        // Normalize TypeName to String
                        if let Some(ref mut arg) = def.arg {
                            if let Some(NodeEnum::TypeName(tn)) = arg.node.as_mut() {
                                if tn.names.len() == 1
                                    && tn.typmods.is_empty()
                                    && tn.array_bounds.is_empty()
                                    && !tn.setof
                                    && !tn.pct_type
                                {
                                    if let Some(first) = tn.names.first() {
                                        arg.node = first.node.clone();
                                    }
                                }
                            }
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
            NodeMut::VariableSetStmt(n) => {
                // SET statement values are case-insensitive in PostgreSQL
                // e.g., SET xmloption = DOCUMENT vs SET xmloption = document
                for arg in &mut (*n).args {
                    if let Some(NodeEnum::AConst(aconst)) = arg.node.as_mut() {
                        aconst.location = 0;
                        if let Some(pgls_query::protobuf::a_const::Val::Sval(s)) =
                            aconst.val.as_mut()
                        {
                            s.sval = s.sval.to_lowercase();
                        }
                    }
                }
            }
            _ => {}
        });
    }
}

/// Flatten nested BoolExpr of the same type.
///
/// PostgreSQL's parser flattens `a AND (b AND c)` to `AND(a, b, c)`
/// but our emitter preserves the original nesting. This function normalizes
/// both ASTs to the flattened form for comparison.
fn flatten_bool_expr(be: &mut pgls_query::protobuf::BoolExpr) {
    // First recursively flatten children
    for arg in &mut be.args {
        if let Some(NodeEnum::BoolExpr(ref mut child)) = arg.node {
            flatten_bool_expr(child);
        }
    }

    // Only flatten AND and OR expressions
    let boolop = be.boolop();
    if boolop == pgls_query::protobuf::BoolExprType::AndExpr
        || boolop == pgls_query::protobuf::BoolExprType::OrExpr
    {
        let mut new_args = Vec::new();
        for arg in std::mem::take(&mut be.args) {
            if let Some(NodeEnum::BoolExpr(ref child)) = arg.node {
                if child.boolop() == boolop {
                    // Same boolop type - flatten by pulling up child args
                    new_args.extend(child.args.clone());
                    continue;
                }
            }
            new_args.push(arg);
        }
        be.args = new_args;
    }
}

/// Normalize AIndirection nodes by flattening nested AIndirection into a single node.
///
/// This handles the case where `(col[0])[0]` parses to a flat structure but `col[0][0]`
/// parses to a nested structure - they're semantically equivalent.
/// Also converts AIndirection(ColumnRef, [String]) to ColumnRef with merged fields.
fn normalize_a_indirection(node: &mut NodeEnum) {
    if let NodeEnum::AIndirection(ind) = node {
        // Recursively normalize the arg first
        if let Some(ref mut arg) = ind.arg {
            if let Some(ref mut inner_node) = arg.node {
                normalize_a_indirection(inner_node);
            }
        }

        // Now flatten: if arg is another AIndirection, pull up its contents
        loop {
            let inner_opt = ind.arg.as_ref().and_then(|arg| {
                if let Some(NodeEnum::AIndirection(inner)) = arg.node.as_ref() {
                    Some(inner.clone())
                } else {
                    None
                }
            });

            if let Some(inner) = inner_opt {
                // Replace arg with inner's arg
                ind.arg = inner.arg;
                // Prepend inner's indirection to current indirection
                let mut new_indirection = inner.indirection;
                new_indirection.append(&mut ind.indirection);
                ind.indirection = new_indirection;
            } else {
                break;
            }
        }

        // Merge leading String/AStar elements from indirection into ColumnRef fields
        // This normalizes `d1.r` which may parse as either structure
        // Also handles partial cases like `value.if2[1]` where we merge "if2" but keep AIndices
        if let Some(ref mut arg) = ind.arg {
            if let Some(NodeEnum::ColumnRef(col)) = arg.node.as_mut() {
                // Find how many leading elements are String or AStar
                let merge_count = ind
                    .indirection
                    .iter()
                    .take_while(|indir| {
                        matches!(
                            indir.node.as_ref(),
                            Some(NodeEnum::String(_) | NodeEnum::AStar(_))
                        )
                    })
                    .count();

                if merge_count > 0 {
                    // Split indirection: merge first `merge_count` into ColumnRef, keep rest
                    let remaining = ind.indirection.split_off(merge_count);
                    col.fields.append(&mut ind.indirection);
                    ind.indirection = remaining;

                    // If no indirection left, convert to just ColumnRef
                    if ind.indirection.is_empty() {
                        *node = NodeEnum::ColumnRef(col.clone());
                        return;
                    }
                }
            }
        }
    }

    // Recursively process all children
    match node {
        NodeEnum::SelectStmt(stmt) => {
            for target in &mut stmt.target_list {
                if let Some(ref mut n) = target.node {
                    normalize_a_indirection(n);
                }
            }
            for from in &mut stmt.from_clause {
                if let Some(ref mut n) = from.node {
                    normalize_a_indirection(n);
                }
            }
            if let Some(ref mut w) = stmt.where_clause {
                if let Some(ref mut n) = w.node {
                    normalize_a_indirection(n);
                }
            }
            for sort in &mut stmt.sort_clause {
                if let Some(ref mut n) = sort.node {
                    normalize_a_indirection(n);
                }
            }
        }
        NodeEnum::SortBy(sb) => {
            if let Some(ref mut n) = sb.node {
                if let Some(ref mut inner) = n.node {
                    normalize_a_indirection(inner);
                }
            }
        }
        NodeEnum::ResTarget(rt) => {
            if let Some(ref mut v) = rt.val {
                if let Some(ref mut n) = v.node {
                    normalize_a_indirection(n);
                }
            }
        }
        NodeEnum::FuncCall(fc) => {
            for arg in &mut fc.args {
                if let Some(ref mut n) = arg.node {
                    normalize_a_indirection(n);
                }
            }
            // Normalize NORMALIZE function's second argument:
            // NFC/NFD/NFKC/NFKD can be emitted as identifier but parsed back as ColumnRef
            // Convert ColumnRef with these values to AConst string
            if fc.funcname.len() == 1 {
                if let Some(NodeEnum::String(s)) = fc.funcname.first().and_then(|n| n.node.as_ref())
                {
                    if s.sval.eq_ignore_ascii_case("normalize") && fc.args.len() == 2 {
                        if let Some(NodeEnum::ColumnRef(cref)) = fc.args[1].node.as_ref() {
                            if cref.fields.len() == 1 {
                                if let Some(NodeEnum::String(field_str)) =
                                    cref.fields[0].node.as_ref()
                                {
                                    let form = field_str.sval.to_uppercase();
                                    if matches!(form.as_str(), "NFC" | "NFD" | "NFKC" | "NFKD") {
                                        // Convert to AConst string for normalization
                                        fc.args[1].node =
                                            Some(NodeEnum::AConst(pgls_query::protobuf::AConst {
                                                isnull: false,
                                                location: 0,
                                                val: Some(
                                                    pgls_query::protobuf::a_const::Val::Sval(
                                                        pgls_query::protobuf::String {
                                                            sval: form.to_lowercase(),
                                                        },
                                                    ),
                                                ),
                                            }));
                                    }
                                }
                            }
                        }
                        // Also lowercase AConst values for comparison
                        if let Some(NodeEnum::AConst(aconst)) = fc.args[1].node.as_mut() {
                            if let Some(pgls_query::protobuf::a_const::Val::Sval(s)) =
                                aconst.val.as_mut()
                            {
                                s.sval = s.sval.to_lowercase();
                            }
                        }
                    }
                }
            }
        }
        NodeEnum::AExpr(expr) => {
            if let Some(ref mut l) = expr.lexpr {
                if let Some(ref mut n) = l.node {
                    normalize_a_indirection(n);
                }
            }
            if let Some(ref mut r) = expr.rexpr {
                if let Some(ref mut n) = r.node {
                    normalize_a_indirection(n);
                }
            }
        }
        NodeEnum::UpdateStmt(stmt) => {
            for target in &mut stmt.target_list {
                if let Some(ref mut n) = target.node {
                    normalize_a_indirection(n);
                }
            }
            if let Some(ref mut w) = stmt.where_clause {
                if let Some(ref mut n) = w.node {
                    normalize_a_indirection(n);
                }
            }
        }
        NodeEnum::DeleteStmt(stmt) => {
            if let Some(ref mut w) = stmt.where_clause {
                if let Some(ref mut n) = w.node {
                    normalize_a_indirection(n);
                }
            }
        }
        NodeEnum::InsertStmt(stmt) => {
            if let Some(ref mut sel) = stmt.select_stmt {
                if let Some(ref mut n) = sel.node {
                    normalize_a_indirection(n);
                }
            }
        }
        NodeEnum::Constraint(c) => {
            if let Some(ref mut raw) = c.raw_expr {
                if let Some(ref mut n) = raw.node {
                    normalize_a_indirection(n);
                }
            }
        }
        NodeEnum::AlterDomainStmt(stmt) => {
            if let Some(ref mut def) = stmt.def {
                if let Some(ref mut n) = def.node {
                    normalize_a_indirection(n);
                    // For NOT NULL constraints on domains, clear keys and conname
                    // since they aren't emitted/reparsed
                    if let NodeEnum::Constraint(c) = n {
                        if c.contype == pgls_query::protobuf::ConstrType::ConstrNotnull as i32 {
                            c.keys.clear();
                            // conname is emitted, so leave it for comparison
                        }
                    }
                }
            }
        }
        NodeEnum::CreateDomainStmt(stmt) => {
            for constraint in &mut stmt.constraints {
                if let Some(ref mut n) = constraint.node {
                    normalize_a_indirection(n);
                }
            }
        }
        NodeEnum::NullTest(nt) => {
            if let Some(ref mut arg) = nt.arg {
                if let Some(ref mut n) = arg.node {
                    normalize_a_indirection(n);
                }
            }
        }
        NodeEnum::BoolExpr(be) => {
            // First flatten nested BoolExpr of the same type
            flatten_bool_expr(be);
            // Then recursively normalize children
            for arg in &mut be.args {
                if let Some(ref mut n) = arg.node {
                    normalize_a_indirection(n);
                }
            }
        }
        NodeEnum::RuleStmt(stmt) => {
            for action in &mut stmt.actions {
                if let Some(ref mut n) = action.node {
                    normalize_a_indirection(n);
                }
            }
        }
        NodeEnum::AlterTableStmt(stmt) => {
            for cmd in &mut stmt.cmds {
                if let Some(NodeEnum::AlterTableCmd(c)) = cmd.node.as_mut() {
                    if let Some(ref mut def) = c.def {
                        if let Some(ref mut n) = def.node {
                            normalize_a_indirection(n);
                        }
                    }
                }
            }
        }
        NodeEnum::CreateStatsStmt(stmt) => {
            for expr in &mut stmt.exprs {
                if let Some(ref mut n) = expr.node {
                    normalize_a_indirection(n);
                }
            }
        }
        NodeEnum::StatsElem(se) => {
            if let Some(ref mut expr) = se.expr {
                if let Some(ref mut n) = expr.node {
                    normalize_a_indirection(n);
                }
            }
        }
        NodeEnum::IndexStmt(stmt) => {
            for param in &mut stmt.index_params {
                if let Some(ref mut n) = param.node {
                    normalize_a_indirection(n);
                }
            }
        }
        NodeEnum::IndexElem(ie) => {
            if let Some(ref mut expr) = ie.expr {
                if let Some(ref mut n) = expr.node {
                    normalize_a_indirection(n);
                }
            }
        }
        NodeEnum::JoinExpr(je) => {
            if let Some(ref mut quals) = je.quals {
                if let Some(ref mut n) = quals.node {
                    normalize_a_indirection(n);
                }
            }
            if let Some(ref mut larg) = je.larg {
                if let Some(ref mut n) = larg.node {
                    normalize_a_indirection(n);
                }
            }
            if let Some(ref mut rarg) = je.rarg {
                if let Some(ref mut n) = rarg.node {
                    normalize_a_indirection(n);
                }
            }
        }
        NodeEnum::RangeSubselect(rs) => {
            if let Some(ref mut sub) = rs.subquery {
                if let Some(ref mut n) = sub.node {
                    normalize_a_indirection(n);
                }
            }
        }
        NodeEnum::SubLink(sl) => {
            if let Some(ref mut sub) = sl.subselect {
                if let Some(ref mut n) = sub.node {
                    normalize_a_indirection(n);
                }
            }
            if let Some(ref mut test) = sl.testexpr {
                if let Some(ref mut n) = test.node {
                    normalize_a_indirection(n);
                }
            }
        }
        _ => {}
    }
}

/// Normalize ObjectWithArgs by clearing objfuncargs.
///
/// When we emit from objargs, the reparsed objfuncargs may differ in count
/// (e.g., OUT parameters in objfuncargs but not in objargs).
fn normalize_object_with_args(node: &mut NodeEnum) {
    match node {
        NodeEnum::ObjectWithArgs(owa) => {
            // Clear objfuncargs - we emit from objargs only
            owa.objfuncargs.clear();
        }
        NodeEnum::DropStmt(ds) => {
            for obj in &mut ds.objects {
                if let Some(ref mut n) = obj.node {
                    normalize_object_with_args(n);
                }
            }
        }
        NodeEnum::AlterObjectSchemaStmt(stmt) => {
            if let Some(ref mut obj) = stmt.object {
                if let Some(ref mut n) = obj.node {
                    normalize_object_with_args(n);
                }
            }
        }
        NodeEnum::CommentStmt(stmt) => {
            if let Some(ref mut obj) = stmt.object {
                if let Some(ref mut n) = obj.node {
                    normalize_object_with_args(n);
                }
            }
        }
        NodeEnum::AlterFunctionStmt(stmt) => {
            if let Some(ref mut func) = stmt.func {
                func.objfuncargs.clear();
            }
        }
        _ => {}
    }
}

/// Normalize JoinExpr nodes - normalize semantically equivalent forms.
fn normalize_join_expr(node: &mut NodeEnum) {
    match node {
        NodeEnum::JoinExpr(je) => {
            // Normalize INNER JOIN ON TRUE to no quals (equivalent to CROSS JOIN)
            if je.jointype == pgls_query::protobuf::JoinType::JoinInner as i32 {
                let is_true_qual = je
                    .quals
                    .as_ref()
                    .and_then(|q| q.node.as_ref())
                    .map(|n| {
                        matches!(
                            n,
                            NodeEnum::AConst(c)
                                if matches!(c.val.as_ref(), Some(pgls_query::protobuf::a_const::Val::Boolval(b)) if b.boolval)
                        )
                    })
                    .unwrap_or(false);
                if is_true_qual {
                    je.quals = None;
                }
            }
            // NOTE: We do NOT clear join alias or join_using_alias here.
            // If the emitter doesn't emit them, the test should fail and we need to fix the emitter.
            // Recursively normalize nested joins and subqueries
            if let Some(ref mut larg) = je.larg {
                if let Some(ref mut n) = larg.node {
                    normalize_join_expr(n);
                }
            }
            if let Some(ref mut rarg) = je.rarg {
                if let Some(ref mut n) = rarg.node {
                    normalize_join_expr(n);
                }
            }
        }
        NodeEnum::RangeSubselect(rs) => {
            if let Some(ref mut sub) = rs.subquery {
                if let Some(ref mut n) = sub.node {
                    normalize_join_expr(n);
                }
            }
        }
        NodeEnum::SelectStmt(stmt) => {
            for from in &mut stmt.from_clause {
                if let Some(ref mut n) = from.node {
                    normalize_join_expr(n);
                }
            }
        }
        NodeEnum::ViewStmt(vs) => {
            if let Some(ref mut query) = vs.query {
                if let Some(ref mut n) = query.node {
                    normalize_join_expr(n);
                }
            }
        }
        NodeEnum::DeleteStmt(del) => {
            if let Some(ref mut where_clause) = del.where_clause {
                if let Some(ref mut n) = where_clause.node {
                    normalize_join_expr(n);
                }
            }
        }
        NodeEnum::SubLink(sl) => {
            if let Some(ref mut sub) = sl.subselect {
                if let Some(ref mut n) = sub.node {
                    normalize_join_expr(n);
                }
            }
        }
        _ => {}
    }
}

/// Normalize CreateForeignTableStmt partbound.
///
/// When partbound is None but inh_relations is non-empty, set a default partition bound.
/// Also clear table_elts for partitions since they inherit from parent.
fn normalize_foreign_table_partbound(node: &mut NodeEnum) {
    if let NodeEnum::CreateForeignTableStmt(stmt) = node {
        if let Some(ref mut base) = stmt.base_stmt {
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
            if let Some(query) = &mut cs.query {
                if let Some(n) = &mut query.node {
                    normalize_merge_support_func_recursive(n);
                }
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
    if let Some(node) = opt {
        if let Some(n) = &mut node.node {
            normalize_merge_support_func_recursive(n);
        }
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

/// Normalize SqlvalueFunction nodes to FuncCall for comparison.
///
/// PostgreSQL represents CURRENT_SCHEMA, CURRENT_USER, etc. as SqlvalueFunction
/// internally, but when we emit them as functions and reparse, they become FuncCall.
fn normalize_sql_value_function(node: &mut NodeEnum) {
    normalize_sql_value_function_recursive(node);
}

fn normalize_sql_value_function_recursive(node: &mut NodeEnum) {
    // First recursively process all children
    match node {
        NodeEnum::SelectStmt(stmt) => {
            for target in &mut stmt.target_list {
                if let Some(ref mut n) = target.node {
                    normalize_sql_value_function_recursive(n);
                }
            }
            for from in &mut stmt.from_clause {
                if let Some(ref mut n) = from.node {
                    normalize_sql_value_function_recursive(n);
                }
            }
            if let Some(ref mut w) = stmt.where_clause {
                if let Some(ref mut n) = w.node {
                    normalize_sql_value_function_recursive(n);
                }
            }
        }
        NodeEnum::AExpr(expr) => {
            if let Some(ref mut l) = expr.lexpr {
                if let Some(ref mut n) = l.node {
                    // Check if this is a SqlvalueFunction that needs conversion
                    if let NodeEnum::SqlvalueFunction(svf) = n {
                        if let Some(func_call) = sql_value_to_func_call(svf) {
                            *n = NodeEnum::FuncCall(Box::new(func_call));
                        }
                    } else {
                        normalize_sql_value_function_recursive(n);
                    }
                }
            }
            if let Some(ref mut r) = expr.rexpr {
                if let Some(ref mut n) = r.node {
                    if let NodeEnum::SqlvalueFunction(svf) = n {
                        if let Some(func_call) = sql_value_to_func_call(svf) {
                            *n = NodeEnum::FuncCall(Box::new(func_call));
                        }
                    } else {
                        normalize_sql_value_function_recursive(n);
                    }
                }
            }
        }
        NodeEnum::ResTarget(rt) => {
            if let Some(ref mut v) = rt.val {
                if let Some(ref mut n) = v.node {
                    if let NodeEnum::SqlvalueFunction(svf) = n {
                        if let Some(func_call) = sql_value_to_func_call(svf) {
                            *n = NodeEnum::FuncCall(Box::new(func_call));
                        }
                    } else {
                        normalize_sql_value_function_recursive(n);
                    }
                }
            }
        }
        NodeEnum::FuncCall(fc) => {
            for arg in &mut fc.args {
                if let Some(ref mut n) = arg.node {
                    if let NodeEnum::SqlvalueFunction(svf) = n {
                        if let Some(func_call) = sql_value_to_func_call(svf) {
                            *n = NodeEnum::FuncCall(Box::new(func_call));
                        }
                    } else {
                        normalize_sql_value_function_recursive(n);
                    }
                }
            }
        }
        _ => {}
    }
}

fn sql_value_to_func_call(
    svf: &pgls_query::protobuf::SqlValueFunction,
) -> Option<pgls_query::protobuf::FuncCall> {
    // Map SqlValueFunctionOp to function name
    let func_name = match svf.op() {
        pgls_query::protobuf::SqlValueFunctionOp::SvfopCurrentSchema => "current_schema",
        pgls_query::protobuf::SqlValueFunctionOp::SvfopCurrentUser => "current_user",
        pgls_query::protobuf::SqlValueFunctionOp::SvfopSessionUser => "session_user",
        pgls_query::protobuf::SqlValueFunctionOp::SvfopUser => "user",
        pgls_query::protobuf::SqlValueFunctionOp::SvfopCurrentCatalog => "current_catalog",
        pgls_query::protobuf::SqlValueFunctionOp::SvfopCurrentDate => "current_date",
        pgls_query::protobuf::SqlValueFunctionOp::SvfopCurrentTime => "current_time",
        pgls_query::protobuf::SqlValueFunctionOp::SvfopCurrentTimeN => return None, // Has args
        pgls_query::protobuf::SqlValueFunctionOp::SvfopCurrentTimestamp => "current_timestamp",
        pgls_query::protobuf::SqlValueFunctionOp::SvfopCurrentTimestampN => return None, // Has args
        pgls_query::protobuf::SqlValueFunctionOp::SvfopLocaltime => "localtime",
        pgls_query::protobuf::SqlValueFunctionOp::SvfopLocaltimeN => return None, // Has args
        pgls_query::protobuf::SqlValueFunctionOp::SvfopLocaltimestamp => "localtimestamp",
        pgls_query::protobuf::SqlValueFunctionOp::SvfopLocaltimestampN => return None, // Has args
        pgls_query::protobuf::SqlValueFunctionOp::SvfopCurrentRole => "current_role",
        _ => return None,
    };

    Some(pgls_query::protobuf::FuncCall {
        funcname: vec![pgls_query::protobuf::Node {
            node: Some(NodeEnum::String(pgls_query::protobuf::String {
                sval: func_name.to_string(),
            })),
        }],
        args: vec![],
        agg_order: vec![],
        agg_filter: None,
        over: None,
        agg_within_group: false,
        agg_star: false,
        agg_distinct: false,
        func_variadic: false,
        funcformat: pgls_query::protobuf::CoercionForm::CoerceExplicitCall.into(),
        location: 0,
    })
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
