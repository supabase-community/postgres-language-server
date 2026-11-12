use pgls_query::protobuf::Var;

use super::prelude::*;

pub(super) fn emit_var(e: &mut EventEmitter, n: &Var) {
    e.group_start();

    let repr = if n.varlevelsup == 0 {
        format!("var#{}.{}", n.varno, n.varattno)
    } else {
        format!("var#{}^{}.{}", n.varno, n.varlevelsup, n.varattno)
    };
    emit_identifier(e, &repr);

    e.group_end();
}
