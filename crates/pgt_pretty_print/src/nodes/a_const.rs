use pgt_query::protobuf::AConst;
use pgt_query::protobuf::a_const::Val;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_a_const(e: &mut EventEmitter, n: &AConst) {
    e.group_start(GroupKind::AConst);

    if n.isnull {
        e.token(TokenKind::NULL_KW);
    } else if let Some(ref val) = n.val {
        emit_val(e, val);
    } else {
        unreachable!("AConst must have either isnull=true or a val");
    }

    e.group_end();
}

fn emit_val(e: &mut EventEmitter, n: &Val) {
    match n {
        Val::Ival(integer) => {
            super::emit_integer(e, integer);
        }
        Val::Fval(float) => {
            super::emit_float(e, float);
        }
        Val::Boolval(boolean) => {
            super::emit_boolean(e, boolean);
        }
        Val::Sval(string) => {
            super::emit_string_literal(e, string);
        }
        Val::Bsval(bsval) => {
            super::emit_bitstring(e, bsval);
        }
    }
}
