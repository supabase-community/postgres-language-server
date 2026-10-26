use pgls_query::protobuf::{ObjectType, RenameStmt};

use super::emit_range_var;
use super::prelude::*;

pub(super) fn emit_rename_stmt(e: &mut EventEmitter, n: &RenameStmt) {
    e.group_start();

    e.token(TokenKind::ALTER_KW);
    e.space();

    let rename_type = ObjectType::try_from(n.rename_type).unwrap_or(ObjectType::Undefined);
    let relation_type = ObjectType::try_from(n.relation_type).unwrap_or(ObjectType::Undefined);

    // For table-related renames, use the relation_type if provided, otherwise default appropriately
    let target_type = resolve_alter_target(rename_type, relation_type);

    emit_keywords(e, object_type_keywords(target_type).unwrap_or("TABLE"));

    if n.missing_ok {
        e.space();
        emit_if_exists(e);
    }

    match rename_type {
        ObjectType::ObjectColumn => {
            emit_relation_head(e, n);
            emit_keyworded_rename(e, TokenKind::COLUMN_KW, &n.subname, &n.newname);
        }
        ObjectType::ObjectTabconstraint => {
            emit_relation_head(e, n);
            emit_keyworded_rename(e, TokenKind::CONSTRAINT_KW, &n.subname, &n.newname);
        }
        ObjectType::ObjectTrigger => {
            // ALTER TRIGGER trigger_name ON table_name RENAME TO new_name
            e.space();
            emit_identifier_maybe_quoted(e, &n.subname);
            if let Some(ref relation) = n.relation {
                e.space();
                e.token(TokenKind::ON_KW);
                e.space();
                emit_range_var(e, relation);
            }
            emit_simple_rename(e, &n.newname);
        }
        ObjectType::ObjectRule => {
            emit_relation_head(e, n);
            emit_keyworded_rename(e, TokenKind::RULE_KW, &n.subname, &n.newname);
        }
        ObjectType::ObjectPolicy => emit_policy_statement(e, n),
        ObjectType::ObjectOpfamily => {
            if !emit_operator_collection_head(e, n) {
                emit_default_head(e, n);
            }
            emit_simple_rename(e, &n.newname);
        }
        ObjectType::ObjectOpclass => {
            if !emit_operator_collection_head(e, n) {
                emit_default_head(e, n);
            }
            emit_simple_rename(e, &n.newname);
        }
        ObjectType::ObjectDomconstraint => {
            // Domain name is in object field for domain constraints
            emit_object_head(e, n);
            emit_keyworded_rename(e, TokenKind::CONSTRAINT_KW, &n.subname, &n.newname);
        }
        ObjectType::ObjectAttribute => {
            // The composite type name is stored as a RangeVar in `relation`; it is not an
            // inheritance target, so ONLY must not be emitted
            if let Some(ref relation) = n.relation {
                e.space();
                super::emit_range_var_name(e, relation);
            }
            emit_attribute_rename(e, &n.subname, &n.newname);
            emit_drop_behavior(e, n.behavior(), Separator::Space, Restrict::Omitted);
        }
        ObjectType::ObjectAggregate => {
            emit_aggregate_head(e, n);
            emit_simple_rename(e, &n.newname);
        }
        _ => {
            emit_default_head(e, n);
            emit_simple_rename(e, &n.newname);
        }
    }

    e.group_end();
}

fn resolve_alter_target(rename_type: ObjectType, relation_type: ObjectType) -> ObjectType {
    match rename_type {
        ObjectType::ObjectColumn | ObjectType::ObjectTabconstraint | ObjectType::ObjectRule => {
            // Only accept valid table-like relation types
            match relation_type {
                ObjectType::ObjectView
                | ObjectType::ObjectMatview
                | ObjectType::ObjectForeignTable => relation_type,
                _ => ObjectType::ObjectTable, // Default to TABLE for others
            }
        }
        ObjectType::ObjectTrigger => {
            // ALTER TRIGGER syntax, not ALTER TABLE
            ObjectType::ObjectTrigger
        }
        ObjectType::ObjectDomconstraint => {
            // Domain constraint renames should always be ALTER DOMAIN
            ObjectType::ObjectDomain
        }
        ObjectType::ObjectAttribute => match relation_type {
            ObjectType::Undefined => ObjectType::ObjectType,
            other => other,
        },
        other => other,
    }
}

