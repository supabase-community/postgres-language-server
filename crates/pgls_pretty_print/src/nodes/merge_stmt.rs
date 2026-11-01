use pgls_query::protobuf::MergeStmt;

use super::emit_merge_when_clause;
use super::prelude::*;

pub(super) fn emit_merge_stmt(e: &mut EventEmitter, n: &MergeStmt) {
    e.group_start();

    if let Some(ref with_clause) = n.with_clause {
        super::emit_with_clause(e, with_clause);
        e.line(LineType::SoftOrSpace);
    }

    e.token(TokenKind::MERGE_KW);
    e.space();
    e.token(TokenKind::INTO_KW);
    e.space();

    // Target relation
    if let Some(ref relation) = n.relation {
        super::emit_range_var(e, relation);
    }

    // USING clause
    if let Some(ref source) = n.source_relation {
        emit_comments_before_keyword(e, "using", source);
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::USING_KW);
        e.space();
        emit_node(e, source);
    }

    // ON condition
    if let Some(ref condition) = n.join_condition {
        emit_comments_before_keyword(e, "on", condition);
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::ON_KW);
        e.space();
        emit_node(e, condition);
    }

    // WHEN clauses. A branch has no location of its own, so a comment before `WHEN` is attached
    // to the node that follows the last node written before it. That node has to be the branch's
    // own first node: after a branch without nodes, `WHEN MATCHED THEN DELETE`, it belongs to a
    // later branch.
    let mut last_location = n
        .join_condition
        .as_ref()
        .and_then(|node| node.node.as_ref())
        .and_then(|node| crate::comments::last_node_location(node.to_ref()));
    for when_clause_node in &n.merge_when_clauses {
        let Some(NodeEnum::MergeWhenClause(when_clause)) = when_clause_node.node.as_ref() else {
            emit_node(e, when_clause_node);
            continue;
        };
        let branch = pgls_query::NodeRef::MergeWhenClause(when_clause);
        if let Some(first) = crate::comments::first_node_location(branch)
            && last_location.and_then(|location| e.next_node_location(location)) == Some(first)
        {
            e.take_comments_before_keyword_at(first, "when");
        }
        e.line(LineType::SoftOrSpace);
        emit_merge_when_clause(e, when_clause);
        last_location = crate::comments::last_node_location(branch).or(last_location);
    }

    // RETURNING clause - wrap in group for compact formatting
    if let Some(first) = n.returning_list.first() {
        emit_comments_before_keyword(e, "returning", first);
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::RETURNING_KW);
        e.space();
        e.indent_start();
        emit_comma_separated_list(e, &n.returning_list, emit_node);
        e.indent_end();
    }

    e.group_end();
}
