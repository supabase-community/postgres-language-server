use pgt_query::protobuf::BitString;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_bitstring(e: &mut EventEmitter, n: &BitString) {
    e.group_start(GroupKind::BitString);
    // The bsval contains the bit string value including any prefix
    // For binary strings: "b..." or "B..."
    // For hex strings: "x..." or "X..."
    if n.bsval.starts_with("b'")
        || n.bsval.starts_with("B'")
        || n.bsval.starts_with("x'")
        || n.bsval.starts_with("X'")
    {
        e.token(TokenKind::STRING(n.bsval.to_uppercase()));
    } else if n.bsval.starts_with('b') || n.bsval.starts_with('B') {
        // Handle binary without quotes
        let digits = &n.bsval[1..];
        e.token(TokenKind::STRING(format!("B'{}'", digits)));
    } else if n.bsval.starts_with('x') || n.bsval.starts_with('X') {
        // Handle hex without quotes
        let digits = &n.bsval[1..];
        e.token(TokenKind::STRING(format!("X'{}'", digits)));
    } else {
        // Default to binary if no prefix
        e.token(TokenKind::STRING(format!("B'{}'", n.bsval)));
    }
    e.group_end();
}
