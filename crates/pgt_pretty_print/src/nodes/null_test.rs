use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};
use pgt_query::protobuf::NullTest;

pub(super) fn emit_null_test(e: &mut EventEmitter, n: &NullTest) {
    e.group_start(GroupKind::NullTest);

    // Emit the expression being tested
    if let Some(ref arg) = n.arg {
        super::emit_node(arg, e);
    }

    e.space();

    // Emit IS [NOT] NULL
    e.token(TokenKind::IS_KW);
    e.space();

    // nulltesttype: 0 = Undefined, 1 = IS_NULL, 2 = IS_NOT_NULL
    if n.nulltesttype == 2 {
        e.token(TokenKind::NOT_KW);
        e.space();
    }

    e.token(TokenKind::NULL_KW);

    e.group_end();
}
