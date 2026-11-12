use pgls_query::protobuf::CreateDomainStmt;

use super::prelude::*;

pub(super) fn emit_create_domain_stmt(e: &mut EventEmitter, n: &CreateDomainStmt) {
    e.group_start();

    e.token(TokenKind::CREATE_KW);
    e.space();
    e.token(TokenKind::DOMAIN_KW);

    // Emit domain name (qualified name as a list)
    if !n.domainname.is_empty() {
        e.space();
        emit_dot_separated_list(e, &n.domainname);
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
        e.indent_start();
        for constraint in &n.constraints {
            e.line(LineType::SoftOrSpace);
            emit_node(e, constraint);
        }
        e.indent_end();
    }

    e.group_end();
}
