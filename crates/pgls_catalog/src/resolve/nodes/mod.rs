//! One module per parse node, like the nodes of `pgls_pretty_print`. [`resolve_node_enum`]
//! dispatches every node; nodes that only contain other nodes are walked right there.

mod alias;
mod column_ref;
mod common_table_expr;
mod copy_stmt;
mod create_function_stmt;
mod create_table_as_stmt;
mod declare_cursor_stmt;
mod delete_stmt;
mod explain_stmt;
mod from_clause;
mod func_call;
mod insert_stmt;
mod join_expr;
mod merge_stmt;
mod on_conflict_clause;
mod range_function;
mod range_subselect;
mod range_var;
mod res_target;
mod select_stmt;
mod type_name;
mod update_stmt;
mod view_stmt;
mod with_clause;

use pgls_query::{Node, NodeEnum};

use super::{
    lookups::{
        record_a_expr, record_create_function, record_function, record_sort_by, record_sub_link,
        record_unresolved,
    },
    resolver::Resolver,
    scope::Columns,
};

pub(super) use column_ref::record_column_ref_lookup;
use column_ref::resolve_column_ref;
use copy_stmt::resolve_copy_stmt;
use create_function_stmt::resolve_create_function_stmt;
use create_table_as_stmt::resolve_create_table_as_stmt;
use declare_cursor_stmt::resolve_declare_cursor_stmt;
use delete_stmt::resolve_delete_stmt;
use explain_stmt::resolve_explain_stmt;
use func_call::resolve_func_call;
use insert_stmt::resolve_insert_stmt;
use merge_stmt::resolve_merge_stmt;
use range_var::lookup_relation;
#[cfg(test)]
pub(crate) use select_stmt::RESOLVED_QUERIES;
use select_stmt::resolve_select_stmt;
use type_name::{record_type_names, resolve_type_name};
use update_stmt::resolve_update_stmt;
use view_stmt::resolve_view_stmt;

/// Resolves a node. Returns the output columns of queries, and of data-modifying statements
/// with `RETURNING`, if they are known.
pub(super) fn resolve_node(r: &mut Resolver, node: &Node) -> Columns {
    resolve_node_enum(r, node.node.as_ref()?)
}

