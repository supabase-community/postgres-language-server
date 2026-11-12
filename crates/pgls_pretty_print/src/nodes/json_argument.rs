use pgls_query::protobuf::JsonArgument;

use super::prelude::*;

pub(super) fn emit_json_argument(e: &mut EventEmitter, argument: &JsonArgument) {
    e.group_start();

    if let Some(value) = argument.val.as_ref() {
        super::emit_json_value_expr(e, value);
    }

    if !argument.name.is_empty() {
        e.space();
        e.token(TokenKind::AS_KW);
        e.space();
        emit_identifier_maybe_quoted(e, &argument.name);
    }

    e.group_end();
}
