use pgls_query::protobuf::{DropStmt, ObjectType};

use super::prelude::*;

pub(super) fn emit_drop_stmt(e: &mut EventEmitter, n: &DropStmt) {
    e.group_start();

    e.token(TokenKind::DROP_KW);
    e.space();

    // Object type - use typed enum accessor
    let object_type = n.remove_type();
    emit_keywords(e, object_type_keywords(object_type).unwrap_or("TABLE"));

    // CONCURRENTLY for indexes
    if n.concurrent && object_type == ObjectType::ObjectIndex {
        e.space();
        e.token(TokenKind::CONCURRENTLY_KW);
    }

    // IF EXISTS
    if n.missing_ok {
        e.space();
        emit_if_exists(e);
    }

    // Object names - indent list when it breaks to multiple lines
    if !n.objects.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.indent_start();
        match object_type {
            ObjectType::ObjectCast => {
                emit_comma_separated_list(e, &n.objects, emit_drop_cast_object);
            }
            ObjectType::ObjectOpclass | ObjectType::ObjectOpfamily => {
                // DROP OPERATOR CLASS/FAMILY uses: name USING access_method
                emit_comma_separated_list(e, &n.objects, emit_drop_opclass_object);
            }
            ObjectType::ObjectRule | ObjectType::ObjectTrigger | ObjectType::ObjectPolicy => {
                // DROP RULE/TRIGGER/POLICY uses: name ON table_name
                emit_comma_separated_list(e, &n.objects, emit_drop_on_object);
            }
            ObjectType::ObjectAggregate => {
                // DROP AGGREGATE needs (*) for "any argument types"
                emit_comma_separated_list(e, &n.objects, |e, node| {
                    if let Some(pgls_query::NodeEnum::ObjectWithArgs(owa)) = node.node.as_ref() {
                        super::emit_object_with_args_as(
                            e,
                            owa,
                            super::ObjectArguments::AggregateSignature,
                        );
                    } else {
                        emit_node(e, node);
                    }
                });
            }
            _ => {
                emit_comma_separated_list(e, &n.objects, |e, node| {
                    if let Some(pgls_query::NodeEnum::List(list)) = node.node.as_ref() {
                        emit_dot_separated_identifiers(e, &list.items);
                    } else {
                        emit_node(e, node);
                    }
                });
            }
        }
        e.indent_end();
    }

    emit_drop_behavior(e, n.behavior(), Separator::Space, Restrict::Omitted);

    e.group_end();
}

fn emit_dot_separated_identifiers(e: &mut EventEmitter, items: &[pgls_query::protobuf::Node]) {
    emit_dot_separated_list_with(e, items, |e, item| {
        if let Some(pgls_query::NodeEnum::String(s)) = item.node.as_ref() {
            emit_identifier_maybe_quoted(e, &s.sval);
        } else {
            emit_node(e, item);
        }
    });
}

fn emit_drop_cast_object(e: &mut EventEmitter, node: &pgls_query::protobuf::Node) {
    if let Some(pgls_query::NodeEnum::List(list)) = node.node.as_ref()
        && list.items.len() == 2
    {
        e.token(TokenKind::L_PAREN);
        emit_node(e, &list.items[0]);
        e.space();
        e.token(TokenKind::AS_KW);
        e.space();
        emit_node(e, &list.items[1]);
        e.token(TokenKind::R_PAREN);
        return;
    }

    // Fallback for unexpected structure
    emit_node(e, node);
}

/// Emit DROP OPERATOR CLASS/FAMILY object.
/// Format: name USING access_method
/// The list is [access_method, name_parts...]
fn emit_drop_opclass_object(e: &mut EventEmitter, node: &pgls_query::protobuf::Node) {
    if let Some(pgls_query::NodeEnum::List(list)) = node.node.as_ref()
        && list.items.len() >= 2
    {
        // First element is the access method name
        // Remaining elements are the operator class/family name parts
        let name_parts = &list.items[1..];
        emit_dot_separated_identifiers(e, name_parts);
        e.space();
        e.token(TokenKind::USING_KW);
        e.space();
        if let Some(pgls_query::NodeEnum::String(s)) = list.items[0].node.as_ref() {
            emit_identifier_maybe_quoted(e, &s.sval);
        }
        return;
    }

    // Fallback for unexpected structure
    emit_node(e, node);
}

/// Emit DROP RULE/TRIGGER/POLICY object.
/// Format: name ON table_name
/// The list is [table_name_parts..., object_name] - last item is the object name
fn emit_drop_on_object(e: &mut EventEmitter, node: &pgls_query::protobuf::Node) {
    if let Some(pgls_query::NodeEnum::List(list)) = node.node.as_ref()
        && list.items.len() >= 2
    {
        // Last element is the object name (trigger/rule/policy)
        // All previous elements are the table name parts (schema, table, etc.)
        let (table_parts, object_name_item) = list.items.split_at(list.items.len() - 1);
        let object_name = &object_name_item[0];

        // Emit object name first
        if let Some(pgls_query::NodeEnum::String(s)) = object_name.node.as_ref() {
            emit_identifier_maybe_quoted(e, &s.sval);
        } else {
            emit_node(e, object_name);
        }

        e.space();
        e.token(TokenKind::ON_KW);
        e.space();

        // Emit table name parts dot-separated
        emit_dot_separated_identifiers(e, table_parts);
        return;
    }

    // Fallback for unexpected structure
    emit_node(e, node);
}
