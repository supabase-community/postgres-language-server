//! The frame of a window: `ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW`, shared by the
//! window of a call (`WindowDef`) and the named windows of a WINDOW clause (`WindowClause`).

use pgls_query::Node;

use crate::TokenKind;
use crate::emitter::{EventEmitter, LineType};
use crate::nodes::emit_node;
use crate::nodes::utils::tokens::emit_keyword;

/// Frame option bits, from `parsenodes.h`.
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

/// Whether `frame_options` describe a frame of their own rather than the default one.
pub(crate) fn has_window_frame(frame_options: i32) -> bool {
    frame_options & FRAMEOPTION_NONDEFAULT != 0
}

/// Where a frame with a single bound, `ROWS UNBOUNDED PRECEDING`, may break.
#[derive(Clone, Copy)]
pub(crate) enum SingleBoundBreak {
    /// The mode and the bound stay together.
    Never,
    /// The bound may move to the next line. A named window of the WINDOW clause breaks there.
    AfterMode,
}

/// Emits the frame of a window, `ROWS BETWEEN ... AND ... EXCLUDE ...`, when it has one
/// (`has_window_frame`).
pub(crate) fn emit_window_frame(
    e: &mut EventEmitter,
    options: i32,
    start_offset: Option<&Node>,
    end_offset: Option<&Node>,
    single_bound_break: SingleBoundBreak,
) {
    emit_frame_mode(e, options);
    e.space();

    if options & FRAMEOPTION_BETWEEN != 0 {
        e.token(TokenKind::BETWEEN_KW);
        e.line(LineType::SoftOrSpace);
        emit_frame_bound(e, options, start_offset, FrameBoundSide::Start);
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::AND_KW);
        e.line(LineType::SoftOrSpace);
        emit_frame_bound(e, options, end_offset, FrameBoundSide::End);
    } else {
        if matches!(single_bound_break, SingleBoundBreak::AfterMode) {
            e.line(LineType::SoftOrSpace);
        }
        emit_frame_bound(e, options, start_offset, FrameBoundSide::Start);
    }

    if options & FRAMEOPTION_EXCLUSION_MASK != 0 {
        e.line(LineType::SoftOrSpace);
        emit_frame_exclusion(e, options);
    }
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
                if let Some(offset) = offset {
                    emit_node(e, offset);
                    e.space();
                }
                emit_keyword(e, "PRECEDING");
            } else if options & FRAMEOPTION_START_OFFSET_FOLLOWING != 0 {
                if let Some(offset) = offset {
                    emit_node(e, offset);
                    e.space();
                }
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
                if let Some(offset) = offset {
                    emit_node(e, offset);
                    e.space();
                }
                emit_keyword(e, "PRECEDING");
            } else if options & FRAMEOPTION_END_OFFSET_FOLLOWING != 0 {
                if let Some(offset) = offset {
                    emit_node(e, offset);
                    e.space();
                }
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
