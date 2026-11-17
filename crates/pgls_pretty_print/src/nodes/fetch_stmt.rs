use pgls_query::protobuf::{FetchDirection, FetchStmt};

use super::prelude::*;

/// `FETCH ALL` is `FETCH_ALL`, which is `LONG_MAX` of the platform libpg_query is built for.
/// https://github.com/postgres/postgres/blob/REL_18_6/src/include/nodes/parsenodes.h#L3425
// `c_long` is `i64` on Linux and macOS but `i32` on Windows.
#[allow(clippy::unnecessary_cast)]
const FETCH_ALL: i64 = std::ffi::c_long::MAX as i64;

/// Any other count is printed as a number, even 0 and negative ones: `FETCH RELATIVE 0` refetches
/// the current row and `FETCH FORWARD -2` moves backwards, neither is `ALL`.
/// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L7479
pub(super) fn emit_fetch_stmt(e: &mut EventEmitter, n: &FetchStmt) {
    e.group_start();

    if n.ismove {
        e.token(TokenKind::MOVE_KW);
    } else {
        e.token(TokenKind::FETCH_KW);
    }

    let (keyword, count) = match (n.direction(), n.how_many) {
        (FetchDirection::FetchForward, 1) => (Some(TokenKind::NEXT_KW), None),
        (FetchDirection::FetchForward, _) => (Some(TokenKind::FORWARD_KW), Some(n.how_many)),
        (FetchDirection::FetchBackward, 1) => (Some(TokenKind::PRIOR_KW), None),
        (FetchDirection::FetchBackward, _) => (Some(TokenKind::BACKWARD_KW), Some(n.how_many)),
        (FetchDirection::FetchAbsolute, _) => (Some(TokenKind::ABSOLUTE_KW), Some(n.how_many)),
        (FetchDirection::FetchRelative, _) => (Some(TokenKind::RELATIVE_KW), Some(n.how_many)),
        (FetchDirection::Undefined, _) => (None, None),
    };

    if let Some(keyword) = keyword {
        e.space();
        e.token(keyword);
    }
    match count {
        Some(FETCH_ALL) => {
            e.space();
            e.token(TokenKind::ALL_KW);
        }
        Some(count) => {
            e.space();
            e.token(TokenKind::IDENT(count.to_string()));
        }
        None => {}
    }

    if !n.portalname.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::FROM_KW);
        e.space();
        emit_identifier_maybe_quoted(e, &n.portalname);
    }

    e.group_end();
}
