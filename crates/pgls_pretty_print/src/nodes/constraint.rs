use pgls_query::protobuf::{ConstrType, Constraint};

use crate::Layout;

use super::prelude::*;

pub(super) fn emit_constraint(e: &mut EventEmitter, n: &Constraint) {
    e.group_start();
    // A broken constraint continues one level deeper than where it starts. The indent starts at
    // the first continuation line, so a column list on the first line breaks relative to it.
    let mut continued = false;

    match n.contype {
        x if x == ConstrType::ConstrNull as i32 => {
            e.token(TokenKind::NULL_KW);
        }
        x if x == ConstrType::ConstrNotnull as i32 => {
            // For domain constraints: CONSTRAINT name NOT NULL
            if !n.conname.is_empty() {
                e.token(TokenKind::CONSTRAINT_KW);
                e.space();
                emit_identifier_maybe_quoted(e, &n.conname);
                e.space();
            }
            e.token(TokenKind::NOT_KW);
            e.space();
            e.token(TokenKind::NULL_KW);
            // Note: keys field may contain 'value' for domains, but it's not emitted
        }
        x if x == ConstrType::ConstrDefault as i32 => {
            e.token(TokenKind::DEFAULT_KW);
            if let Some(ref raw_expr) = n.raw_expr {
                e.space();

                // Complex expressions (SubLink, BoolExpr, etc.) need parentheses in DEFAULT
                let needs_parens = matches!(
                    raw_expr.node.as_ref(),
                    Some(NodeEnum::SubLink(_))
                        | Some(NodeEnum::BoolExpr(_))
                        | Some(NodeEnum::AExpr(_))
                );

                if needs_parens {
                    e.token(TokenKind::L_PAREN);
                }
                emit_node(e, raw_expr);
                if needs_parens {
                    e.token(TokenKind::R_PAREN);
                }
            }
        }
        x if x == ConstrType::ConstrIdentity as i32 => {
            // CONSTRAINT name GENERATED {ALWAYS | BY DEFAULT} AS IDENTITY
            if !n.conname.is_empty() {
                e.token(TokenKind::CONSTRAINT_KW);
                e.space();
                emit_identifier_maybe_quoted(e, &n.conname);
                e.space();
            }

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

            continue_line(e, &mut continued);
            e.token(TokenKind::AS_KW);
            e.space();
            e.token(TokenKind::IDENTITY_KW);

            // Add sequence options if present (e.g., START 7 INCREMENT 5)
            if !n.options.is_empty() {
                continue_line(e, &mut continued);
                e.token(TokenKind::L_PAREN);
                for (idx, opt) in n.options.iter().enumerate() {
                    if idx > 0 {
                        e.line(LineType::SoftOrSpace);
                    }
                    if let Some(NodeEnum::DefElem(def)) = &opt.node {
                        // Emit sequence option like START 7, INCREMENT 5
                        emit_sequence_option(e, def);
                    }
                }
                e.token(TokenKind::R_PAREN);
            }
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
                emit_node(e, raw_expr);
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
                emit_identifier_maybe_quoted(e, &n.conname);
                e.space();
            }

            e.token(TokenKind::CHECK_KW);

            if let Some(ref raw_expr) = n.raw_expr {
                e.space();
                emit_check_condition(e, raw_expr);
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
                emit_identifier_maybe_quoted(e, &n.conname);
                e.space();
            }

            e.token(TokenKind::PRIMARY_KW);
            e.space();
            e.token(TokenKind::KEY_KW);

            if !n.keys.is_empty() {
                e.space();
                emit_parenthesized_list(e, &n.keys, emit_node);
            }

            // INCLUDE (columns)
            if !n.including.is_empty() {
                continue_line(e, &mut continued);
                e.token(TokenKind::INCLUDE_KW);
                e.space();
                emit_parenthesized_list(e, &n.including, |emitter, node| {
                    if let Some(pgls_query::NodeEnum::String(s)) = &node.node {
                        emit_identifier_maybe_quoted(emitter, &s.sval);
                    } else {
                        emit_node(emitter, node);
                    }
                });
            }

            if !n.indexname.is_empty() {
                e.space();
                e.token(TokenKind::USING_KW);
                e.space();
                e.token(TokenKind::INDEX_KW);
                e.space();
                emit_identifier_maybe_quoted(e, &n.indexname);
            }

            // USING INDEX TABLESPACE for PRIMARY KEY
            if !n.indexspace.is_empty() {
                continue_line(e, &mut continued);
                e.token(TokenKind::USING_KW);
                e.space();
                e.token(TokenKind::INDEX_KW);
                e.space();
                e.token(TokenKind::TABLESPACE_KW);
                e.space();
                emit_identifier_maybe_quoted(e, &n.indexspace);
            }

            emit_deferrable(e, n, &mut continued);
        }
        x if x == ConstrType::ConstrUnique as i32 => {
            // CONSTRAINT name UNIQUE (columns)
            if !n.conname.is_empty() {
                e.token(TokenKind::CONSTRAINT_KW);
                e.space();
                emit_identifier_maybe_quoted(e, &n.conname);
                e.space();
            }

            e.token(TokenKind::UNIQUE_KW);

            // For a table constraint the qualifier sits between the keyword and the column list,
            // unlike CREATE INDEX where it follows the list. Dropping it would let two NULLs
            // coexist where the schema forbids it.
            if n.nulls_not_distinct {
                e.space();
                e.token(TokenKind::NULLS_KW);
                e.space();
                e.token(TokenKind::NOT_KW);
                e.space();
                e.token(TokenKind::DISTINCT_KW);
            }

            if !n.keys.is_empty() {
                e.space();
                emit_parenthesized_list(e, &n.keys, emit_node);
            }

            // INCLUDE (columns)
            if !n.including.is_empty() {
                continue_line(e, &mut continued);
                e.token(TokenKind::INCLUDE_KW);
                e.space();
                emit_parenthesized_list(e, &n.including, |emitter, node| {
                    if let Some(pgls_query::NodeEnum::String(s)) = &node.node {
                        emit_identifier_maybe_quoted(emitter, &s.sval);
                    } else {
                        emit_node(emitter, node);
                    }
                });
            }

            if !n.indexname.is_empty() {
                e.space();
                e.token(TokenKind::USING_KW);
                e.space();
                e.token(TokenKind::INDEX_KW);
                e.space();
                emit_identifier_maybe_quoted(e, &n.indexname);
            }

            // USING INDEX TABLESPACE for UNIQUE
            if !n.indexspace.is_empty() {
                continue_line(e, &mut continued);
                e.token(TokenKind::USING_KW);
                e.space();
                e.token(TokenKind::INDEX_KW);
                e.space();
                e.token(TokenKind::TABLESPACE_KW);
                e.space();
                emit_identifier_maybe_quoted(e, &n.indexspace);
            }

            emit_deferrable(e, n, &mut continued);
        }
        x if x == ConstrType::ConstrExclusion as i32 => {
            // CONSTRAINT name EXCLUDE [USING method] (exclusion_list) [WHERE (predicate)]
            if !n.conname.is_empty() {
                e.token(TokenKind::CONSTRAINT_KW);
                e.space();
                emit_identifier_maybe_quoted(e, &n.conname);
                e.space();
            }

            e.token(TokenKind::EXCLUDE_KW);

            if !n.access_method.is_empty() {
                e.space();
                e.token(TokenKind::USING_KW);
                e.space();
                emit_identifier_maybe_quoted(e, &n.access_method);
            }

            if !n.exclusions.is_empty() {
                e.space();
                emit_parenthesized_list(e, &n.exclusions, emit_exclusion);
            }

            // INCLUDE (columns)
            if !n.including.is_empty() {
                continue_line(e, &mut continued);
                e.token(TokenKind::INCLUDE_KW);
                e.space();
                emit_parenthesized_list(e, &n.including, |emitter, node| {
                    if let Some(pgls_query::NodeEnum::String(s)) = &node.node {
                        emit_identifier_maybe_quoted(emitter, &s.sval);
                    } else {
                        emit_node(emitter, node);
                    }
                });
            }

            if let Some(ref where_clause) = n.where_clause {
                e.space();
                e.token(TokenKind::WHERE_KW);
                e.space();
                emit_parenthesized_condition(e, where_clause);
            }

            emit_deferrable(e, n, &mut continued);
        }
        x if x == ConstrType::ConstrForeign as i32 => {
            // CONSTRAINT name FOREIGN KEY (fk_attrs) REFERENCES pktable (pk_attrs) [actions]
            if !n.conname.is_empty() {
                e.token(TokenKind::CONSTRAINT_KW);
                e.space();
                emit_identifier_maybe_quoted(e, &n.conname);
                e.space();
            }

            // Table-level constraint has FOREIGN KEY (...)
            // Column-level constraint just has REFERENCES
            if !n.fk_attrs.is_empty() {
                e.token(TokenKind::FOREIGN_KW);
                e.space();
                e.token(TokenKind::KEY_KW);
                e.space();
                emit_parenthesized_list(e, &n.fk_attrs, emit_node);
                continue_line(e, &mut continued);
            }

            e.token(TokenKind::REFERENCES_KW);

            if let Some(ref pktable) = n.pktable {
                e.space();
                super::emit_range_var(e, pktable);

                if !n.pk_attrs.is_empty() {
                    e.space();
                    emit_parenthesized_list(e, &n.pk_attrs, emit_node);
                }
            }

            // MATCH clause
            if !n.fk_matchtype.is_empty() {
                match n.fk_matchtype.as_str() {
                    "f" => {
                        continue_line(e, &mut continued);
                        e.token(TokenKind::MATCH_KW);
                        e.space();
                        e.token(TokenKind::FULL_KW);
                    }
                    "p" => {
                        continue_line(e, &mut continued);
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

            // ON DELETE action - allow line break before
            if !n.fk_del_action.is_empty() && n.fk_del_action != "a" {
                continue_line(e, &mut continued);
                emit_foreign_key_action(e, &n.fk_del_action, "DELETE", &n.fk_del_set_cols);
            }

            // ON UPDATE action - allow line break before
            if !n.fk_upd_action.is_empty() && n.fk_upd_action != "a" {
                continue_line(e, &mut continued);
                emit_foreign_key_action(e, &n.fk_upd_action, "UPDATE", &[]);
            }

            // DEFERRABLE - allow line break before
            if n.deferrable {
                continue_line(e, &mut continued);
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
                continue_line(e, &mut continued);
                e.token(TokenKind::NOT_KW);
                e.space();
                e.token(TokenKind::VALID_KW);
            }
        }
        x if x == ConstrType::ConstrAttrDeferrable as i32 => {
            // DEFERRABLE constraint attribute
            e.token(TokenKind::DEFERRABLE_KW);
        }
        x if x == ConstrType::ConstrAttrNotDeferrable as i32 => {
            // NOT DEFERRABLE constraint attribute
            e.token(TokenKind::NOT_KW);
            e.space();
            e.token(TokenKind::DEFERRABLE_KW);
        }
        x if x == ConstrType::ConstrAttrDeferred as i32 => {
            // INITIALLY DEFERRED constraint attribute
            e.token(TokenKind::INITIALLY_KW);
            e.space();
            e.token(TokenKind::DEFERRED_KW);
        }
        x if x == ConstrType::ConstrAttrImmediate as i32 => {
            // INITIALLY IMMEDIATE constraint attribute
            e.token(TokenKind::INITIALLY_KW);
            e.space();
            e.token(TokenKind::IMMEDIATE_KW);
        }
        _ => {
            // Unknown constraint type - emit placeholder
            e.token(TokenKind::IDENT(format!(
                "UNKNOWN_CONSTRAINT_{}",
                n.contype
            )));
        }
    }

    if continued {
        e.indent_end();
    }
    e.group_end();
}

/// Emits the parenthesized condition of a CHECK constraint.
///
/// Expanded layout breaks between clauses, and a constraint is not one: its condition stays on one
/// line when it fits. Once it breaks, expanded layout still puts each operand of its AND or OR chain
/// on its own line, so the chain breaks with the parentheses rather than on its own.
fn emit_check_condition(e: &mut EventEmitter, condition: &Node) {
    let expanded = matches!(e.config().layout, Layout::Expanded);

    e.with_layout(Layout::Fit, |e| {
        emit_parenthesized_condition_with(e, condition, |e, node| match node {
            NodeEnum::BoolExpr(n) if expanded => {
                super::emit_bool_expr_as(e, n, super::OperandGroup::Enclosing);
            }
            _ => emit_node_enum(e, node),
        });
    });
}

/// Starts a continuation line of a constraint, which is indented once whatever their number.
fn continue_line(e: &mut EventEmitter, continued: &mut bool) {
    if !*continued {
        e.indent_start();
        *continued = true;
    }
    e.line(LineType::SoftOrSpace);
}

/// `INITIALLY DEFERRED` implies `DEFERRABLE` in the AST, so emitting both round-trips either spelling.
fn emit_deferrable(e: &mut EventEmitter, n: &Constraint, continued: &mut bool) {
    if n.deferrable {
        continue_line(e, continued);
        e.token(TokenKind::DEFERRABLE_KW);

        if n.initdeferred {
            e.space();
            e.token(TokenKind::INITIALLY_KW);
            e.space();
            e.token(TokenKind::DEFERRED_KW);
        }
    }
}

/// Emits one element of an exclusion constraint: `column WITH operator`.
fn emit_exclusion(e: &mut EventEmitter, exclusion: &pgls_query::Node) {
    let Some(NodeEnum::List(exclusion_list)) = exclusion.node.as_ref() else {
        emit_node(e, exclusion);
        return;
    };
    debug_assert!(exclusion_list.items.len() >= 2);

    if let Some(index_elem) = exclusion_list.items.first() {
        emit_node(e, index_elem);
    }

    if let Some(operators) = exclusion_list.items.get(1) {
        e.space();
        e.token(TokenKind::WITH_KW);
        e.space();

        match operators.node.as_ref() {
            Some(pgls_query::NodeEnum::List(op_list)) => {
                for (op_idx, op) in op_list.items.iter().enumerate() {
                    if op_idx > 0 {
                        e.token(TokenKind::COMMA);
                        e.space();
                    }
                    emit_operator_part(e, op);
                }
            }
            _ => emit_operator_part(e, operators),
        }
    }
}

fn emit_foreign_key_action(
    e: &mut EventEmitter,
    action: &str,
    event: &str,
    set_cols: &[pgls_query::protobuf::Node],
) {
    // Caller handles line break - we just emit the action
    e.token(TokenKind::ON_KW);
    e.space();
    emit_keyword(e, event);
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
                emit_parenthesized_list(e, set_cols, emit_node);
            }
        }
        "d" => {
            e.token(TokenKind::SET_KW);
            e.space();
            e.token(TokenKind::DEFAULT_KW);

            if !set_cols.is_empty() {
                e.space();
                emit_parenthesized_list(e, set_cols, emit_node);
            }
        }
        "a" => {
            // NO ACTION is the default - emit explicitly when caller wants it
            e.token(TokenKind::NO_KW);
            e.space();
            e.token(TokenKind::ACTION_KW);
        }
        _ => {}
    }
}
