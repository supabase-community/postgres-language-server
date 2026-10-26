use pgls_query::protobuf::RtePermissionInfo;

use super::prelude::*;

pub(super) fn emit_rtepermission_info(e: &mut EventEmitter, n: &RtePermissionInfo) {
    e.group_start(GroupKind::RtepermissionInfo);

    emit_identifier(e, &format!("rteperm#{}", n.relid));

    if !n.inh {
        e.space();
        emit_identifier(e, "no_inherit");
    }

    if n.required_perms != 0 {
        e.space();
        emit_identifier(e, &format!("perms#{}", n.required_perms));
    }

    if n.check_as_user != 0 {
        e.space();
        emit_identifier(e, &format!("checkuser#{}", n.check_as_user));
    }

    if !n.selected_cols.is_empty() {
        e.space();
        emit_identifier(e, &format!("selected_cols#{}", n.selected_cols.len()));
    }

    if !n.inserted_cols.is_empty() {
        e.space();
        emit_identifier(e, &format!("inserted_cols#{}", n.inserted_cols.len()));
    }

    if !n.updated_cols.is_empty() {
        e.space();
        emit_identifier(e, &format!("updated_cols#{}", n.updated_cols.len()));
    }

    e.group_end();
}
