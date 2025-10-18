use super::node_list::{emit_comma_separated_list, emit_dot_separated_list};
use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};
use pgt_query::protobuf::CreateOpClassStmt;

pub(super) fn emit_create_op_class_stmt(e: &mut EventEmitter, n: &CreateOpClassStmt) {
    e.group_start(GroupKind::CreateOpClassStmt);

    e.token(TokenKind::CREATE_KW);
    e.space();
    e.token(TokenKind::IDENT("OPERATOR".to_string()));
    e.space();
    e.token(TokenKind::CLASS_KW);
    e.space();

    // Operator class name
    emit_dot_separated_list(e, &n.opclassname);

    // DEFAULT
    if n.is_default {
        e.space();
        e.token(TokenKind::DEFAULT_KW);
    }

    // FOR TYPE
    e.line(LineType::SoftOrSpace);
    e.indent_start();
    e.token(TokenKind::FOR_KW);
    e.space();
    e.token(TokenKind::TYPE_KW);
    e.space();
    if let Some(ref datatype) = n.datatype {
        super::emit_type_name(e, datatype);
    }

    // USING access_method
    e.line(LineType::SoftOrSpace);
    e.token(TokenKind::USING_KW);
    e.space();
    e.token(TokenKind::IDENT(n.amname.clone()));

    // FAMILY
    if !n.opfamilyname.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::IDENT("FAMILY".to_string()));
        e.space();
        emit_dot_separated_list(e, &n.opfamilyname);
    }

    // AS items
    if !n.items.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::AS_KW);
        e.space();
        emit_comma_separated_list(e, &n.items, super::emit_node);
    }

    e.indent_end();

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
