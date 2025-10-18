use super::node_list::emit_dot_separated_list;
use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};
use pgt_query::protobuf::CreateOpFamilyStmt;

pub(super) fn emit_create_op_family_stmt(e: &mut EventEmitter, n: &CreateOpFamilyStmt) {
    e.group_start(GroupKind::CreateOpFamilyStmt);

    e.token(TokenKind::CREATE_KW);
    e.space();
    e.token(TokenKind::IDENT("OPERATOR".to_string()));
    e.space();
    e.token(TokenKind::IDENT("FAMILY".to_string()));
    e.space();

    // Operator family name
    emit_dot_separated_list(e, &n.opfamilyname);

    // USING access_method
    e.space();
    e.token(TokenKind::USING_KW);
    e.space();
    e.token(TokenKind::IDENT(n.amname.clone()));

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
