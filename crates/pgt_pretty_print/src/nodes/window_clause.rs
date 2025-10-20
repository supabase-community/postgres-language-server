use crate::nodes::node_list::emit_comma_separated_list;
use crate::nodes::string::{emit_identifier_maybe_quoted, emit_keyword};
use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};
use pgt_query::protobuf::{Node, WindowClause};

pub(super) fn emit_window_clause(e: &mut EventEmitter, n: &WindowClause) {
    e.group_start(GroupKind::WindowClause);

    // Emit: name AS (window_spec)
    emit_identifier_maybe_quoted(e, &n.name);
    e.space();
    e.token(TokenKind::AS_KW);
    e.space();
    emit_window_spec(e, n);

    e.group_end();
}

/// Emit the window specification part: (PARTITION BY ... ORDER BY ... frame_clause)
fn emit_window_spec(e: &mut EventEmitter, n: &WindowClause) {
    e.token(TokenKind::L_PAREN);

    let mut has_content = false;

    if !n.refname.is_empty() {
        e.line(LineType::SoftOrSpace);
        emit_identifier_maybe_quoted(e, &n.refname);
        has_content = true;
    }

    if !n.partition_clause.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::PARTITION_KW);
        e.space();
        e.token(TokenKind::BY_KW);
        e.space();
        emit_comma_separated_list(e, &n.partition_clause, |node, emitter| {
            super::emit_node(node, emitter)
        });
        has_content = true;
    }

    if !n.order_clause.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::ORDER_KW);
        e.space();
        e.token(TokenKind::BY_KW);
        e.space();
        emit_comma_separated_list(e, &n.order_clause, |node, emitter| {
            super::emit_node(node, emitter)
        });
        has_content = true;
    }

    if emit_frame_clause(e, n) {
        has_content = true;
    }

    if !has_content {
        // Preserve empty parentheses for OVER ()
        e.token(TokenKind::R_PAREN);
        return;
    }

    e.token(TokenKind::R_PAREN);
}

/// Frame option constants (from parser/nodes.h)
const FRAMEOPTION_NONDEFAULT: i32 = 0x00001;
const FRAMEOPTION_RANGE: i32 = 0x00002;
const FRAMEOPTION_ROWS: i32 = 0x00004;
const FRAMEOPTION_GROUPS: i32 = 0x00008;
const FRAMEOPTION_BETWEEN: i32 = 0x00010;
const FRAMEOPTION_START_UNBOUNDED_PRECEDING: i32 = 0x00020;
const FRAMEOPTION_END_UNBOUNDED_PRECEDING: i32 = 0x00040;
const FRAMEOPTION_START_UNBOUNDED_FOLLOWING: i32 = 0x00080;
const FRAMEOPTION_END_UNBOUNDED_FOLLOWING: i32 = 0x00100;
const FRAMEOPTION_START_CURRENT_ROW: i32 = 0x00200;
const FRAMEOPTION_END_CURRENT_ROW: i32 = 0x00400;
const FRAMEOPTION_START_OFFSET_PRECEDING: i32 = 0x00800;
const FRAMEOPTION_END_OFFSET_PRECEDING: i32 = 0x01000;
const FRAMEOPTION_START_OFFSET_FOLLOWING: i32 = 0x02000;
const FRAMEOPTION_END_OFFSET_FOLLOWING: i32 = 0x04000;
const FRAMEOPTION_EXCLUDE_CURRENT_ROW: i32 = 0x08000;
const FRAMEOPTION_EXCLUDE_GROUP: i32 = 0x10000;
const FRAMEOPTION_EXCLUDE_TIES: i32 = 0x20000;
const FRAMEOPTION_EXCLUSION_MASK: i32 =
    FRAMEOPTION_EXCLUDE_CURRENT_ROW | FRAMEOPTION_EXCLUDE_GROUP | FRAMEOPTION_EXCLUDE_TIES;

#[derive(Copy, Clone)]
enum FrameBoundSide {
    Start,
    End,
}

fn emit_frame_clause(e: &mut EventEmitter, n: &WindowClause) -> bool {
    let options = n.frame_options;

    if options & FRAMEOPTION_NONDEFAULT == 0 {
        return false;
    }

    e.line(LineType::SoftOrSpace);

    emit_frame_mode(e, options);
    e.space();

    if options & FRAMEOPTION_BETWEEN != 0 {
        e.token(TokenKind::BETWEEN_KW);
        e.line(LineType::SoftOrSpace);
        emit_frame_bound(e, options, n.start_offset.as_deref(), FrameBoundSide::Start);
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::AND_KW);
        e.line(LineType::SoftOrSpace);
        emit_frame_bound(e, options, n.end_offset.as_deref(), FrameBoundSide::End);
    } else {
        e.line(LineType::SoftOrSpace);
        emit_frame_bound(e, options, n.start_offset.as_deref(), FrameBoundSide::Start);
    }

    if options & FRAMEOPTION_EXCLUSION_MASK != 0 {
        e.line(LineType::SoftOrSpace);
        emit_frame_exclusion(e, options);
    }

    true
}

