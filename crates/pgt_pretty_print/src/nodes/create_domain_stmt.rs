use pgt_query::protobuf::CreateDomainStmt;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_create_domain_stmt(e: &mut EventEmitter, n: &CreateDomainStmt) {
    e.group_start(GroupKind::CreateDomainStmt);

    e.token(TokenKind::CREATE_KW);
    e.space();
    e.token(TokenKind::DOMAIN_KW);

    // Emit domain name (qualified name as a list)
    if !n.domainname.is_empty() {
        e.space();
        super::node_list::emit_dot_separated_list(e, &n.domainname);
    }

    // Emit AS type_name
    if let Some(ref type_name) = n.type_name {
        e.space();
        e.token(TokenKind::AS_KW);
        e.space();
        super::emit_type_name(e, type_name);
    }

    // Emit COLLATE clause
    if let Some(ref coll_clause) = n.coll_clause {
        e.space();
        super::emit_collate_clause(e, coll_clause);
    }

    // Emit constraints (CHECK, NOT NULL, DEFAULT, etc.)
    if !n.constraints.is_empty() {
        for constraint in &n.constraints {
            e.space();
            super::emit_node(constraint, e);
        }
    }

    e.token(TokenKind::SEMICOLON);
    e.group_end();
}
