use pgls_query::protobuf::JsonOutput;

use super::prelude::*;

pub(super) fn emit_json_output(e: &mut EventEmitter, n: &JsonOutput) {
    e.group_start();

    e.token(TokenKind::RETURNING_KW);

    if let Some(ref type_name) = n.type_name {
        e.space();
        super::emit_type_name(e, type_name);
    }

    if let Some(ref returning) = n.returning {
        let mut has_content = true;
        emit_json_returning_clause(e, returning, &mut has_content);
    }

    e.group_end();
}
