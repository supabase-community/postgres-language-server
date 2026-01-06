use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind};
use pgls_query::protobuf::CreateOpClassItem;

use super::node_list::{emit_comma_separated_list, emit_dot_separated_list};
use super::object_with_args::{emit_object_name_only, emit_object_with_args};

pub(super) fn emit_create_op_class_item(e: &mut EventEmitter, n: &CreateOpClassItem) {
    e.group_start(GroupKind::CreateOpClassItem);

    // itemtype: 1=OPERATOR, 2=FUNCTION, 3=STORAGE
    match n.itemtype {
        1 => {
            // OPERATOR strategy_number [operator_name] (arg_types)
            // For DROP: no operator name, use class_args for types
            // For ADD: operator name with objargs for types
            e.token(TokenKind::IDENT("OPERATOR".to_string()));
            e.space();
            e.token(TokenKind::IDENT(n.number.to_string()));
            e.space();

            if let Some(ref name) = n.name {
                // ADD case: emit operator name
                emit_object_name_only(e, name);

                // Emit argument types in parentheses from objargs
                if !name.objargs.is_empty() {
                    e.space();
                    e.group_start(GroupKind::CreateOpClassItem);
                    e.token(TokenKind::L_PAREN);
                    emit_comma_separated_list(e, &name.objargs, super::emit_node);
                    e.token(TokenKind::R_PAREN);
                    e.group_end();
                }
            } else if !n.class_args.is_empty() {
                // DROP case: no operator name, just emit (left_type, right_type) from class_args
                e.token(TokenKind::L_PAREN);
                emit_comma_separated_list(e, &n.class_args, super::emit_node);
                e.token(TokenKind::R_PAREN);
            }

            // Optional FOR ORDER BY opfamily
            if !n.order_family.is_empty() {
                e.space();
                e.token(TokenKind::FOR_KW);
                e.space();
                e.token(TokenKind::ORDER_KW);
                e.space();
                e.token(TokenKind::BY_KW);
                e.space();
                emit_dot_separated_list(e, &n.order_family);
            }
        }
        2 => {
            // FUNCTION support_number [(left_type, right_type)] function_name(arg_types)
            // For DROP: no function name, use class_args for types
            // For ADD: class_args come BEFORE function name (if present)
            e.token(TokenKind::IDENT("FUNCTION".to_string()));
            e.space();
            e.token(TokenKind::IDENT(n.number.to_string()));

            // class_args come first (for ADD or DROP)
            if !n.class_args.is_empty() {
                e.space();
                e.token(TokenKind::L_PAREN);
                emit_comma_separated_list(e, &n.class_args, super::emit_node);
                e.token(TokenKind::R_PAREN);
            }

            if let Some(ref name) = n.name {
                // ADD case: emit function name with args after class_args
                e.space();
                emit_object_with_args(e, name);
            }
        }
        3 => {
            // STORAGE storage_type
            e.token(TokenKind::IDENT("STORAGE".to_string()));
            e.space();

            if let Some(ref storedtype) = n.storedtype {
                super::emit_type_name(e, storedtype);
            }
        }
        _ => {
            // Unknown item type
        }
    }

    e.group_end();
}
