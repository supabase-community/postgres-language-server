use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
    nodes::node_list::emit_comma_separated_list,
};
use pgls_query::protobuf::{XmlExpr, XmlOptionType};

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
            // Names with special characters need quoting
            super::emit_identifier_maybe_quoted(e, &n.name);

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
            // XmlOptionType: XmloptionDocument=1, XmloptionContent=2
            match n.xmloption() {
                XmlOptionType::XmloptionDocument => {
                    e.token(TokenKind::IDENT("DOCUMENT".to_string()));
                }
                _ => {
                    e.token(TokenKind::IDENT("CONTENT".to_string()));
                }
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
            // Names with special characters need quoting
            super::emit_identifier_maybe_quoted(e, &n.name);

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
                // Check if version is NULL (NO VALUE)
                if let Some(pgls_query::NodeEnum::AConst(ac)) = n.args[1].node.as_ref() {
                    if ac.isnull {
                        e.token(TokenKind::IDENT("NO VALUE".to_string()));
                    } else {
                        super::emit_node(&n.args[1], e);
                    }
                } else {
                    super::emit_node(&n.args[1], e);
                }
            }
            // Handle standalone option if there's a 3rd arg
            // XML_STANDALONE_YES = 0, XML_STANDALONE_NO = 1,
            // XML_STANDALONE_NO_VALUE = 2, XML_STANDALONE_OMITTED = 3
            if n.args.len() > 2 {
                if let Some(pgls_query::NodeEnum::AConst(ac)) = n.args[2].node.as_ref() {
                    if let Some(pgls_query::protobuf::a_const::Val::Ival(ref ival)) = ac.val {
                        match ival.ival {
                            0 => {
                                // XML_STANDALONE_YES
                                e.token(TokenKind::COMMA);
                                e.space();
                                e.token(TokenKind::IDENT("STANDALONE YES".to_string()));
                            }
                            1 => {
                                // XML_STANDALONE_NO
                                e.token(TokenKind::COMMA);
                                e.space();
                                e.token(TokenKind::IDENT("STANDALONE NO".to_string()));
                            }
                            2 => {
                                // XML_STANDALONE_NO_VALUE
                                e.token(TokenKind::COMMA);
                                e.space();
                                e.token(TokenKind::IDENT("STANDALONE NO VALUE".to_string()));
                            }
                            // 3 = XML_STANDALONE_OMITTED - don't emit anything
                            _ => {}
                        }
                    }
                }
            }
            e.token(TokenKind::R_PAREN);
        }
        7 => {
            // XMLSERIALIZE - this is handled by XmlSerialize node, not XmlExpr
            // But we'll handle it here just in case
            e.token(TokenKind::IDENT("XMLSERIALIZE".to_string()));
            e.token(TokenKind::L_PAREN);
            match n.xmloption() {
                XmlOptionType::XmloptionDocument => {
                    e.token(TokenKind::IDENT("DOCUMENT".to_string()));
                }
                _ => {
                    e.token(TokenKind::IDENT("CONTENT".to_string()));
                }
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
