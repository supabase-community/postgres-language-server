use pgls_query::protobuf::{AlterObjectSchemaStmt, ObjectType};

use super::prelude::*;

pub(super) fn emit_alter_object_schema_stmt(e: &mut EventEmitter, n: &AlterObjectSchemaStmt) {
    e.group_start();

    e.token(TokenKind::ALTER_KW);
    e.space();

    // Emit object type
    let object_type = n.object_type();
    match object_type_keywords(object_type) {
        Some(keywords) => emit_keywords(e, keywords),
        None => {
            debug_assert!(
                false,
                "Unhandled ObjectType in AlterObjectSchemaStmt: {object_type:?}"
            );
            e.token(TokenKind::IDENT("UNKNOWN".to_string()));
        }
    }
    e.space();

    if n.missing_ok {
        emit_if_exists(e);
        e.space();
    }

    // Emit object name
    if let Some(ref relation) = n.relation {
        super::emit_range_var(e, relation);
    } else if let Some(ref object) = n.object {
        match n.object_type() {
            ObjectType::ObjectOpclass | ObjectType::ObjectOpfamily => {
                match operator_class_parts(object) {
                    Some((method, name)) => emit_operator_class_name(e, method, name),
                    None => emit_node(e, object),
                }
            }
            ObjectType::ObjectAggregate => {
                // Aggregate needs (*) for "any argument types"
                if let Some(NodeEnum::ObjectWithArgs(owa)) = object.node.as_ref() {
                    super::emit_object_with_args_as(
                        e,
                        owa,
                        super::ObjectArguments::AggregateSignature,
                    );
                } else {
                    emit_node(e, object);
                }
            }
            _ => match &object.node {
                // Qualified names (`any_name`) arrive as a list of strings
                Some(NodeEnum::List(list)) => emit_dot_separated_list(e, &list.items),
                _ => emit_node(e, object),
            },
        }
    }

    // Emit new schema
    if !n.newschema.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::SET_KW);
        e.space();
        e.token(TokenKind::SCHEMA_KW);
        e.space();
        emit_identifier_maybe_quoted(e, &n.newschema);
    }

    e.group_end();
}
