use pgt_query::protobuf::{SqlValueFunction, SqlValueFunctionOp};

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_sql_value_function(e: &mut EventEmitter, n: &SqlValueFunction) {
    e.group_start(GroupKind::SqlvalueFunction);

    // Map function type to SQL keyword
    match n.op() {
        SqlValueFunctionOp::SvfopCurrentDate => {
            e.token(TokenKind::CURRENT_DATE_KW);
        }
        SqlValueFunctionOp::SvfopCurrentTime => {
            e.token(TokenKind::CURRENT_TIME_KW);
        }
        SqlValueFunctionOp::SvfopCurrentTimestamp => {
            e.token(TokenKind::CURRENT_TIMESTAMP_KW);
        }
        SqlValueFunctionOp::SvfopCurrentUser => {
            e.token(TokenKind::CURRENT_USER_KW);
        }
        SqlValueFunctionOp::SvfopCurrentRole => {
            e.token(TokenKind::CURRENT_ROLE_KW);
        }
        SqlValueFunctionOp::SvfopCurrentCatalog => {
            e.token(TokenKind::CURRENT_CATALOG_KW);
        }
        SqlValueFunctionOp::SvfopCurrentSchema => {
            e.token(TokenKind::CURRENT_SCHEMA_KW);
        }
        SqlValueFunctionOp::SvfopSessionUser => {
            e.token(TokenKind::SESSION_USER_KW);
        }
        SqlValueFunctionOp::SvfopUser => {
            e.token(TokenKind::USER_KW);
        }
        SqlValueFunctionOp::SvfopLocaltime => {
            e.token(TokenKind::LOCALTIME_KW);
        }
        SqlValueFunctionOp::SvfopLocaltimestamp => {
            e.token(TokenKind::LOCALTIMESTAMP_KW);
        }
        _ => {
            // Fallback for unknown types
            e.token(TokenKind::IDENT("UNKNOWN_SQL_VALUE_FUNCTION".to_string()));
        }
    }

    e.group_end();
}