fn emit_relation_head(e: &mut EventEmitter, n: &RenameStmt) {
    if let Some(ref relation) = n.relation {
        e.space();
        emit_range_var(e, relation);
    }
}

fn emit_object_head(e: &mut EventEmitter, n: &RenameStmt) {
    if let Some(ref object) = n.object {
        e.space();
        emit_object_name(e, object);
    }
}

fn emit_object_name(e: &mut EventEmitter, object: &pgls_query::Node) {
    match &object.node {
        // Qualified names (`any_name`) arrive as a list of strings
        Some(NodeEnum::List(list)) => emit_dot_separated_list(e, &list.items),
        _ => emit_node(e, object),
    }
}

fn emit_default_head(e: &mut EventEmitter, n: &RenameStmt) {
    if let Some(ref relation) = n.relation {
        e.space();
        emit_range_var(e, relation);
    } else if let Some(ref object) = n.object {
        e.space();
        emit_object_name(e, object);
    } else if !n.subname.is_empty() {
        e.space();
        emit_identifier_maybe_quoted(e, &n.subname);
    }
}

fn emit_aggregate_head(e: &mut EventEmitter, n: &RenameStmt) {
    if let Some(ref object) = n.object {
        e.space();
        if let Some(NodeEnum::ObjectWithArgs(owa)) = object.node.as_ref() {
            super::emit_object_with_args_as(e, owa, super::ObjectArguments::AggregateSignature);
        } else {
            emit_node(e, object);
        }
    }
}

fn emit_simple_rename(e: &mut EventEmitter, new_name: &str) {
    e.line(LineType::SoftOrSpace);
    e.token(TokenKind::RENAME_KW);
    e.space();
    e.token(TokenKind::TO_KW);
    e.space();
    emit_identifier_maybe_quoted(e, new_name);
}

fn emit_operator_collection_head(e: &mut EventEmitter, n: &RenameStmt) -> bool {
    if let Some(ref object) = n.object
        && let Some(NodeEnum::List(list)) = &object.node
        && list.items.len() >= 2
    {
        let (method_node, name_nodes) = list.items.split_first().unwrap();
        if !name_nodes.is_empty() {
            e.space();
            emit_dot_separated_list(e, name_nodes);
            e.space();
            e.token(TokenKind::USING_KW);
            e.space();
            emit_node(e, method_node);
            return true;
        }
    }

    false
}

fn emit_keyworded_rename(e: &mut EventEmitter, keyword: TokenKind, old_name: &str, new_name: &str) {
    if old_name.is_empty() {
        emit_simple_rename(e, new_name);
        return;
    }

    e.line(LineType::SoftOrSpace);
    e.token(TokenKind::RENAME_KW);
    e.space();
    e.token(keyword);
    e.space();
    emit_identifier_maybe_quoted(e, old_name);
    e.line(LineType::SoftOrSpace);
    e.token(TokenKind::TO_KW);
    e.space();
    emit_identifier_maybe_quoted(e, new_name);
}

fn emit_policy_statement(e: &mut EventEmitter, n: &RenameStmt) {
    if !n.subname.is_empty() {
        e.space();
        emit_identifier_maybe_quoted(e, &n.subname);
    }

    if let Some(ref relation) = n.relation {
        e.space();
        e.token(TokenKind::ON_KW);
        e.space();
        emit_range_var(e, relation);
    }

    emit_simple_rename(e, &n.newname);
}

fn emit_attribute_rename(e: &mut EventEmitter, old_name: &str, new_name: &str) {
    e.line(LineType::SoftOrSpace);
    e.token(TokenKind::RENAME_KW);
    e.space();
    e.token(TokenKind::ATTRIBUTE_KW);
    e.space();
    emit_identifier_maybe_quoted(e, old_name);
    e.space();
    e.token(TokenKind::TO_KW);
    e.space();
    emit_identifier_maybe_quoted(e, new_name);
}
