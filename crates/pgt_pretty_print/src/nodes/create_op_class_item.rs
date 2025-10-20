use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind};
use pgt_query::protobuf::CreateOpClassItem;

use super::node_list::{emit_comma_separated_list, emit_dot_separated_list};
use super::object_with_args::{emit_object_name_only, emit_object_with_args};

pub(super) fn emit_create_op_class_item(e: &mut EventEmitter, n: &CreateOpClassItem) {
    e.group_start(GroupKind::CreateOpClassItem);

    // itemtype: 1=OPERATOR, 2=FUNCTION, 3=STORAGE
    match n.itemtype {
        1 => {
            // OPERATOR strategy_number operator_name (arg_types)
            e.token(TokenKind::IDENT("OPERATOR".to_string()));
            e.space();
            e.token(TokenKind::IDENT(n.number.to_string()));
            e.space();

            if let Some(ref name) = n.name {
                // Emit operator name
                emit_object_name_only(e, name);

                // Emit argument types in parentheses
                // Use a tight group to keep the type list together
                if !name.objargs.is_empty() {
                    e.space();
                    e.group_start(GroupKind::CreateOpClassItem);
                    e.token(TokenKind::L_PAREN);
                    emit_comma_separated_list(e, &name.objargs, super::emit_node);
                    e.token(TokenKind::R_PAREN);
                    e.group_end();
                }
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
            // FUNCTION support_number function_name
            e.token(TokenKind::IDENT("FUNCTION".to_string()));
            e.space();
            e.token(TokenKind::IDENT(n.number.to_string()));
            e.space();

            if let Some(ref name) = n.name {
                emit_object_with_args(e, name);
            }

            // Optional class_args for function arguments
            if !n.class_args.is_empty() {
                e.space();
                e.token(TokenKind::L_PAREN);
                emit_comma_separated_list(e, &n.class_args, super::emit_node);
                e.token(TokenKind::R_PAREN);
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
