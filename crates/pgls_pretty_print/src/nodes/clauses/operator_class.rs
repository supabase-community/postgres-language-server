//! The name of an operator class or family in DDL: `schema.name USING btree`.

use crate::nodes::prelude::*;

/// The access method and the name of an operator class or family, which the parser stores as one
/// list: the access method first, then the parts of the name.
pub(crate) fn operator_class_parts(object: &Node) -> Option<(&Node, &[Node])> {
    match object.node.as_ref() {
        Some(NodeEnum::List(list)) if list.items.len() >= 2 => list.items.split_first(),
        _ => None,
    }
}

/// Emits `schema.name USING method`.
pub(crate) fn emit_operator_class_name(e: &mut EventEmitter, method: &Node, name: &[Node]) {
    emit_dot_separated_list(e, name);
    e.space();
    e.token(TokenKind::USING_KW);
    e.space();
    emit_node(e, method);
}
