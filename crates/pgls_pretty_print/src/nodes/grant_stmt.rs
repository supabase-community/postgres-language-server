use pgls_query::protobuf::{DropBehavior, GrantStmt, GrantTargetType, ObjectType};

use super::prelude::*;

pub(super) fn emit_grant_stmt(e: &mut EventEmitter, n: &GrantStmt) {
    e.group_start(GroupKind::GrantStmt);

    // GRANT or REVOKE
    if n.is_grant {
        e.token(TokenKind::GRANT_KW);
    } else {
        e.token(TokenKind::REVOKE_KW);
    }

    // GRANT OPTION FOR (for revoke)
    if !n.is_grant && n.grant_option {
        e.space();
        e.token(TokenKind::GRANT_KW);
        e.space();
        e.token(TokenKind::OPTION_KW);
        e.space();
        e.token(TokenKind::FOR_KW);
    }

    e.space();

    // Privileges
    if n.privileges.is_empty() {
        e.token(TokenKind::ALL_KW);
    } else {
        emit_grant_list(e, &n.privileges, |node, e| {
            if let Some(pgls_query::NodeEnum::AccessPriv(priv_node)) = &node.node {
                emit_access_priv(e, priv_node);
            }
        });
    }

    // The clauses after the privileges continue the statement, so a broken one indents them.
    e.indent_start();

    if let Some(object) = n.objects.first() {
        emit_comments_before_keyword(e, "on", object);
    }
    e.line(LineType::SoftOrSpace);
    e.token(TokenKind::ON_KW);
    e.space();

    // Target type and object type
    let targtype = n.targtype();
    let objtype = n.objtype();

    if let GrantTargetType::AclTargetAllInSchema = targtype {
        e.token(TokenKind::ALL_KW);
        e.space();
        match objtype {
            ObjectType::ObjectTable => {
                e.token(TokenKind::TABLES_KW);
            }
            ObjectType::ObjectSequence => {
                e.token(TokenKind::SEQUENCES_KW);
            }
            ObjectType::ObjectFunction => {
                e.token(TokenKind::FUNCTIONS_KW);
            }
            ObjectType::ObjectProcedure => {
                e.token(TokenKind::PROCEDURES_KW);
            }
            ObjectType::ObjectRoutine => {
                e.token(TokenKind::ROUTINES_KW);
            }
            _ => {}
        }
        e.space();
        e.token(TokenKind::IN_KW);
        e.space();
        e.token(TokenKind::SCHEMA_KW);
        e.space();
    } else if let GrantTargetType::AclTargetDefaults = targtype {
        // For ALTER DEFAULT PRIVILEGES, use plural object types
        match objtype {
            ObjectType::ObjectTable => {
                e.token(TokenKind::TABLES_KW);
            }
            ObjectType::ObjectSequence => {
                e.token(TokenKind::SEQUENCES_KW);
            }
            ObjectType::ObjectFunction => {
                e.token(TokenKind::FUNCTIONS_KW);
            }
            ObjectType::ObjectProcedure => {
                e.token(TokenKind::PROCEDURES_KW);
            }
            ObjectType::ObjectRoutine => {
                e.token(TokenKind::ROUTINES_KW);
            }
            ObjectType::ObjectType => {
                e.token(TokenKind::TYPES_KW);
            }
            ObjectType::ObjectSchema => {
                e.token(TokenKind::SCHEMAS_KW);
            }
            _ => {}
        }
        // No space here - will be added by line break before TO/FROM
    } else {
        // Add explicit object type (singular)
        match objtype {
            ObjectType::ObjectTable => {
                e.token(TokenKind::TABLE_KW);
                e.space();
            }
            ObjectType::ObjectSequence => {
                e.token(TokenKind::SEQUENCE_KW);
                e.space();
            }
            ObjectType::ObjectDatabase => {
                e.token(TokenKind::DATABASE_KW);
                e.space();
            }
            ObjectType::ObjectSchema => {
                e.token(TokenKind::SCHEMA_KW);
                e.space();
            }
            ObjectType::ObjectFunction => {
                e.token(TokenKind::FUNCTION_KW);
                e.space();
            }
            ObjectType::ObjectProcedure => {
                e.token(TokenKind::PROCEDURE_KW);
                e.space();
            }
            ObjectType::ObjectType => {
                e.token(TokenKind::TYPE_KW);
                e.space();
            }
            ObjectType::ObjectLargeobject => {
                e.token(TokenKind::LARGE_KW);
                e.space();
                e.token(TokenKind::OBJECT_KW);
                e.space();
            }
            ObjectType::ObjectFdw => {
                e.token(TokenKind::FOREIGN_KW);
                e.space();
                e.token(TokenKind::DATA_KW);
                e.space();
                e.token(TokenKind::WRAPPER_KW);
                e.space();
            }
            ObjectType::ObjectForeignServer => {
                e.token(TokenKind::FOREIGN_KW);
                e.space();
                e.token(TokenKind::SERVER_KW);
                e.space();
            }
            ObjectType::ObjectLanguage => {
                e.token(TokenKind::LANGUAGE_KW);
                e.space();
            }
            ObjectType::ObjectTablespace => {
                e.token(TokenKind::TABLESPACE_KW);
                e.space();
            }
            ObjectType::ObjectDomain => {
                e.token(TokenKind::DOMAIN_KW);
                e.space();
            }
            ObjectType::ObjectRoutine => {
                e.token(TokenKind::ROUTINE_KW);
                e.space();
            }
            _ => {}
        }
    }

    // Object names
    if !n.objects.is_empty() {
        emit_grant_list(e, &n.objects, emit_node);
    }

    // TO/FROM - allow line break before this clause
    if let Some(grantee) = n.grantees.first() {
        emit_comments_before_keyword(e, if n.is_grant { "to" } else { "from" }, grantee);
    }
    e.line(LineType::SoftOrSpace);

    if n.is_grant {
        e.token(TokenKind::TO_KW);
    } else {
        e.token(TokenKind::FROM_KW);
    }

    e.space();

    // Grantees
    if !n.grantees.is_empty() {
        emit_grant_list(e, &n.grantees, emit_node);
    }

    // WITH GRANT OPTION (for grant)
    if n.is_grant && n.grant_option {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::WITH_KW);
        e.space();
        e.token(TokenKind::GRANT_KW);
        e.space();
        e.token(TokenKind::OPTION_KW);
    }

    // GRANTED BY
    if let Some(ref grantor) = n.grantor {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::GRANTED_KW);
        e.space();
        e.token(TokenKind::BY_KW);
        e.space();
        super::emit_role_spec(e, grantor);
    }

    // CASCADE/RESTRICT (for revoke)
    if !n.is_grant && matches!(n.behavior(), DropBehavior::DropCascade) {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::CASCADE_KW);
    }

    e.indent_end();

    e.group_end();
}

