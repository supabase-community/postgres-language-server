use pgls_query::protobuf::InlineCodeBlock;

use crate::emitter::{EventEmitter, GroupKind};

use super::string::emit_body_literal;

pub(super) fn emit_inline_code_block(e: &mut EventEmitter, n: &InlineCodeBlock) {
    e.group_start(GroupKind::InlineCodeBlock);

    if !n.source_text.is_empty() {
        // InlineCodeBlock is typically used for anonymous code blocks (like DO)
        emit_body_literal(e, &n.source_text);
    }

    e.group_end();
}
