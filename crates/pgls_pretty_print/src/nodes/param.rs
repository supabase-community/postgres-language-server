use pgls_query::protobuf::{Param, ParamKind};

use crate::emitter::{EventEmitter, GroupKind};

pub(super) fn emit_param(e: &mut EventEmitter, n: &Param) {
    e.group_start(GroupKind::Param);

    let kind = match n.paramkind() {
        ParamKind::ParamExtern => "extern",
        ParamKind::ParamExec => "exec",
        ParamKind::ParamSublink => "sublink",
        ParamKind::ParamMultiexpr => "multiexpr",
        ParamKind::Undefined => "unknown",
    };

    let repr = format!("param#{}:{}", kind, n.paramid);
    super::emit_identifier(e, &repr);

    e.group_end();
}
