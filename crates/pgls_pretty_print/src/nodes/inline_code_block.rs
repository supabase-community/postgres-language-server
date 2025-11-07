use pgls_query::protobuf::InlineCodeBlock;

use crate::emitter::{EventEmitter, GroupKind};

pub(super) fn emit_inline_code_block(e: &mut EventEmitter, n: &InlineCodeBlock) {
    e.group_start(GroupKind::InlineCodeBlock);

    if !n.source_text.is_empty() {
        super::string::emit_dollar_quoted_str(e, &n.source_text);
    }

    e.group_end();
}
