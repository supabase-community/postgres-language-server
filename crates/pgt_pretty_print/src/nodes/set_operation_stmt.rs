use pgt_query::protobuf::{SetOperation, SetOperationStmt};

use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind, LineType};

pub(super) fn emit_set_operation_stmt(e: &mut EventEmitter, n: &SetOperationStmt) {
    e.group_start(GroupKind::SetOperationStmt);

    // Emit left operand (SELECT or another set operation)
    if let Some(ref larg) = n.larg {
        super::emit_node(larg, e);
    }

    // Emit set operation keyword (UNION, INTERSECT, EXCEPT)
    e.line(LineType::Hard);

    match n.op() {
        SetOperation::SetopUnion => {
            e.token(TokenKind::UNION_KW);
            if n.all {
                e.space();
                e.token(TokenKind::ALL_KW);
            }
        }
        SetOperation::SetopIntersect => {
            e.token(TokenKind::INTERSECT_KW);
            if n.all {
                e.space();
                e.token(TokenKind::ALL_KW);
            }
        }
        SetOperation::SetopExcept => {
            e.token(TokenKind::EXCEPT_KW);
            if n.all {
                e.space();
                e.token(TokenKind::ALL_KW);
            }
        }
        SetOperation::SetopNone | SetOperation::Undefined => {
            assert!(false, "unexpected SetOperation: {:?}", n.op());
        }
    }

    // Emit right operand
    if let Some(ref rarg) = n.rarg {
        e.line(LineType::Hard);
        super::emit_node(rarg, e);
    }

    e.group_end();
}
