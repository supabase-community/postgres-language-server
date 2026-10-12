use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};
use pgls_query::protobuf::{XmlOptionType, XmlSerialize};

pub(super) fn emit_xml_serialize(e: &mut EventEmitter, n: &XmlSerialize) {
    e.group_start(GroupKind::XmlSerialize);

    super::emit_keyword(e, "xmlserialize");
    e.token(TokenKind::L_PAREN);

    // XmlOptionType: XmloptionDocument=1, XmloptionContent=2
    match n.xmloption() {
        XmlOptionType::XmloptionDocument => {
            super::emit_keyword(e, "document");
            e.space();
        }
        _ => {
            super::emit_keyword(e, "content");
            e.space();
        }
    }

    // Expression to serialize
    if let Some(ref expr) = n.expr {
        super::emit_node(expr, e);
    }

    // AS type
    if let Some(ref type_name) = n.type_name {
        e.space();
        e.token(TokenKind::AS_KW);
        e.space();
        super::emit_type_name(e, type_name);
    }

    // INDENT option
    if n.indent {
        e.space();
        super::emit_keyword(e, "indent");
    }

    e.token(TokenKind::R_PAREN);

    e.group_end();
}
