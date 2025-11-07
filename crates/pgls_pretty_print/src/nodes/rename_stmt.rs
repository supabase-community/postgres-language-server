use pgls_query::{
    NodeEnum,
    protobuf::{ObjectType, RenameStmt},
};

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};

use super::{
    emit_identifier_maybe_quoted, emit_node, emit_range_var, node_list::emit_dot_separated_list,
};

pub(super) fn emit_rename_stmt(e: &mut EventEmitter, n: &RenameStmt) {
    e.group_start(GroupKind::RenameStmt);

    e.token(TokenKind::ALTER_KW);
    e.space();

    let rename_type = ObjectType::try_from(n.rename_type).unwrap_or(ObjectType::Undefined);
    let relation_type = ObjectType::try_from(n.relation_type).unwrap_or(ObjectType::Undefined);

    // For table-related renames with an actual relation, use the relation to determine the type
    let target_type = if n.relation.is_some() {
        match rename_type {
            ObjectType::ObjectColumn
            | ObjectType::ObjectTabconstraint
            | ObjectType::ObjectTrigger
            | ObjectType::ObjectRule => ObjectType::ObjectTable,
            ObjectType::ObjectDomconstraint => ObjectType::ObjectDomain,
            ObjectType::ObjectAttribute => ObjectType::ObjectType,
            _ => resolve_alter_target(rename_type, relation_type),
        }
    } else {
        resolve_alter_target(rename_type, relation_type)
    };

    emit_object_type(e, target_type);

    if n.missing_ok {
        e.space();
        e.token(TokenKind::IF_KW);
        e.space();
        e.token(TokenKind::EXISTS_KW);
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
            emit_relation_head(e, n);
            emit_keyworded_rename(e, TokenKind::TRIGGER_KW, &n.subname, &n.newname);
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
            emit_relation_head(e, n);
            emit_keyworded_rename(e, TokenKind::CONSTRAINT_KW, &n.subname, &n.newname);
        }
        ObjectType::ObjectAttribute => {
            emit_object_head(e, n);
            emit_attribute_rename(e, &n.subname, &n.newname);
        }
        _ => {
            emit_default_head(e, n);
            emit_simple_rename(e, &n.newname);
        }
    }

    e.token(TokenKind::SEMICOLON);
    e.group_end();
}

