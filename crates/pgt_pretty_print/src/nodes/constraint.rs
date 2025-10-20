use pgt_query::protobuf::{ConstrType, Constraint};

use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind};

use super::node_list::emit_comma_separated_list;
use super::string::emit_identifier;

pub(super) fn emit_constraint(e: &mut EventEmitter, n: &Constraint) {
    e.group_start(GroupKind::Constraint);

    match n.contype {
        x if x == ConstrType::ConstrNull as i32 => {
            e.token(TokenKind::NULL_KW);
        }
        x if x == ConstrType::ConstrNotnull as i32 => {
            e.token(TokenKind::NOT_KW);
            e.space();
            e.token(TokenKind::NULL_KW);
        }
        x if x == ConstrType::ConstrDefault as i32 => {
            e.token(TokenKind::DEFAULT_KW);
            if let Some(ref raw_expr) = n.raw_expr {
                e.space();
                super::emit_node(raw_expr, e);
            }
        }
        x if x == ConstrType::ConstrIdentity as i32 => {
            // GENERATED {ALWAYS | BY DEFAULT} AS IDENTITY
            e.token(TokenKind::GENERATED_KW);
            e.space();

            // generated_when is a string like "a" (ALWAYS) or "d" (BY DEFAULT)
            match n.generated_when.as_str() {
                "a" => {
                    e.token(TokenKind::ALWAYS_KW);
                }
                "d" => {
                    e.token(TokenKind::BY_KW);
                    e.space();
                    e.token(TokenKind::DEFAULT_KW);
                }
                _ => {
                    // Default to ALWAYS if not specified
                    e.token(TokenKind::ALWAYS_KW);
                }
            }

            e.space();
            e.token(TokenKind::AS_KW);
            e.space();
            e.token(TokenKind::IDENTITY_KW);

            // TODO: Add sequence options from n.options if present
        }
        x if x == ConstrType::ConstrGenerated as i32 => {
            // GENERATED ALWAYS AS (expr) STORED
            e.token(TokenKind::GENERATED_KW);
            e.space();
            e.token(TokenKind::ALWAYS_KW);
            e.space();
            e.token(TokenKind::AS_KW);

            if let Some(ref raw_expr) = n.raw_expr {
                e.space();
                e.token(TokenKind::L_PAREN);
                super::emit_node(raw_expr, e);
                e.token(TokenKind::R_PAREN);
            }

            e.space();
            e.token(TokenKind::STORED_KW);
        }
        x if x == ConstrType::ConstrCheck as i32 => {
            // CONSTRAINT name CHECK (expr) [NO INHERIT]
            if !n.conname.is_empty() {
                e.token(TokenKind::CONSTRAINT_KW);
                e.space();
                emit_identifier(e, &n.conname);
                e.space();
            }

            e.token(TokenKind::CHECK_KW);

            if let Some(ref raw_expr) = n.raw_expr {
                e.space();
                e.token(TokenKind::L_PAREN);
                super::emit_node(raw_expr, e);
                e.token(TokenKind::R_PAREN);
            }

            if n.is_no_inherit {
                e.space();
                e.token(TokenKind::NO_KW);
                e.space();
                e.token(TokenKind::INHERIT_KW);
            }

            if !n.initially_valid {
                e.space();
                e.token(TokenKind::NOT_KW);
                e.space();
                e.token(TokenKind::VALID_KW);
            }
        }
        x if x == ConstrType::ConstrPrimary as i32 => {
            // CONSTRAINT name PRIMARY KEY (columns)
            if !n.conname.is_empty() {
                e.token(TokenKind::CONSTRAINT_KW);
                e.space();
                emit_identifier(e, &n.conname);
                e.space();
            }

            e.token(TokenKind::PRIMARY_KW);
            e.space();
            e.token(TokenKind::KEY_KW);

            if !n.keys.is_empty() {
                e.space();
                e.token(TokenKind::L_PAREN);
                emit_comma_separated_list(e, &n.keys, super::emit_node);
                e.token(TokenKind::R_PAREN);
            }

            if !n.indexname.is_empty() {
                e.space();
                e.token(TokenKind::USING_KW);
                e.space();
                e.token(TokenKind::INDEX_KW);
                e.space();
                emit_identifier(e, &n.indexname);
            }
        }
        x if x == ConstrType::ConstrUnique as i32 => {
            // CONSTRAINT name UNIQUE (columns)
            if !n.conname.is_empty() {
                e.token(TokenKind::CONSTRAINT_KW);
                e.space();
                emit_identifier(e, &n.conname);
                e.space();
            }

            e.token(TokenKind::UNIQUE_KW);

            if !n.keys.is_empty() {
                e.space();
                e.token(TokenKind::L_PAREN);
                emit_comma_separated_list(e, &n.keys, super::emit_node);
                e.token(TokenKind::R_PAREN);
            }

            if !n.indexname.is_empty() {
                e.space();
                e.token(TokenKind::USING_KW);
                e.space();
                e.token(TokenKind::INDEX_KW);
                e.space();
                emit_identifier(e, &n.indexname);
            }
        }
        x if x == ConstrType::ConstrExclusion as i32 => {
            // CONSTRAINT name EXCLUDE [USING method] (exclusion_list) [WHERE (predicate)]
            if !n.conname.is_empty() {
                e.token(TokenKind::CONSTRAINT_KW);
                e.space();
                emit_identifier(e, &n.conname);
                e.space();
            }

            e.token(TokenKind::EXCLUDE_KW);

            if !n.access_method.is_empty() {
                e.space();
                e.token(TokenKind::USING_KW);
                e.space();
                e.token(TokenKind::IDENT(n.access_method.clone()));
            }

            if !n.exclusions.is_empty() {
                e.space();
                e.token(TokenKind::L_PAREN);
                emit_comma_separated_list(e, &n.exclusions, super::emit_node);
                e.token(TokenKind::R_PAREN);
            }

            if let Some(ref where_clause) = n.where_clause {
                e.space();
                e.token(TokenKind::WHERE_KW);
                e.space();
                e.token(TokenKind::L_PAREN);
                super::emit_node(where_clause, e);
                e.token(TokenKind::R_PAREN);
            }
        }
        x if x == ConstrType::ConstrForeign as i32 => {
            // CONSTRAINT name FOREIGN KEY (fk_attrs) REFERENCES pktable (pk_attrs) [actions]
            if !n.conname.is_empty() {
                e.token(TokenKind::CONSTRAINT_KW);
                e.space();
                emit_identifier(e, &n.conname);
                e.space();
            }

            // Table-level constraint has FOREIGN KEY (...)
            // Column-level constraint just has REFERENCES
            if !n.fk_attrs.is_empty() {
                e.token(TokenKind::FOREIGN_KW);
                e.space();
                e.token(TokenKind::KEY_KW);
                e.space();
                e.token(TokenKind::L_PAREN);
                emit_comma_separated_list(e, &n.fk_attrs, super::emit_node);
                e.token(TokenKind::R_PAREN);
                e.space();
            }

            e.token(TokenKind::REFERENCES_KW);

            if let Some(ref pktable) = n.pktable {
                e.space();
                super::emit_range_var(e, pktable);

                if !n.pk_attrs.is_empty() {
                    e.space();
                    e.token(TokenKind::L_PAREN);
                    emit_comma_separated_list(e, &n.pk_attrs, super::emit_node);
                    e.token(TokenKind::R_PAREN);
                }
            }

            // MATCH clause
            if !n.fk_matchtype.is_empty() {
                match n.fk_matchtype.as_str() {
                    "f" => {
                        e.space();
                        e.token(TokenKind::MATCH_KW);
                        e.space();
                        e.token(TokenKind::FULL_KW);
                    }
                    "p" => {
                        e.space();
                        e.token(TokenKind::MATCH_KW);
                        e.space();
                        e.token(TokenKind::PARTIAL_KW);
                    }
                    "s" => {
                        // MATCH SIMPLE is the default, usually not emitted
                    }
                    _ => {}
                }
            }

            // ON DELETE action
            if !n.fk_del_action.is_empty() {
                emit_foreign_key_action(e, &n.fk_del_action, "DELETE", &n.fk_del_set_cols);
            }

            // ON UPDATE action
            if !n.fk_upd_action.is_empty() {
                emit_foreign_key_action(e, &n.fk_upd_action, "UPDATE", &[]);
            }

            // DEFERRABLE
            if n.deferrable {
                e.space();
                e.token(TokenKind::DEFERRABLE_KW);

                if n.initdeferred {
                    e.space();
                    e.token(TokenKind::INITIALLY_KW);
                    e.space();
                    e.token(TokenKind::DEFERRED_KW);
                } else {
                    e.space();
                    e.token(TokenKind::INITIALLY_KW);
                    e.space();
                    e.token(TokenKind::IMMEDIATE_KW);
                }
            }

            if !n.initially_valid {
                e.space();
                e.token(TokenKind::NOT_KW);
                e.space();
                e.token(TokenKind::VALID_KW);
            }
        }
        _ => {
            // Unknown constraint type - emit placeholder
            e.token(TokenKind::IDENT(format!(
                "UNKNOWN_CONSTRAINT_{}",
                n.contype
            )));
        }
    }

    e.group_end();
}

