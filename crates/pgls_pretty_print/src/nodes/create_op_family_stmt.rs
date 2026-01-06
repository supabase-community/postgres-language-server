use super::node_list::emit_dot_separated_list;
use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};
use pgls_query::protobuf::CreateOpFamilyStmt;

pub(super) fn emit_create_op_family_stmt(e: &mut EventEmitter, n: &CreateOpFamilyStmt) {
    e.group_start(GroupKind::CreateOpFamilyStmt);

    e.token(TokenKind::CREATE_KW);
    e.space();
    e.token(TokenKind::OPERATOR_KW);
    e.space();
    e.token(TokenKind::FAMILY_KW);
    e.space();

    // Operator family name
    emit_dot_separated_list(e, &n.opfamilyname);

    // USING access_method
    e.line(LineType::SoftOrSpace);
    e.token(TokenKind::USING_KW);
    e.space();
    e.token(TokenKind::IDENT(n.amname.clone()));

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
