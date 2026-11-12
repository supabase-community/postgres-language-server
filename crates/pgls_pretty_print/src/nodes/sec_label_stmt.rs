use pgls_query::protobuf::{ObjectType, SecLabelStmt};

use super::prelude::*;

pub(super) fn emit_sec_label_stmt(e: &mut EventEmitter, n: &SecLabelStmt) {
    e.group_start();

    emit_keyword(e, "SECURITY");
    e.space();
    emit_keyword(e, "LABEL");

    // Emit FOR provider if present
    if !n.provider.is_empty() {
        e.space();
        e.token(TokenKind::FOR_KW);
        e.space();
        emit_identifier_maybe_quoted(e, &n.provider);
    }

    // Emit ON object_type object
    e.line(LineType::SoftOrSpace);
    e.token(TokenKind::ON_KW);
    e.space();

    // Map object type to SQL keyword
    // SECURITY LABEL names a foreign server `FOREIGN SERVER`.
    let keywords = match n.objtype() {
        ObjectType::ObjectForeignServer => "FOREIGN SERVER",
        objtype => object_type_keywords(objtype).unwrap_or("TABLE"),
    };
    emit_keywords(e, keywords);

    e.space();

    // Emit object name
    if let Some(ref object) = n.object {
        // Aggregate needs (*) for "any argument types"
        if n.objtype() == ObjectType::ObjectAggregate {
            if let Some(NodeEnum::ObjectWithArgs(owa)) = object.node.as_ref() {
                super::emit_object_with_args_as(e, owa, super::ObjectArguments::AggregateSignature);
            } else {
                emit_node(e, object);
            }
        } else {
            emit_node(e, object);
        }
    }

    // Emit IS 'label'
    e.line(LineType::SoftOrSpace);
    e.token(TokenKind::IS_KW);
    e.space();
    emit_single_quoted_str(e, &n.label);

    e.group_end();
}
