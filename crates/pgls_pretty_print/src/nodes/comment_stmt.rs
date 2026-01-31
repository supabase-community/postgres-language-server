use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};
use pgls_query::{NodeEnum, protobuf::CommentStmt};

use super::emit_object_with_args_for_aggregate;
use super::node_list::emit_dot_separated_list;
use super::string::{emit_keyword, emit_single_quoted_str};

pub(super) fn emit_comment_stmt(e: &mut EventEmitter, n: &CommentStmt) {
    e.group_start(GroupKind::CommentStmt);

    emit_keyword(e, "COMMENT");
    e.space();
    e.token(TokenKind::ON_KW);
    e.space();

    // Object type - map ObjectType enum to keyword
    // Values from pgls_query::protobuf::ObjectType
    let object_type_tokens: &[&str] = match n.objtype {
        1 => &["ACCESS", "METHOD"],                 // ObjectAccessMethod = 1
        2 => &["AGGREGATE"],                        // ObjectAggregate = 2
        6 => &["CAST"],                             // ObjectCast = 6
        7 => &["COLUMN"],                           // ObjectColumn = 7
        8 => &["COLLATION"],                        // ObjectCollation = 8
        9 => &["CONVERSION"],                       // ObjectConversion = 9
        10 => &["DATABASE"],                        // ObjectDatabase = 10
        13 => &["DOMAIN"],                          // ObjectDomain = 13
        14 => &["CONSTRAINT"],                      // ObjectDomconstraint = 14
        15 => &["EVENT", "TRIGGER"],                // ObjectEventTrigger = 15
        16 => &["EXTENSION"],                       // ObjectExtension = 16
        17 => &["FOREIGN", "DATA", "WRAPPER"],      // ObjectFdw = 17
        18 => &["SERVER"],                          // ObjectForeignServer = 18 (COMMENT ON SERVER)
        19 => &["FOREIGN", "TABLE"],                // ObjectForeignTable = 19
        20 => &["FUNCTION"],                        // ObjectFunction = 20
        21 => &["INDEX"],                           // ObjectIndex = 21
        22 => &["LANGUAGE"],                        // ObjectLanguage = 22
        23 => &["LARGE", "OBJECT"],                 // ObjectLargeobject = 23
        24 => &["MATERIALIZED", "VIEW"],            // ObjectMatview = 24
        25 => &["OPERATOR", "CLASS"],               // ObjectOpclass = 25
        26 => &["OPERATOR"],                        // ObjectOperator = 26
        27 => &["OPERATOR", "FAMILY"],              // ObjectOpfamily = 27
        29 => &["POLICY"],                          // ObjectPolicy = 29
        30 => &["PROCEDURE"],                       // ObjectProcedure = 30
        31 => &["PUBLICATION"],                     // ObjectPublication = 31
        34 => &["ROLE"],                            // ObjectRole = 34
        35 => &["ROUTINE"],                         // ObjectRoutine = 35
        36 => &["RULE"],                            // ObjectRule = 36
        37 => &["SCHEMA"],                          // ObjectSchema = 37
        38 => &["SEQUENCE"],                        // ObjectSequence = 38
        39 => &["SUBSCRIPTION"],                    // ObjectSubscription = 39
        40 => &["STATISTICS"],                      // ObjectStatisticExt = 40
        41 => &["CONSTRAINT"],                      // ObjectTabconstraint = 41
        42 => &["TABLE"],                           // ObjectTable = 42
        43 => &["TABLESPACE"],                      // ObjectTablespace = 43
        44 => &["TRANSFORM"],                       // ObjectTransform = 44
        45 => &["TRIGGER"],                         // ObjectTrigger = 45
        46 => &["TEXT", "SEARCH", "CONFIGURATION"], // ObjectTsconfiguration = 46
        47 => &["TEXT", "SEARCH", "DICTIONARY"],    // ObjectTsdictionary = 47
        48 => &["TEXT", "SEARCH", "PARSER"],        // ObjectTsparser = 48
        49 => &["TEXT", "SEARCH", "TEMPLATE"],      // ObjectTstemplate = 49
        50 => &["TYPE"],                            // ObjectType = 50
        51 => &["USER", "MAPPING"],                 // ObjectUserMapping = 51
        52 => &["VIEW"],                            // ObjectView = 52
        _ => &["OBJECT"],
    };
    for (idx, token) in object_type_tokens.iter().enumerate() {
        if idx > 0 {
            e.space();
        }
        emit_keyword(e, token);
    }
    e.space();

    // Object name
    if let Some(ref object) = n.object {
        // For COLUMN (7), the object is a list of identifiers that should be dot-separated
        // e.g., "table.column" not "table, column"
        if n.objtype == 7 {
            // ObjectColumn - emit as dot-separated list
            if let Some(pgls_query::NodeEnum::List(list)) = object.node.as_ref() {
                emit_dot_separated_list(e, &list.items);
            } else {
                super::emit_node(object, e);
            }
        } else if n.objtype == 2 {
            // ObjectAggregate (2) - needs special handling for (*) syntax
            if let Some(NodeEnum::ObjectWithArgs(owa)) = object.node.as_ref() {
                emit_object_with_args_for_aggregate(e, owa);
            } else {
                super::emit_node(object, e);
            }
        } else if n.objtype == 14 {
            // ObjectDomconstraint (14)
            // Format: CONSTRAINT constraint_name ON DOMAIN domain_name
            // The list has [TypeName(domain_name), constraint_name]
            if let Some(NodeEnum::List(list)) = object.node.as_ref() {
                if list.items.len() == 2 {
                    // Emit constraint name first (second element)
                    super::emit_node(&list.items[1], e);
                    e.line(LineType::SoftOrSpace);
                    e.token(TokenKind::ON_KW);
                    e.space();
                    emit_keyword(e, "DOMAIN");
                    e.space();
                    // Then domain name (first element)
                    super::emit_node(&list.items[0], e);
                } else {
                    super::emit_node(object, e);
                }
            } else {
                super::emit_node(object, e);
            }
        } else if n.objtype == 41 {
            // ObjectTabconstraint (41)
            // Format: CONSTRAINT constraint_name ON table_name
            // The list has [table_name, constraint_name]
            if let Some(NodeEnum::List(list)) = object.node.as_ref() {
                if list.items.len() == 2 {
                    // Emit constraint name first (second element)
                    super::emit_node(&list.items[1], e);
                    e.line(LineType::SoftOrSpace);
                    e.token(TokenKind::ON_KW);
                    e.space();
                    // Then table name (first element)
                    super::emit_node(&list.items[0], e);
                } else {
                    super::emit_node(object, e);
                }
            } else {
                super::emit_node(object, e);
            }
        } else if n.objtype == 36 {
            // ObjectRule (36)
            // Format: RULE rule_name ON table_name
            // The list has [table_name, rule_name]
            if let Some(NodeEnum::List(list)) = object.node.as_ref() {
                if list.items.len() == 2 {
                    // Emit rule name first (second element)
                    super::emit_node(&list.items[1], e);
                    e.line(LineType::SoftOrSpace);
                    e.token(TokenKind::ON_KW);
                    e.space();
                    // Then table name (first element)
                    super::emit_node(&list.items[0], e);
                } else {
                    super::emit_node(object, e);
                }
            } else {
                super::emit_node(object, e);
            }
        } else if n.objtype == 45 {
            // ObjectTrigger (45)
            // Format: TRIGGER trigger_name ON table_name
            // The list has [table_name, trigger_name]
            if let Some(NodeEnum::List(list)) = object.node.as_ref() {
                if list.items.len() == 2 {
                    // Emit trigger name first (second element)
                    super::emit_node(&list.items[1], e);
                    e.line(LineType::SoftOrSpace);
                    e.token(TokenKind::ON_KW);
                    e.space();
                    // Then table name (first element)
                    super::emit_node(&list.items[0], e);
                } else {
                    super::emit_node(object, e);
                }
            } else {
                super::emit_node(object, e);
            }
        } else if n.objtype == 29 {
            // ObjectPolicy (29)
            // Format: POLICY policy_name ON table_name
            // The list has [table_name, policy_name]
            if let Some(NodeEnum::List(list)) = object.node.as_ref() {
                if list.items.len() == 2 {
                    // Emit policy name first (second element)
                    super::emit_node(&list.items[1], e);
                    e.line(LineType::SoftOrSpace);
                    e.token(TokenKind::ON_KW);
                    e.space();
                    // Then table name (first element)
                    super::emit_node(&list.items[0], e);
                } else {
                    super::emit_node(object, e);
                }
            } else {
                super::emit_node(object, e);
            }
        } else if n.objtype == 51 {
            // ObjectUserMapping (51) - requires FOR before the role name
            // COMMENT ON USER MAPPING FOR role_name SERVER server_name IS 'comment'
            e.token(TokenKind::FOR_KW);
            e.space();
            super::emit_node(object, e);
        } else {
            super::emit_node(object, e);
        }
    }

    e.space();
    e.token(TokenKind::IS_KW);
    e.space();

    // Comment text
    if n.comment.is_empty() {
        e.token(TokenKind::NULL_KW);
    } else {
        emit_single_quoted_str(e, &n.comment);
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
