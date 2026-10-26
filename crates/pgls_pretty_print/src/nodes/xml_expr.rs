use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
    nodes::node_list::{emit_comma_separated_list, emit_parenthesized_list},
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
            e.token(TokenKind::XMLCONCAT_KW);
            emit_parenthesized_list(e, &n.args, super::emit_node);
        }
        2 => {
            // XMLELEMENT
            e.token(TokenKind::XMLELEMENT_KW);
            super::open_call_arguments(e);
            e.token(TokenKind::NAME_KW);
            e.space();
            // Names with special characters need quoting
            super::emit_identifier_maybe_quoted(e, &n.name);

            // Named args represent XML attributes (XMLATTRIBUTES)
            if !n.named_args.is_empty() {
                e.token(TokenKind::COMMA);
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::XMLATTRIBUTES_KW);
                emit_parenthesized_list(e, &n.named_args, super::emit_node);
            }

            // Args represent content
            if !n.args.is_empty() {
                e.token(TokenKind::COMMA);
                e.line(LineType::SoftOrSpace);
                emit_comma_separated_list(e, &n.args, super::emit_node);
            }

            super::close_call_arguments(e);
        }
        3 => {
            // XMLFOREST
            e.token(TokenKind::XMLFOREST_KW);
            emit_parenthesized_list(e, &n.named_args, super::emit_node);
        }
        4 => {
            // XMLPARSE
            e.token(TokenKind::XMLPARSE_KW);
            super::open_call_arguments(e);
            // XmlOptionType: XmloptionDocument=1, XmloptionContent=2
            match n.xmloption() {
                XmlOptionType::XmloptionDocument => {
                    e.token(TokenKind::DOCUMENT_KW);
                }
                _ => {
                    e.token(TokenKind::CONTENT_KW);
                }
            }
            e.space();
            if !n.args.is_empty() {
                super::emit_node(&n.args[0], e);
            }
            super::close_call_arguments(e);
        }
        5 => {
            // XMLPI
            e.token(TokenKind::XMLPI_KW);
            super::open_call_arguments(e);
            e.token(TokenKind::NAME_KW);
            e.space();
            // Names with special characters need quoting
            super::emit_identifier_maybe_quoted(e, &n.name);

            if !n.args.is_empty() {
                e.token(TokenKind::COMMA);
                e.line(LineType::SoftOrSpace);
                super::emit_node(&n.args[0], e);
            }

            super::close_call_arguments(e);
        }
        6 => {
            // XMLROOT
            e.token(TokenKind::XMLROOT_KW);
            super::open_call_arguments(e);
            if !n.args.is_empty() {
                super::emit_node(&n.args[0], e);
            }
            e.token(TokenKind::COMMA);
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::VERSION_KW);
            e.space();
            if n.args.len() > 1 {
                // Check if version is NULL (NO VALUE)
                if let Some(pgls_query::NodeEnum::AConst(ac)) = n.args[1].node.as_ref() {
                    if ac.isnull {
                        e.token(TokenKind::NO_KW);
                        e.space();
                        e.token(TokenKind::VALUE_KW);
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
            if n.args.len() > 2
                && let Some(pgls_query::NodeEnum::AConst(ac)) = n.args[2].node.as_ref()
                && let Some(pgls_query::protobuf::a_const::Val::Ival(ref ival)) = ac.val
            {
                match ival.ival {
                    0 => {
                        // XML_STANDALONE_YES
                        e.token(TokenKind::COMMA);
                        e.line(LineType::SoftOrSpace);
                        e.token(TokenKind::STANDALONE_KW);
                        e.space();
                        e.token(TokenKind::YES_KW);
                    }
                    1 => {
                        // XML_STANDALONE_NO
                        e.token(TokenKind::COMMA);
                        e.line(LineType::SoftOrSpace);
                        e.token(TokenKind::STANDALONE_KW);
                        e.space();
                        e.token(TokenKind::NO_KW);
                    }
                    2 => {
                        // XML_STANDALONE_NO_VALUE
                        e.token(TokenKind::COMMA);
                        e.line(LineType::SoftOrSpace);
                        e.token(TokenKind::STANDALONE_KW);
                        e.space();
                        e.token(TokenKind::NO_KW);
                        e.space();
                        e.token(TokenKind::VALUE_KW);
                    }
                    // 3 = XML_STANDALONE_OMITTED - don't emit anything
                    _ => {}
                }
            }
            super::close_call_arguments(e);
        }
        7 => {
            // XMLSERIALIZE - this is handled by XmlSerialize node, not XmlExpr
            // But we'll handle it here just in case
            e.token(TokenKind::XMLSERIALIZE_KW);
            super::open_call_arguments(e);
            match n.xmloption() {
                XmlOptionType::XmloptionDocument => {
                    e.token(TokenKind::DOCUMENT_KW);
                }
                _ => {
                    e.token(TokenKind::CONTENT_KW);
                }
            }
            e.space();
            if !n.args.is_empty() {
                emit_comma_separated_list(e, &n.args, super::emit_node);
            }
            super::close_call_arguments(e);
        }
        8 => {
            // IS DOCUMENT - expr IS DOCUMENT
            if !n.args.is_empty() {
                super::emit_node(&n.args[0], e);
            }
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::IS_KW);
            e.space();
            e.token(TokenKind::DOCUMENT_KW);
        }
        _ => {
            // Unknown/Undefined XML operation - emit XMLCOMMENT as fallback
            e.token(TokenKind::IDENT("xmlcomment".to_string()));
            super::open_call_arguments(e);
            if !n.args.is_empty() {
                super::emit_node(&n.args[0], e);
            }
            super::close_call_arguments(e);
        }
    }

    e.group_end();
}