pub(super) fn resolve_node_enum(r: &mut Resolver, node: &NodeEnum) -> Columns {
    match node {
        NodeEnum::SelectStmt(_)
        | NodeEnum::InsertStmt(_)
        | NodeEnum::UpdateStmt(_)
        | NodeEnum::DeleteStmt(_)
        | NodeEnum::MergeStmt(_) => {
            r.query_depth += 1;
            let columns = match node {
                NodeEnum::SelectStmt(n) => resolve_select_stmt(r, n),
                NodeEnum::InsertStmt(n) => resolve_insert_stmt(r, n),
                NodeEnum::UpdateStmt(n) => resolve_update_stmt(r, n),
                NodeEnum::DeleteStmt(n) => resolve_delete_stmt(r, n),
                NodeEnum::MergeStmt(n) => resolve_merge_stmt(r, n),
                _ => None,
            };
            r.query_depth -= 1;
            return columns;
        }
        NodeEnum::CreateFunctionStmt(n) => {
            record_type_names(r, node);
            record_create_function(r, n);
            resolve_create_function_stmt(r, n);
        }
        NodeEnum::CreateTableAsStmt(n) => resolve_create_table_as_stmt(r, n),
        NodeEnum::ViewStmt(n) => resolve_view_stmt(r, n),
        NodeEnum::ExplainStmt(n) => resolve_explain_stmt(r, n),
        NodeEnum::DeclareCursorStmt(n) => resolve_declare_cursor_stmt(r, n),
        NodeEnum::CopyStmt(n) => resolve_copy_stmt(r, n),
        // The relations of `TRUNCATE` and `LOCK`, looked up like the ones of queries
        // ([`ExecuteTruncate`], [`LockTableCommand`]).
        //
        // [`ExecuteTruncate`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/tablecmds.c#L1870
        // [`LockTableCommand`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/lockcmds.c#L41
        NodeEnum::TruncateStmt(n) => lookup_relations(r, &n.relations),
        NodeEnum::LockStmt(n) => lookup_relations(r, &n.relations),
        // The parameter types and the query of `PREPARE` ([`PrepareQuery`]).
        //
        // [`PrepareQuery`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/prepare.c#L59
        NodeEnum::PrepareStmt(n) => {
            r.depends_on_file();
            for argtype in &n.argtypes {
                if let Some(NodeEnum::TypeName(type_name)) = argtype.node.as_ref() {
                    resolve_type_name(r, type_name);
                }
            }
            resolve_opt(r, &n.query);
        }
        NodeEnum::CallStmt(n) => {
            if let Some(call) = &n.funccall {
                record_function(r, call);
                for arg in &call.args {
                    record_unresolved_node(r, arg, false);
                }
                super::expr::resolve_procedure_call(r, call);
            }
        }
        NodeEnum::ColumnRef(n) => resolve_column_ref(r, n),
        NodeEnum::MergeSupportFunc(n) => merge_stmt::check_merge_action(r, n),
        NodeEnum::FuncCall(n) => resolve_func_call(r, n),
        NodeEnum::TypeCast(n) => {
            resolve_opt(r, &n.arg);
            if let Some(type_name) = &n.type_name {
                resolve_type_name(r, type_name);
            }
        }
        NodeEnum::SubLink(n) => {
            record_sub_link(r, n);
            resolve_opt(r, &n.testexpr);
            if let Some(query) = n.subselect.as_deref() {
                // The subquery sees the enclosing queries, like in Postgres, where
                // `transformSubLink` analyses it with the enclosing query as its parent.
                let enclosing_output = r.output.take();
                resolve_node(r, query);
                let output = crate::resolve::resolve_unknown_outputs(r.catalog, r.output.take());
                r.output = enclosing_output;
                r.remember_subquery_output(query, output);
            }
        }
        // The subscripts of an assignment, like `a[i]` in `UPDATE ... SET a[i] = 1`, are
        // expressions.
        NodeEnum::ResTarget(n) => {
            resolve_list(r, &n.indirection);
            resolve_opt(r, &n.val);
        }
        NodeEnum::AExpr(n) => {
            record_a_expr(r, n);
            resolve_opt(r, &n.lexpr);
            resolve_opt(r, &n.rexpr);
        }
        NodeEnum::BoolExpr(n) => resolve_list(r, &n.args),
        NodeEnum::NullTest(n) => resolve_opt(r, &n.arg),
        NodeEnum::BooleanTest(n) => resolve_opt(r, &n.arg),
        NodeEnum::CaseExpr(n) => {
            resolve_opt(r, &n.arg);
            resolve_list(r, &n.args);
            resolve_opt(r, &n.defresult);
        }
        NodeEnum::CaseWhen(n) => {
            resolve_opt(r, &n.expr);
            resolve_opt(r, &n.result);
        }
        NodeEnum::CoalesceExpr(n) => resolve_list(r, &n.args),
        NodeEnum::MinMaxExpr(n) => resolve_list(r, &n.args),
        NodeEnum::RowExpr(n) => resolve_list(r, &n.args),
        NodeEnum::AArrayExpr(n) => resolve_list(r, &n.elements),
        NodeEnum::AIndirection(n) => {
            resolve_opt(r, &n.arg);
            resolve_list(r, &n.indirection);
        }
        NodeEnum::AIndices(n) => {
            resolve_opt(r, &n.lidx);
            resolve_opt(r, &n.uidx);
        }
        NodeEnum::CollateClause(n) => resolve_opt(r, &n.arg),
        NodeEnum::NamedArgExpr(n) => resolve_opt(r, &n.arg),
        NodeEnum::SortBy(n) => {
            record_sort_by(r, n);
            resolve_opt(r, &n.node);
        }
        NodeEnum::GroupingSet(n) => resolve_list(r, &n.content),
        NodeEnum::GroupingFunc(n) => resolve_list(r, &n.args),
        NodeEnum::MultiAssignRef(n) => resolve_opt(r, &n.source),
        NodeEnum::WindowDef(n) => {
            resolve_list(r, &n.partition_clause);
            resolve_list(r, &n.order_clause);
            resolve_opt(r, &n.start_offset);
            resolve_opt(r, &n.end_offset);
        }
        NodeEnum::List(n) => resolve_list(r, &n.items),
        // `$1` in the body of a SQL function.
        NodeEnum::ParamRef(n) => {
            let params = r.function.map_or(0, |function| function.params.len());
            if let Some(index) = usize::try_from(n.number)
                .ok()
                .and_then(|number| number.checked_sub(1))
                .filter(|&index| index < params)
            {
                r.record_parameter(index, n.location);
            }
        }
        // Field names in indirection (`(row).field`) and leaves.
        NodeEnum::String(_)
        | NodeEnum::AStar(_)
        | NodeEnum::AConst(_)
        | NodeEnum::SqlvalueFunction(_)
        | NodeEnum::SetToDefault(_)
        | NodeEnum::CurrentOfExpr(_) => {}
        // Anything else (DDL, XML and JSON expressions, ...) is not checked, but its types are
        // recorded.
        _ => {
            r.depends_on_file();
            record_type_names(r, node);
            record_unresolved(r, node.to_ref(), r.query_depth > 0);
        }
    }
    None
}

/// Looks up the relations of `TRUNCATE` or `LOCK`.
fn lookup_relations(r: &mut Resolver, relations: &[Node]) {
    r.depends_on_file();
    for relation in relations {
        if let Some(NodeEnum::RangeVar(relation)) = relation.node.as_ref() {
            lookup_relation(r, relation);
        }
    }
}

/// Records the lookups of `node`, which the resolver doesn't resolve, see [`record_unresolved`].
pub(super) fn record_unresolved_node(r: &mut Resolver, node: &Node, in_query: bool) {
    if let Some(node) = node.node.as_ref() {
        record_unresolved(r, node.to_ref(), in_query);
    }
}

pub(super) fn resolve_opt(r: &mut Resolver, node: &Option<Box<Node>>) {
    if let Some(node) = node.as_deref() {
        resolve_node(r, node);
    }
}

pub(super) fn resolve_list(r: &mut Resolver, nodes: &[Node]) {
    for node in nodes {
        resolve_node(r, node);
    }
}
