use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
    nodes::node_list::emit_comma_separated_list,
};
use pgls_query::protobuf::XmlExpr;

pub(super) fn emit_xml_expr(e: &mut EventEmitter, n: &XmlExpr) {
    e.group_start(GroupKind::XmlExpr);

    // XmlExprOp enum:
    // 0 = Undefined
    // 1 = IsXmlconcat
    // 2 = IsXmlelement
    // 3 = IsXmlforest
    // 4 = IsXmlparse
    // 5 = IsXmlpi
    // 6 = IsXmlroot
    // 7 = IsXmlserialize
    // 8 = IsDocument
    match n.op {
        1 => {
            // XMLCONCAT
            e.token(TokenKind::IDENT("XMLCONCAT".to_string()));
            e.token(TokenKind::L_PAREN);
            emit_comma_separated_list(e, &n.args, super::emit_node);
            e.token(TokenKind::R_PAREN);
        }
        2 => {
            // XMLELEMENT
            e.token(TokenKind::IDENT("XMLELEMENT".to_string()));
            e.token(TokenKind::L_PAREN);
            e.token(TokenKind::IDENT("NAME".to_string()));
            e.space();
            e.token(TokenKind::IDENT(n.name.clone()));

            // Named args represent XML attributes (XMLATTRIBUTES)
            if !n.named_args.is_empty() {
                e.token(TokenKind::COMMA);
                e.space();
                e.token(TokenKind::IDENT("XMLATTRIBUTES".to_string()));
                e.token(TokenKind::L_PAREN);
                emit_comma_separated_list(e, &n.named_args, super::emit_node);
                e.token(TokenKind::R_PAREN);
            }

            // Args represent content
            if !n.args.is_empty() {
                e.token(TokenKind::COMMA);
                e.space();
                emit_comma_separated_list(e, &n.args, super::emit_node);
            }

            e.token(TokenKind::R_PAREN);
        }
        3 => {
            // XMLFOREST
            e.token(TokenKind::IDENT("XMLFOREST".to_string()));
            e.token(TokenKind::L_PAREN);
            emit_comma_separated_list(e, &n.named_args, super::emit_node);
            e.token(TokenKind::R_PAREN);
        }
        4 => {
            // XMLPARSE
            e.token(TokenKind::IDENT("XMLPARSE".to_string()));
            e.token(TokenKind::L_PAREN);
            // xmloption: 0=XMLOPTION_DOCUMENT, 1=XMLOPTION_CONTENT
            if n.xmloption == 0 {
                e.token(TokenKind::IDENT("DOCUMENT".to_string()));
            } else {
                e.token(TokenKind::IDENT("CONTENT".to_string()));
            }
            e.space();
            if !n.args.is_empty() {
                super::emit_node(&n.args[0], e);
            }
            e.token(TokenKind::R_PAREN);
        }
        5 => {
            // XMLPI
            e.token(TokenKind::IDENT("XMLPI".to_string()));
            e.token(TokenKind::L_PAREN);
            e.token(TokenKind::IDENT("NAME".to_string()));
            e.space();
            e.token(TokenKind::IDENT(n.name.clone()));

            if !n.args.is_empty() {
                e.token(TokenKind::COMMA);
                e.space();
                super::emit_node(&n.args[0], e);
            }

            e.token(TokenKind::R_PAREN);
        }
        6 => {
            // XMLROOT
            e.token(TokenKind::IDENT("XMLROOT".to_string()));
            e.token(TokenKind::L_PAREN);
            if !n.args.is_empty() {
                super::emit_node(&n.args[0], e);
            }
            e.token(TokenKind::COMMA);
            e.space();
            e.token(TokenKind::IDENT("VERSION".to_string()));
            e.space();
            if n.args.len() > 1 {
                super::emit_node(&n.args[1], e);
            } else {
                e.token(TokenKind::IDENT("NO VALUE".to_string()));
            }
            // Handle standalone option if there's a 3rd arg
            if n.args.len() > 2 {
                e.token(TokenKind::COMMA);
                e.space();
                e.token(TokenKind::IDENT("STANDALONE".to_string()));
                e.space();
                // The third arg encodes the standalone option
                // This might need more complex handling based on the actual value
                super::emit_node(&n.args[2], e);
            }
            e.token(TokenKind::R_PAREN);
        }
        7 => {
            // XMLSERIALIZE - this is handled by XmlSerialize node, not XmlExpr
            // But we'll handle it here just in case
            e.token(TokenKind::IDENT("XMLSERIALIZE".to_string()));
            e.token(TokenKind::L_PAREN);
            if n.xmloption == 0 {
                e.token(TokenKind::IDENT("DOCUMENT".to_string()));
            } else {
                e.token(TokenKind::IDENT("CONTENT".to_string()));
            }
            e.space();
            if !n.args.is_empty() {
                emit_comma_separated_list(e, &n.args, super::emit_node);
            }
            e.token(TokenKind::R_PAREN);
        }
        8 => {
            // IS DOCUMENT - expr IS DOCUMENT
            if !n.args.is_empty() {
                super::emit_node(&n.args[0], e);
            }
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::IS_KW);
            e.space();
            e.token(TokenKind::IDENT("DOCUMENT".to_string()));
        }
        _ => {
            // Unknown/Undefined XML operation - emit XMLCOMMENT as fallback
            e.token(TokenKind::IDENT("XMLCOMMENT".to_string()));
            e.token(TokenKind::L_PAREN);
            if !n.args.is_empty() {
                super::emit_node(&n.args[0], e);
            }
            e.token(TokenKind::R_PAREN);
        }
    }

    e.group_end();
}