fn emit_foreign_key_action(
    e: &mut EventEmitter,
    action: &str,
    event: &str,
    set_cols: &[pgt_query::protobuf::Node],
) {
    if action == "a" {
        // NO ACTION is the default, usually not emitted
        return;
    }

    e.space();
    e.token(TokenKind::ON_KW);
    e.space();
    e.token(TokenKind::IDENT(event.to_string()));
    e.space();

    match action {
        "r" => {
            e.token(TokenKind::RESTRICT_KW);
        }
        "c" => {
            e.token(TokenKind::CASCADE_KW);
        }
        "n" => {
            e.token(TokenKind::SET_KW);
            e.space();
            e.token(TokenKind::NULL_KW);

            if !set_cols.is_empty() {
                e.space();
                e.token(TokenKind::L_PAREN);
                emit_comma_separated_list(e, set_cols, super::emit_node);
                e.token(TokenKind::R_PAREN);
            }
        }
        "d" => {
            e.token(TokenKind::SET_KW);
            e.space();
            e.token(TokenKind::DEFAULT_KW);

            if !set_cols.is_empty() {
                e.space();
                e.token(TokenKind::L_PAREN);
                emit_comma_separated_list(e, set_cols, super::emit_node);
                e.token(TokenKind::R_PAREN);
            }
        }
        _ => {}
    }
}
