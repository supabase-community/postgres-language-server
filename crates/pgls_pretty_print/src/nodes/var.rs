use pgls_query::protobuf::Var;

use crate::emitter::{EventEmitter, GroupKind};

pub(super) fn emit_var(e: &mut EventEmitter, n: &Var) {
    e.group_start(GroupKind::Var);

    let repr = if n.varlevelsup == 0 {
        format!("var#{}.{}", n.varno, n.varattno)
    } else {
        format!("var#{}^{}.{}", n.varno, n.varlevelsup, n.varattno)
    };
    super::emit_identifier(e, &repr);

    e.group_end();
}
