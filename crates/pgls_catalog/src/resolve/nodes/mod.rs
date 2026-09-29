//! One module per parse node, like the nodes of `pgls_pretty_print`. [`resolve_node_enum`]
//! dispatches every node; nodes that only contain other nodes are walked right there.

mod alias;
mod column_ref;
mod common_table_expr;
mod copy_stmt;
mod create_table_as_stmt;
mod declare_cursor_stmt;
mod delete_stmt;
mod explain_stmt;
mod from_clause;
mod func_call;
mod insert_stmt;
mod join_expr;
mod on_conflict_clause;
mod range_function;
mod range_subselect;
mod range_var;
mod res_target;
mod select_stmt;
mod string;
mod type_name;
mod update_stmt;
mod view_stmt;
mod with_clause;

use pgls_query::{Node, NodeEnum};

use super::{resolver::Resolver, scope::Columns};

use column_ref::resolve_column_ref;
use copy_stmt::resolve_copy_stmt;
use create_table_as_stmt::resolve_create_table_as_stmt;
use declare_cursor_stmt::resolve_declare_cursor_stmt;
use delete_stmt::resolve_delete_stmt;
use explain_stmt::resolve_explain_stmt;
use func_call::resolve_func_call;
use insert_stmt::resolve_insert_stmt;
use select_stmt::resolve_select_stmt;
use type_name::resolve_type_name;
use update_stmt::resolve_update_stmt;
use view_stmt::resolve_view_stmt;

/// Resolves a node. Returns the output columns of queries, and of data-modifying statements
/// with `RETURNING`, if they are known.
pub(super) fn resolve_node(r: &mut Resolver, node: &Node) -> Columns {
    resolve_node_enum(r, node.node.as_ref()?)
}

pub(super) fn resolve_node_enum(r: &mut Resolver, node: &NodeEnum) -> Columns {
    match node {
        NodeEnum::SelectStmt(n) => return resolve_select_stmt(r, n),
        NodeEnum::InsertStmt(n) => return resolve_insert_stmt(r, n),
        NodeEnum::UpdateStmt(n) => return resolve_update_stmt(r, n),
        NodeEnum::DeleteStmt(n) => return resolve_delete_stmt(r, n),
        NodeEnum::CreateTableAsStmt(n) => resolve_create_table_as_stmt(r, n),
        NodeEnum::ViewStmt(n) => resolve_view_stmt(r, n),
        NodeEnum::ExplainStmt(n) => resolve_explain_stmt(r, n),
        NodeEnum::DeclareCursorStmt(n) => resolve_declare_cursor_stmt(r, n),
        NodeEnum::CopyStmt(n) => resolve_copy_stmt(r, n),
        NodeEnum::ColumnRef(n) => resolve_column_ref(r, n),
        NodeEnum::FuncCall(n) => resolve_func_call(r, n),
        NodeEnum::TypeCast(n) => {
            resolve_opt(r, &n.arg);
            if let Some(type_name) = &n.type_name {
                resolve_type_name(r, type_name);
            }
        }
        NodeEnum::SubLink(n) => {
            resolve_opt(r, &n.testexpr);
            resolve_opt(r, &n.subselect);
        }
        NodeEnum::ResTarget(n) => resolve_opt(r, &n.val),
        NodeEnum::AExpr(n) => {
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
        NodeEnum::SortBy(n) => resolve_opt(r, &n.node),
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
        // Field names in indirection (`(row).field`) and leaves.
        NodeEnum::String(_)
        | NodeEnum::AStar(_)
        | NodeEnum::AConst(_)
        | NodeEnum::ParamRef(_)
        | NodeEnum::SqlvalueFunction(_)
        | NodeEnum::SetToDefault(_)
        | NodeEnum::CurrentOfExpr(_) => {}
        // Anything else (XML and JSON expressions, ...) is not checked.
        _ => r.depends_on_file(),
    }
    None
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