fn emit_object_type(e: &mut EventEmitter, object_type: ObjectType) {
    match object_type {
        ObjectType::ObjectTable => e.token(TokenKind::TABLE_KW),
        ObjectType::ObjectSequence => e.token(TokenKind::SEQUENCE_KW),
        ObjectType::ObjectView => e.token(TokenKind::VIEW_KW),
        ObjectType::ObjectMatview => {
            e.token(TokenKind::MATERIALIZED_KW);
            e.space();
            e.token(TokenKind::VIEW_KW);
        }
        ObjectType::ObjectForeignTable => {
            e.token(TokenKind::FOREIGN_KW);
            e.space();
            e.token(TokenKind::TABLE_KW);
        }
        ObjectType::ObjectIndex => e.token(TokenKind::INDEX_KW),
        ObjectType::ObjectType => e.token(TokenKind::TYPE_KW),
        ObjectType::ObjectDomain => e.token(TokenKind::DOMAIN_KW),
        ObjectType::ObjectDatabase => e.token(TokenKind::DATABASE_KW),
        ObjectType::ObjectSchema => e.token(TokenKind::SCHEMA_KW),
        ObjectType::ObjectExtension => e.token(TokenKind::EXTENSION_KW),
        ObjectType::ObjectFunction => e.token(TokenKind::FUNCTION_KW),
        ObjectType::ObjectProcedure => e.token(TokenKind::PROCEDURE_KW),
        ObjectType::ObjectRoutine => e.token(TokenKind::ROUTINE_KW),
        ObjectType::ObjectAggregate => e.token(TokenKind::AGGREGATE_KW),
        ObjectType::ObjectOperator => e.token(TokenKind::OPERATOR_KW),
        ObjectType::ObjectOpclass => {
            e.token(TokenKind::OPERATOR_KW);
            e.space();
            e.token(TokenKind::CLASS_KW);
        }
        ObjectType::ObjectOpfamily => {
            e.token(TokenKind::OPERATOR_KW);
            e.space();
            e.token(TokenKind::FAMILY_KW);
        }
        ObjectType::ObjectConversion => e.token(TokenKind::CONVERSION_KW),
        ObjectType::ObjectCollation => e.token(TokenKind::COLLATION_KW),
        ObjectType::ObjectFdw => {
            e.token(TokenKind::FOREIGN_KW);
            e.space();
            e.token(TokenKind::DATA_KW);
            e.space();
            e.token(TokenKind::WRAPPER_KW);
        }
        ObjectType::ObjectForeignServer => e.token(TokenKind::SERVER_KW),
        ObjectType::ObjectLanguage => e.token(TokenKind::LANGUAGE_KW),
        ObjectType::ObjectPublication => e.token(TokenKind::PUBLICATION_KW),
        ObjectType::ObjectSubscription => e.token(TokenKind::SUBSCRIPTION_KW),
        ObjectType::ObjectRole => e.token(TokenKind::ROLE_KW),
        ObjectType::ObjectTablespace => e.token(TokenKind::TABLESPACE_KW),
        ObjectType::ObjectAccessMethod => {
            e.token(TokenKind::ACCESS_KW);
            e.space();
            e.token(TokenKind::METHOD_KW);
        }
        ObjectType::ObjectLargeobject => {
            e.token(TokenKind::LARGE_KW);
            e.space();
            e.token(TokenKind::OBJECT_KW);
        }
        ObjectType::ObjectPolicy => e.token(TokenKind::POLICY_KW),
        ObjectType::ObjectRule => e.token(TokenKind::RULE_KW),
        ObjectType::ObjectTrigger => e.token(TokenKind::TRIGGER_KW),
        ObjectType::ObjectStatisticExt => e.token(TokenKind::STATISTICS_KW),
        ObjectType::ObjectTsparser => {
            e.token(TokenKind::TEXT_KW);
            e.space();
            e.token(TokenKind::SEARCH_KW);
            e.space();
            e.token(TokenKind::PARSER_KW);
        }
        ObjectType::ObjectTsdictionary => {
            e.token(TokenKind::TEXT_KW);
            e.space();
            e.token(TokenKind::SEARCH_KW);
            e.space();
            e.token(TokenKind::DICTIONARY_KW);
        }
        ObjectType::ObjectTstemplate => {
            e.token(TokenKind::TEXT_KW);
            e.space();
            e.token(TokenKind::SEARCH_KW);
            e.space();
            e.token(TokenKind::TEMPLATE_KW);
        }
        ObjectType::ObjectTsconfiguration => {
            e.token(TokenKind::TEXT_KW);
            e.space();
            e.token(TokenKind::SEARCH_KW);
            e.space();
            e.token(TokenKind::CONFIGURATION_KW);
        }
        ObjectType::ObjectUserMapping => {
            e.token(TokenKind::USER_KW);
            e.space();
            e.token(TokenKind::MAPPING_KW);
        }
        _ => e.token(TokenKind::TABLE_KW),
    }
}

fn resolve_alter_target(rename_type: ObjectType, relation_type: ObjectType) -> ObjectType {
    match rename_type {
        ObjectType::ObjectColumn
        | ObjectType::ObjectTabconstraint
        | ObjectType::ObjectTrigger
        | ObjectType::ObjectRule => match relation_type {
            ObjectType::Undefined => ObjectType::ObjectTable,
            other => other,
        },
        ObjectType::ObjectDomconstraint => match relation_type {
            ObjectType::Undefined => ObjectType::ObjectDomain,
            other => other,
        },
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
        emit_node(object, e);
    }
}

fn emit_default_head(e: &mut EventEmitter, n: &RenameStmt) {
    if let Some(ref relation) = n.relation {
        e.space();
        emit_range_var(e, relation);
    } else if let Some(ref object) = n.object {
        e.space();
        emit_node(object, e);
    } else if !n.subname.is_empty() {
        e.space();
        emit_identifier_maybe_quoted(e, &n.subname);
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
    if let Some(ref object) = n.object {
        if let Some(NodeEnum::List(list)) = &object.node {
            if list.items.len() >= 2 {
                let (method_node, name_nodes) = list.items.split_first().unwrap();
                if !name_nodes.is_empty() {
                    e.space();
                    emit_dot_separated_list(e, name_nodes);
                    e.space();
                    e.token(TokenKind::USING_KW);
                    e.space();
                    emit_node(method_node, e);
                    return true;
                }
            }
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
    e.token(TokenKind::IDENT("ATTRIBUTE".to_string()));
    e.space();
    emit_identifier_maybe_quoted(e, old_name);
    e.space();
    e.token(TokenKind::TO_KW);
    e.space();
    emit_identifier_maybe_quoted(e, new_name);
}
