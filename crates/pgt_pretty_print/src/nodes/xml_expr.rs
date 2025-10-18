use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
    nodes::node_list::emit_comma_separated_list,
};
use pgt_query::protobuf::XmlExpr;

pub(super) fn emit_xml_expr(e: &mut EventEmitter, n: &XmlExpr) {
    e.group_start(GroupKind::XmlExpr);

    // XmlExprOp enum: IsXmlelement = 0, IsXmlconcat = 1, IsXmlcomment = 2, etc.
    match n.op {
        0 => {
            // XMLELEMENT
            e.token(TokenKind::IDENT("XMLELEMENT".to_string()));
            e.token(TokenKind::L_PAREN);
            e.token(TokenKind::IDENT("NAME".to_string()));
            e.space();
            e.token(TokenKind::IDENT(n.name.clone()));

            if !n.args.is_empty() {
                e.token(TokenKind::COMMA);
                e.space();
                emit_comma_separated_list(e, &n.args, super::emit_node);
            }

            e.token(TokenKind::R_PAREN);
        }
        1 => {
            // XMLCONCAT
            e.token(TokenKind::IDENT("XMLCONCAT".to_string()));
            e.token(TokenKind::L_PAREN);
            emit_comma_separated_list(e, &n.args, super::emit_node);
            e.token(TokenKind::R_PAREN);
        }
        2 => {
            // XMLCOMMENT
            e.token(TokenKind::IDENT("XMLCOMMENT".to_string()));
            e.token(TokenKind::L_PAREN);
            if !n.args.is_empty() {
                super::emit_node(&n.args[0], e);
            }
            e.token(TokenKind::R_PAREN);
        }
        3 => {
            // XMLFOREST
            e.token(TokenKind::IDENT("XMLFOREST".to_string()));
            e.token(TokenKind::L_PAREN);
            emit_comma_separated_list(e, &n.args, super::emit_node);
            e.token(TokenKind::R_PAREN);
        }
        4 => {
            // XMLPI
            e.token(TokenKind::IDENT("XMLPI".to_string()));
            e.token(TokenKind::L_PAREN);
            e.token(TokenKind::IDENT("NAME".to_string()));
            e.space();
            e.token(TokenKind::IDENT(n.name.clone()));

            if !n.args.is_empty() {
                e.token(TokenKind::COMMA);
                e.space();
                emit_comma_separated_list(e, &n.args, super::emit_node);
            }

            e.token(TokenKind::R_PAREN);
        }
        5 => {
            // XMLROOT
            e.token(TokenKind::IDENT("XMLROOT".to_string()));
            e.token(TokenKind::L_PAREN);
            emit_comma_separated_list(e, &n.args, super::emit_node);
            e.token(TokenKind::R_PAREN);
        }
        _ => {
            // Unknown XML operation
            e.token(TokenKind::IDENT("XMLFUNC".to_string()));
            e.token(TokenKind::L_PAREN);
            e.token(TokenKind::R_PAREN);
        }
    }

    e.group_end();
}
