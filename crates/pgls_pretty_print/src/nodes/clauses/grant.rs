//! The lists of GRANT, REVOKE and ALTER DEFAULT PRIVILEGES.

use pgls_query::protobuf::AccessPriv;

use crate::nodes::prelude::*;

/// Emits the privileges, objects, or roles of a `GRANT` or `REVOKE`. The list is its own group, so
/// it stays on one line when it fits even though the statement broke, and otherwise fills each line
/// with as many items as fit, wrapping one level deeper than its clause.
///
/// The deeper level starts after the first item, which shares the clause's line: a column list
/// such as `insert (a, b)` that breaks closes its parenthesis under the line that opened it.
pub(crate) fn emit_grant_list<F>(e: &mut EventEmitter, nodes: &[pgls_query::Node], render: F)
where
    F: Fn(&mut EventEmitter, &pgls_query::Node),
{
    let Some((first, rest)) = nodes.split_first() else {
        return;
    };

    e.group_start();
    render(e, first);
    if !rest.is_empty() {
        e.indent_start();
        for node in rest {
            emit_list_comma(e, Some(node), ListSeparatorSpacing::Fill);
            render(e, node);
        }
        e.indent_end();
    }
    e.group_end();
}

/// Emits a privilege of GRANT or REVOKE: `select`, `update (a, b)`, or `all (a, b)` for all
/// privileges on some columns.
///
/// An `AccessPriv` reached through `emit_node`, in ALTER DEFAULT PRIVILEGES, is written by
/// `emit_access_priv` instead, which writes all privileges as `ALL PRIVILEGES` and may break
/// before the column list.
pub(crate) fn emit_privilege(e: &mut EventEmitter, priv_node: &AccessPriv) {
    if priv_node.priv_name.is_empty() && !priv_node.cols.is_empty() {
        e.token(TokenKind::ALL_KW);
    } else if !priv_node.priv_name.is_empty() {
        emit_keywords(e, &priv_node.priv_name);
    }

    // Handle column privileges
    if !priv_node.cols.is_empty() {
        e.space();
        emit_parenthesized_list(e, &priv_node.cols, emit_node);
    }
}
