use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};
use pgt_query::protobuf::CommentStmt;

use super::string::{emit_keyword, emit_single_quoted_str};

pub(super) fn emit_comment_stmt(e: &mut EventEmitter, n: &CommentStmt) {
    e.group_start(GroupKind::CommentStmt);

    emit_keyword(e, "COMMENT");
    e.space();
    e.token(TokenKind::ON_KW);
    e.space();

    // Object type - map ObjectType enum to keyword
    let object_type_tokens: &[&str] = match n.objtype {
        1 => &["ACCESS", "METHOD"],                 // ObjectAccessMethod
        2 => &["AGGREGATE"],                        // ObjectAggregate
        6 => &["CAST"],                             // ObjectCast
        7 => &["COLUMN"],                           // ObjectColumn
        8 => &["COLLATION"],                        // ObjectCollation
        9 => &["CONVERSION"],                       // ObjectConversion
        10 => &["DATABASE"],                        // ObjectDatabase
        13 => &["DOMAIN"],                          // ObjectDomain
        14 => &["CONSTRAINT"],                      // ObjectDomconstraint
        15 => &["EVENT", "TRIGGER"],                // ObjectEventTrigger
        16 => &["EXTENSION"],                       // ObjectExtension
        17 => &["FOREIGN", "DATA", "WRAPPER"],      // ObjectFdw
        18 => &["FOREIGN", "SERVER"],               // ObjectForeignServer
        19 => &["FOREIGN", "TABLE"],                // ObjectForeignTable
        20 => &["FUNCTION"],                        // ObjectFunction
        21 => &["INDEX"],                           // ObjectIndex
        22 => &["LANGUAGE"],                        // ObjectLanguage
        23 => &["LARGE", "OBJECT"],                 // ObjectLargeobject
        24 => &["MATERIALIZED", "VIEW"],            // ObjectMatview
        25 => &["OPERATOR", "CLASS"],               // ObjectOpclass
        26 => &["OPERATOR"],                        // ObjectOperator
        27 => &["OPERATOR", "FAMILY"],              // ObjectOpfamily
        29 => &["POLICY"],                          // ObjectPolicy
        30 => &["PROCEDURE"],                       // ObjectProcedure
        31 => &["PUBLICATION"],                     // ObjectPublication
        34 => &["ROLE"],                            // ObjectRole
        35 => &["ROUTINE"],                         // ObjectRoutine
        36 => &["RULE"],                            // ObjectRule
        37 => &["SCHEMA"],                          // ObjectSchema
        38 => &["SEQUENCE"],                        // ObjectSequence
        39 => &["SUBSCRIPTION"],                    // ObjectSubscription
        40 => &["STATISTICS"],                      // ObjectStatisticExt
        41 => &["CONSTRAINT"],                      // ObjectTabconstraint
        42 => &["TABLE"],                           // ObjectTable
        43 => &["TABLESPACE"],                      // ObjectTablespace
        44 => &["TRANSFORM"],                       // ObjectTransform
        45 => &["TRIGGER"],                         // ObjectTrigger
        46 => &["TEXT", "SEARCH", "CONFIGURATION"], // ObjectTsconfiguration
        47 => &["TEXT", "SEARCH", "DICTIONARY"],    // ObjectTsdictionary
        48 => &["TEXT", "SEARCH", "PARSER"],        // ObjectTsparser
        49 => &["TEXT", "SEARCH", "TEMPLATE"],      // ObjectTstemplate
        51 => &["TYPE"],                            // ObjectType
        52 => &["USER", "MAPPING"],                 // ObjectUsermapping
        53 => &["VIEW"],                            // ObjectView
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
        super::emit_node(object, e);
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
