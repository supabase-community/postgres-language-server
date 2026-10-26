use pgls_query::protobuf::{GrantStmt, GrantTargetType, ObjectType};

use super::prelude::*;

pub(super) fn emit_grant_stmt(e: &mut EventEmitter, n: &GrantStmt) {
    e.group_start();

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
        emit_grant_list(e, &n.privileges, emit_node);
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
        // Add explicit object type (singular). GRANT names a foreign server `FOREIGN SERVER`.
        let keywords = match objtype {
            ObjectType::ObjectForeignServer => Some("FOREIGN SERVER"),
            objtype => object_type_keywords(objtype),
        };
        if let Some(keywords) = keywords {
            emit_keywords(e, keywords);
            e.space();
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
    if !n.is_grant {
        emit_drop_behavior(e, n.behavior(), Separator::SoftOrSpace, Restrict::Omitted);
    }

    e.indent_end();

    e.group_end();
}
