use pgls_query::protobuf::{AlterOwnerStmt, ObjectType};

use super::prelude::*;

pub(super) fn emit_alter_owner_stmt(e: &mut EventEmitter, n: &AlterOwnerStmt) {
    e.group_start();

    e.token(TokenKind::ALTER_KW);
    e.space();

    let object_type = ObjectType::try_from(n.object_type).unwrap_or(ObjectType::Undefined);
    emit_keywords(e, object_type_keywords(object_type).unwrap_or("TABLE"));

    match object_type {
        ObjectType::ObjectOpfamily | ObjectType::ObjectOpclass => {
            if let Some(ref object) = n.object {
                e.space();
                match operator_class_parts(object) {
                    Some((method, name)) => emit_operator_class_name(e, method, name),
                    None => emit_owner_object(e, object),
                }
            }
        }
        ObjectType::ObjectAggregate => {
            // Aggregate needs (*) for "any argument types"
            if let Some(ref object) = n.object {
                e.space();
                if let Some(NodeEnum::ObjectWithArgs(owa)) = object.node.as_ref() {
                    super::emit_object_with_args_as(
                        e,
                        owa,
                        super::ObjectArguments::AggregateSignature,
                    );
                } else {
                    emit_owner_object(e, object);
                }
            }
        }
        _ => {
            if let Some(ref relation) = n.relation {
                e.space();
                super::emit_range_var(e, relation);
            } else if let Some(ref object) = n.object {
                e.space();
                emit_owner_object(e, object);
            }
        }
    }

    e.line(crate::emitter::LineType::SoftOrSpace);
    e.token(TokenKind::OWNER_KW);
    e.space();
    e.token(TokenKind::TO_KW);

    if let Some(ref newowner) = n.newowner {
        e.space();
        super::emit_role_spec(e, newowner);
    }

    e.group_end();
}

fn emit_owner_object(e: &mut EventEmitter, object: &pgls_query::Node) {
    match &object.node {
        Some(NodeEnum::List(list)) => emit_dot_separated_list(e, &list.items),
        _ => emit_node(e, object),
    }
}