fn emit_frame_mode(e: &mut EventEmitter, options: i32) {
    if options & FRAMEOPTION_RANGE != 0 {
        e.token(TokenKind::RANGE_KW);
    } else if options & FRAMEOPTION_ROWS != 0 {
        e.token(TokenKind::ROWS_KW);
    } else if options & FRAMEOPTION_GROUPS != 0 {
        emit_keyword(e, "GROUPS");
    } else {
        e.token(TokenKind::RANGE_KW);
    }
}

fn emit_frame_bound(
    e: &mut EventEmitter,
    options: i32,
    offset: Option<&Node>,
    side: FrameBoundSide,
) {
    match side {
        FrameBoundSide::Start => {
            if options & FRAMEOPTION_START_UNBOUNDED_PRECEDING != 0 {
                emit_keyword(e, "UNBOUNDED");
                e.space();
                emit_keyword(e, "PRECEDING");
            } else if options & FRAMEOPTION_START_UNBOUNDED_FOLLOWING != 0 {
                debug_assert!(false, "window frame start cannot be UNBOUNDED FOLLOWING");
                emit_keyword(e, "UNBOUNDED");
                e.space();
                emit_keyword(e, "FOLLOWING");
            } else if options & FRAMEOPTION_START_CURRENT_ROW != 0 {
                e.token(TokenKind::CURRENT_KW);
                e.space();
                e.token(TokenKind::ROW_KW);
            } else if options & FRAMEOPTION_START_OFFSET_PRECEDING != 0 {
                let offset_node =
                    offset.expect("FRAMEOPTION_START_OFFSET_PRECEDING requires start_offset");
                super::emit_node(offset_node, e);
                e.space();
                emit_keyword(e, "PRECEDING");
            } else if options & FRAMEOPTION_START_OFFSET_FOLLOWING != 0 {
                let offset_node =
                    offset.expect("FRAMEOPTION_START_OFFSET_FOLLOWING requires start_offset");
                super::emit_node(offset_node, e);
                e.space();
                emit_keyword(e, "FOLLOWING");
            } else {
                debug_assert!(false, "unhandled window frame start options: {options:#x}");
                emit_keyword(e, "CURRENT");
                e.space();
                emit_keyword(e, "ROW");
            }
        }
        FrameBoundSide::End => {
            if options & FRAMEOPTION_END_UNBOUNDED_PRECEDING != 0 {
                debug_assert!(false, "window frame end cannot be UNBOUNDED PRECEDING");
                emit_keyword(e, "UNBOUNDED");
                e.space();
                emit_keyword(e, "PRECEDING");
            } else if options & FRAMEOPTION_END_UNBOUNDED_FOLLOWING != 0 {
                emit_keyword(e, "UNBOUNDED");
                e.space();
                emit_keyword(e, "FOLLOWING");
            } else if options & FRAMEOPTION_END_CURRENT_ROW != 0 {
                e.token(TokenKind::CURRENT_KW);
                e.space();
                e.token(TokenKind::ROW_KW);
            } else if options & FRAMEOPTION_END_OFFSET_PRECEDING != 0 {
                let offset_node =
                    offset.expect("FRAMEOPTION_END_OFFSET_PRECEDING requires end_offset");
                super::emit_node(offset_node, e);
                e.space();
                emit_keyword(e, "PRECEDING");
            } else if options & FRAMEOPTION_END_OFFSET_FOLLOWING != 0 {
                let offset_node =
                    offset.expect("FRAMEOPTION_END_OFFSET_FOLLOWING requires end_offset");
                super::emit_node(offset_node, e);
                e.space();
                emit_keyword(e, "FOLLOWING");
            } else {
                debug_assert!(false, "unhandled window frame end options: {options:#x}");
                emit_keyword(e, "CURRENT");
                e.space();
                emit_keyword(e, "ROW");
            }
        }
    }
}

fn emit_frame_exclusion(e: &mut EventEmitter, options: i32) {
    e.token(TokenKind::EXCLUDE_KW);
    e.space();

    if options & FRAMEOPTION_EXCLUDE_CURRENT_ROW != 0 {
        e.token(TokenKind::CURRENT_KW);
        e.space();
        e.token(TokenKind::ROW_KW);
    } else if options & FRAMEOPTION_EXCLUDE_GROUP != 0 {
        e.token(TokenKind::GROUP_KW);
    } else if options & FRAMEOPTION_EXCLUDE_TIES != 0 {
        emit_keyword(e, "TIES");
    }
}
