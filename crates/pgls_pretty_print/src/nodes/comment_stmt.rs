use pgls_query::protobuf::{CommentStmt, ObjectType};

use super::prelude::*;
use super::{ObjectArguments, emit_object_with_args_as};

pub(super) fn emit_comment_stmt(e: &mut EventEmitter, n: &CommentStmt) {
    e.group_start();

    emit_keyword(e, "COMMENT");
    e.space();
    e.token(TokenKind::ON_KW);
    e.space();

    let object_type = n.objtype();
    emit_keywords(e, object_type_keywords(object_type).unwrap_or("OBJECT"));
    e.space();

    // Object name
    if let Some(ref object) = n.object {
        // For COLUMN, the object is a list of identifiers that should be dot-separated
        // e.g., "table.column" not "table, column"
        if object_type == ObjectType::ObjectColumn {
            if let Some(pgls_query::NodeEnum::List(list)) = object.node.as_ref() {
                emit_dot_separated_list(e, &list.items);
            } else {
                emit_node(e, object);
            }
        } else if object_type == ObjectType::ObjectAggregate {
            // An aggregate needs (*) for any arguments
            if let Some(NodeEnum::ObjectWithArgs(owa)) = object.node.as_ref() {
                emit_object_with_args_as(e, owa, ObjectArguments::AggregateSignature);
            } else {
                emit_node(e, object);
            }
        } else if object_type == ObjectType::ObjectDomconstraint {
            // Format: CONSTRAINT constraint_name ON DOMAIN domain_name
            // The list has [TypeName(domain_name), constraint_name]
            if let Some(NodeEnum::List(list)) = object.node.as_ref() {
                if list.items.len() == 2 {
                    // Emit constraint name first (second element)
                    emit_node(e, &list.items[1]);
                    e.line(LineType::SoftOrSpace);
                    e.token(TokenKind::ON_KW);
                    e.space();
                    emit_keyword(e, "DOMAIN");
                    e.space();
                    // Then domain name (first element)
                    emit_node(e, &list.items[0]);
                } else {
                    emit_node(e, object);
                }
            } else {
                emit_node(e, object);
            }
        } else if object_type == ObjectType::ObjectTabconstraint {
            // Format: CONSTRAINT constraint_name ON table_name
            // The list has [table_name, constraint_name]
            if let Some(NodeEnum::List(list)) = object.node.as_ref() {
                if list.items.len() == 2 {
                    // Emit constraint name first (second element)
                    emit_node(e, &list.items[1]);
                    e.line(LineType::SoftOrSpace);
                    e.token(TokenKind::ON_KW);
                    e.space();
                    // Then table name (first element)
                    emit_node(e, &list.items[0]);
                } else {
                    emit_node(e, object);
                }
            } else {
                emit_node(e, object);
            }
        } else if object_type == ObjectType::ObjectRule {
            // Format: RULE rule_name ON table_name
            // The list has [table_name, rule_name]
            if let Some(NodeEnum::List(list)) = object.node.as_ref() {
                if list.items.len() == 2 {
                    // Emit rule name first (second element)
                    emit_node(e, &list.items[1]);
                    e.line(LineType::SoftOrSpace);
                    e.token(TokenKind::ON_KW);
                    e.space();
                    // Then table name (first element)
                    emit_node(e, &list.items[0]);
                } else {
                    emit_node(e, object);
                }
            } else {
                emit_node(e, object);
            }
        } else if object_type == ObjectType::ObjectTrigger {
            // Format: TRIGGER trigger_name ON table_name
            // The list has [table_name, trigger_name]
            if let Some(NodeEnum::List(list)) = object.node.as_ref() {
                if list.items.len() == 2 {
                    // Emit trigger name first (second element)
                    emit_node(e, &list.items[1]);
                    e.line(LineType::SoftOrSpace);
                    e.token(TokenKind::ON_KW);
                    e.space();
                    // Then table name (first element)
                    emit_node(e, &list.items[0]);
                } else {
                    emit_node(e, object);
                }
            } else {
                emit_node(e, object);
            }
        } else if object_type == ObjectType::ObjectPolicy {
            // Format: POLICY policy_name ON table_name
            // The list has [table_name, policy_name]
            if let Some(NodeEnum::List(list)) = object.node.as_ref() {
                if list.items.len() == 2 {
                    // Emit policy name first (second element)
                    emit_node(e, &list.items[1]);
                    e.line(LineType::SoftOrSpace);
                    e.token(TokenKind::ON_KW);
                    e.space();
                    // Then table name (first element)
                    emit_node(e, &list.items[0]);
                } else {
                    emit_node(e, object);
                }
            } else {
                emit_node(e, object);
            }
        } else if object_type == ObjectType::ObjectUserMapping {
            // A user mapping needs FOR before the role name
            // COMMENT ON USER MAPPING FOR role_name SERVER server_name IS 'comment'
            e.token(TokenKind::FOR_KW);
            e.space();
            emit_node(e, object);
        } else {
            // For most object types (TABLE, INDEX, SEQUENCE, VIEW, etc.), the object
            // is a List of String nodes representing a schema-qualified name that must
            // be dot-separated (e.g., public.my_table), not comma-separated.
            if let Some(pgls_query::NodeEnum::List(list)) = object.node.as_ref() {
                emit_dot_separated_list(e, &list.items);
            } else {
                emit_node(e, object);
            }
        }
    }

    // A comment that doesn't fit goes on its own line, one level in.
    e.indent_start();
    e.line(LineType::SoftOrSpace);
    e.token(TokenKind::IS_KW);
    e.space();

    // Comment text
    if n.comment.is_empty() {
        e.token(TokenKind::NULL_KW);
    } else {
        emit_single_quoted_str(e, &n.comment);
    }
    e.indent_end();

    e.group_end();
}
