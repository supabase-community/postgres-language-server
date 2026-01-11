use pgls_query::protobuf::InlineCodeBlock;

use crate::emitter::{EventEmitter, GroupKind};

use super::string::{DollarQuoteHint, emit_dollar_quoted_str_with_hint};

pub(super) fn emit_inline_code_block(e: &mut EventEmitter, n: &InlineCodeBlock) {
    e.group_start(GroupKind::InlineCodeBlock);

    if !n.source_text.is_empty() {
        // InlineCodeBlock is typically used for anonymous code blocks (like DO)
        emit_dollar_quoted_str_with_hint(e, &n.source_text, DollarQuoteHint::Do);
    }

    e.group_end();
}