/// Emits the privileges, objects, or roles of a `GRANT` or `REVOKE`. The list is its own group, so
/// it stays on one line when it fits even though the statement broke, and otherwise fills each line
/// with as many items as fit, wrapping one level deeper than its clause.
///
/// The deeper level starts after the first item, which shares the clause's line: a column list
/// such as `insert (a, b)` that breaks closes its parenthesis under the line that opened it.
pub(super) fn emit_grant_list<F>(e: &mut EventEmitter, nodes: &[pgls_query::Node], render: F)
where
    F: Fn(&pgls_query::Node, &mut EventEmitter),
{
    let Some((first, rest)) = nodes.split_first() else {
        return;
    };

    e.group_start(GroupKind::List);
    render(first, e);
    if !rest.is_empty() {
        e.indent_start();
        for node in rest {
            emit_list_comma(e, Some(node), ListSeparatorSpacing::Fill);
            render(node, e);
        }
        e.indent_end();
    }
    e.group_end();
}

fn emit_access_priv(e: &mut EventEmitter, priv_node: &pgls_query::protobuf::AccessPriv) {
    if priv_node.priv_name.is_empty() && !priv_node.cols.is_empty() {
        e.token(TokenKind::ALL_KW);
    } else if !priv_node.priv_name.is_empty() {
        emit_keywords(e, &priv_node.priv_name);
    }

    // Handle column privileges
    if !priv_node.cols.is_empty() {
        e.space();
        emit_parenthesized_list(e, &priv_node.cols, emit_node);
    }
}
