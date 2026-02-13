use pgls_query::protobuf::{SqlValueFunction, SqlValueFunctionOp};

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
        SqlValueFunctionOp::SvfopCurrentTimeN => {
            e.token(TokenKind::CURRENT_TIME_KW);
            emit_precision(e, n.typmod);
        }
        SqlValueFunctionOp::SvfopCurrentTimestamp => {
            e.token(TokenKind::CURRENT_TIMESTAMP_KW);
        }
        SqlValueFunctionOp::SvfopCurrentTimestampN => {
            e.token(TokenKind::CURRENT_TIMESTAMP_KW);
            emit_precision(e, n.typmod);
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
        SqlValueFunctionOp::SvfopLocaltimeN => {
            e.token(TokenKind::LOCALTIME_KW);
            emit_precision(e, n.typmod);
        }
        SqlValueFunctionOp::SvfopLocaltimestamp => {
            e.token(TokenKind::LOCALTIMESTAMP_KW);
        }
        SqlValueFunctionOp::SvfopLocaltimestampN => {
            e.token(TokenKind::LOCALTIMESTAMP_KW);
            emit_precision(e, n.typmod);
        }
        _ => {
            // Fallback for unknown types
            e.token(TokenKind::IDENT("UNKNOWN_SQL_VALUE_FUNCTION".to_string()));
        }
    }

    e.group_end();
}

fn emit_precision(e: &mut EventEmitter, typmod: i32) {
    e.token(TokenKind::L_PAREN);
    e.token(TokenKind::IDENT(typmod.to_string()));
    e.token(TokenKind::R_PAREN);
}
