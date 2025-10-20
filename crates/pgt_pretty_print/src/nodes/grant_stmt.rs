use pgt_query::protobuf::{DropBehavior, GrantStmt, GrantTargetType, ObjectType};

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
    nodes::node_list::emit_comma_separated_list,
};

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
        emit_comma_separated_list(e, &n.privileges, |node, e| {
            if let Some(pgt_query::NodeEnum::AccessPriv(priv_node)) = &node.node {
                emit_access_priv(e, priv_node);
            }
        });
    }

    e.space();
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
                e.token(TokenKind::IDENT("TABLES".to_string()));
            }
            ObjectType::ObjectSequence => {
                e.token(TokenKind::IDENT("SEQUENCES".to_string()));
            }
            ObjectType::ObjectFunction => {
                e.token(TokenKind::IDENT("FUNCTIONS".to_string()));
            }
            ObjectType::ObjectProcedure => {
                e.token(TokenKind::IDENT("PROCEDURES".to_string()));
            }
            ObjectType::ObjectRoutine => {
                e.token(TokenKind::IDENT("ROUTINES".to_string()));
            }
            _ => {}
        }
        e.space();
        e.token(TokenKind::IN_KW);
        e.space();
        e.token(TokenKind::SCHEMA_KW);
    } else if let GrantTargetType::AclTargetDefaults = targtype {
        // For ALTER DEFAULT PRIVILEGES, use plural object types
        match objtype {
            ObjectType::ObjectTable => {
                e.token(TokenKind::IDENT("TABLES".to_string()));
            }
            ObjectType::ObjectSequence => {
                e.token(TokenKind::IDENT("SEQUENCES".to_string()));
            }
            ObjectType::ObjectFunction => {
                e.token(TokenKind::IDENT("FUNCTIONS".to_string()));
            }
            ObjectType::ObjectProcedure => {
                e.token(TokenKind::IDENT("PROCEDURES".to_string()));
            }
            ObjectType::ObjectRoutine => {
                e.token(TokenKind::IDENT("ROUTINES".to_string()));
            }
            ObjectType::ObjectType => {
                e.token(TokenKind::IDENT("TYPES".to_string()));
            }
            ObjectType::ObjectSchema => {
                e.token(TokenKind::IDENT("SCHEMAS".to_string()));
            }
            _ => {}
        }
        e.space();
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
                e.token(TokenKind::IDENT("LARGE".to_string()));
                e.space();
                e.token(TokenKind::OBJECT_KW);
                e.space();
            }
            _ => {}
        }
    }

    // Object names
    if !n.objects.is_empty() {
        emit_comma_separated_list(e, &n.objects, super::emit_node);
        e.space();
    }

    // TO/FROM
    if n.is_grant {
        e.token(TokenKind::TO_KW);
    } else {
        e.token(TokenKind::FROM_KW);
    }

    e.space();

    // Grantees
    if !n.grantees.is_empty() {
        emit_comma_separated_list(e, &n.grantees, super::emit_node);
    }

    // WITH GRANT OPTION (for grant)
    if n.is_grant && n.grant_option {
        e.space();
        e.token(TokenKind::WITH_KW);
        e.space();
        e.token(TokenKind::GRANT_KW);
        e.space();
        e.token(TokenKind::OPTION_KW);
    }

    // GRANTED BY
    if let Some(ref grantor) = n.grantor {
        e.space();
        e.token(TokenKind::IDENT("GRANTED".to_string()));
        e.space();
        e.token(TokenKind::BY_KW);
        e.space();
        super::emit_role_spec(e, grantor);
    }

    // CASCADE/RESTRICT (for revoke)
    if !n.is_grant {
        if matches!(n.behavior(), DropBehavior::DropCascade) {
            e.space();
            e.token(TokenKind::CASCADE_KW);
        }
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}

fn emit_access_priv(e: &mut EventEmitter, priv_node: &pgt_query::protobuf::AccessPriv) {
    if priv_node.priv_name.is_empty() && !priv_node.cols.is_empty() {
        e.token(TokenKind::ALL_KW);
    } else if !priv_node.priv_name.is_empty() {
        e.token(TokenKind::IDENT(priv_node.priv_name.to_uppercase()));
    }

    // Handle column privileges
    if !priv_node.cols.is_empty() {
        e.space();
        e.token(TokenKind::L_PAREN);
        emit_comma_separated_list(e, &priv_node.cols, super::emit_node);
        e.token(TokenKind::R_PAREN);
    }
}
