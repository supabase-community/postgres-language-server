use pgls_query::NodeEnum;
use pgls_query::protobuf::{DefElem, DefineStmt, List, Node, ObjectType};

use super::string::emit_identifier_maybe_quoted;
use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
    nodes::node_list::{emit_comma_separated_list, emit_dot_separated_list},
};

/// Emit collation definition (FROM clause or parenthesized options)
fn emit_collation_definition(e: &mut EventEmitter, definition: &[Node]) {
    // Check if there's a FROM element with List argument (special FROM syntax)
    // If FROM has a TypeName argument, it's the parenthesized option syntax
    let has_special_from = definition.iter().any(|def_node| {
        if let Some(pgls_query::NodeEnum::DefElem(def_elem)) = &def_node.node
            && def_elem.defname == "from"
            && let Some(ref arg) = def_elem.arg
        {
            return matches!(arg.node, Some(pgls_query::NodeEnum::List(_)));
        }
        false
    });

    if has_special_from {
        // Special FROM collation syntax: CREATE COLLATION name FROM other_collation
        for def_node in definition {
            if let Some(pgls_query::NodeEnum::DefElem(def_elem)) = &def_node.node
                && def_elem.defname == "from"
            {
                e.space();
                e.token(TokenKind::FROM_KW);
                e.space();
                // The arg is a List containing String nodes with the collation name
                if let Some(ref arg) = def_elem.arg
                    && let Some(pgls_query::NodeEnum::List(list)) = &arg.node
                {
                    // Emit the strings in the list as dot-separated qualified name with quotes
                    for (i, item) in list.items.iter().enumerate() {
                        if i > 0 {
                            e.token(TokenKind::DOT);
                        }
                        if let Some(pgls_query::NodeEnum::String(s)) = &item.node {
                            super::emit_string_identifier(e, s);
                        } else {
                            super::emit_node(item, e);
                        }
                    }
                }
            }
        }
    } else {
        // Parenthesized options syntax - emit all options in single parentheses
        e.space();
        e.token(TokenKind::L_PAREN);
        e.indent_start();
        e.line(LineType::Soft);
        let mut first = true;
        for def_node in definition {
            if let Some(pgls_query::NodeEnum::DefElem(_)) = &def_node.node {
                if !first {
                    e.token(TokenKind::COMMA);
                    e.line(LineType::SoftOrSpace);
                }
                super::emit_node(def_node, e);
                first = false;
            }
        }
        e.indent_end();
        e.line(LineType::Soft);
        e.token(TokenKind::R_PAREN);
    }
}

pub(super) fn emit_define_stmt(e: &mut EventEmitter, n: &DefineStmt) {
    e.group_start(GroupKind::DefineStmt);

    e.token(TokenKind::CREATE_KW);

    if n.replace {
        e.space();
        e.token(TokenKind::OR_KW);
        e.space();
        e.token(TokenKind::REPLACE_KW);
    }

    e.space();

    let kind = n.kind();
    match kind {
        ObjectType::ObjectAggregate => e.token(TokenKind::AGGREGATE_KW),
        ObjectType::ObjectOperator => e.token(TokenKind::OPERATOR_KW),
        ObjectType::ObjectType => e.token(TokenKind::TYPE_KW),
        ObjectType::ObjectCollation => e.token(TokenKind::COLLATION_KW),
        ObjectType::ObjectTsdictionary => {
            e.token(TokenKind::TEXT_KW);
            e.space();
            e.token(TokenKind::SEARCH_KW);
            e.space();
            e.token(TokenKind::DICTIONARY_KW);
        }
        ObjectType::ObjectTsconfiguration => {
            e.token(TokenKind::TEXT_KW);
            e.space();
            e.token(TokenKind::SEARCH_KW);
            e.space();
            e.token(TokenKind::CONFIGURATION_KW);
        }
        ObjectType::ObjectTsparser => {
            e.token(TokenKind::TEXT_KW);
            e.space();
            e.token(TokenKind::SEARCH_KW);
            e.space();
            e.token(TokenKind::PARSER_KW);
        }
        ObjectType::ObjectTstemplate => {
            e.token(TokenKind::TEXT_KW);
            e.space();
            e.token(TokenKind::SEARCH_KW);
            e.space();
            e.token(TokenKind::TEMPLATE_KW);
        }
        _ => e.token(TokenKind::IDENT(format!("{kind:?}"))),
    }

    if n.if_not_exists {
        e.space();
        e.token(TokenKind::IF_KW);
        e.space();
        e.token(TokenKind::NOT_KW);
        e.space();
        e.token(TokenKind::EXISTS_KW);
    }

    let is_operator = kind == ObjectType::ObjectOperator;

    if !n.defnames.is_empty() {
        e.space();
        if is_operator {
            emit_operator_name(e, &n.defnames);
        } else {
            emit_dot_separated_list(e, &n.defnames);
        }
    }

    // Args (for operators/aggregates) - keep paren on same line as name
    if !n.args.is_empty() {
        e.space();
        e.token(TokenKind::L_PAREN);
        e.indent_start();
        e.line(LineType::Soft);
        if kind == ObjectType::ObjectAggregate {
            emit_aggregate_args(e, &n.args);
        } else {
            emit_comma_separated_list(e, &n.args, super::emit_node);
        }
        e.indent_end();
        e.line(LineType::Soft);
        e.token(TokenKind::R_PAREN);
    }

    if is_operator {
        if !n.definition.is_empty() {
            e.space();
            e.token(TokenKind::L_PAREN);
            emit_comma_separated_list(e, &n.definition, |node, emitter| {
                if let Some(NodeEnum::DefElem(def)) = node.node.as_ref() {
                    emit_operator_def_elem(emitter, def);
                } else {
                    super::emit_node(node, emitter);
                }
            });
            e.token(TokenKind::R_PAREN);
        }
    } else if kind == ObjectType::ObjectCollation && !n.definition.is_empty() {
        emit_collation_definition(e, &n.definition);
    } else if !n.definition.is_empty() {
        e.space();
        e.token(TokenKind::L_PAREN);
        e.indent_start();
        e.line(LineType::Soft);
        emit_comma_separated_list(e, &n.definition, super::emit_node);
        e.indent_end();
        e.line(LineType::Soft);
        e.token(TokenKind::R_PAREN);
    }

    e.token(TokenKind::SEMICOLON);
    e.group_end();
}

fn emit_operator_name(e: &mut EventEmitter, defnames: &[Node]) {
    for (idx, node) in defnames.iter().enumerate() {
        if idx > 0 {
            e.token(TokenKind::DOT);
        }

        match node.node.as_ref() {
            Some(NodeEnum::String(s)) => {
                if idx == defnames.len() - 1 {
                    e.token(TokenKind::IDENT(s.sval.clone()));
                } else {
                    emit_identifier_maybe_quoted(e, &s.sval);
                }
            }
            _ => super::emit_node(node, e),
        }
    }
}

fn emit_operator_def_elem(e: &mut EventEmitter, def: &DefElem) {
    let name = def.defname.to_ascii_uppercase();
    e.token(TokenKind::IDENT(name));

    if let Some(ref arg) = def.arg {
        e.space();
        e.token(TokenKind::IDENT("=".to_string()));
        e.space();
        emit_operator_def_arg(e, arg);
    }
}

fn emit_operator_def_arg(e: &mut EventEmitter, arg: &Node) {
    match arg.node.as_ref() {
        Some(NodeEnum::List(list)) => emit_operator_list(e, list),
        _ => super::emit_node(arg, e),
    }
}

fn emit_operator_list(e: &mut EventEmitter, list: &List) {
    for (idx, item) in list.items.iter().enumerate() {
        if idx > 0 {
            e.token(TokenKind::DOT);
        }

        match item.node.as_ref() {
            Some(NodeEnum::String(s)) => e.token(TokenKind::IDENT(s.sval.clone())),
            _ => super::emit_node(item, e),
        }
    }
}

fn emit_aggregate_args(e: &mut EventEmitter, args: &[Node]) {
    // Aggregate args can have ORDER BY for ordered-set aggregates
    // The args list is: [direct_args..., sentinel_int, order_by_args...]
    // where sentinel_int indicates the number of direct args
    // A negative integer means ORDER BY args follow, positive is a type count

    let mut first = true;
    let mut i = 0;

    while i < args.len() {
        let arg = &args[i];

        match arg.node.as_ref() {
            Some(NodeEnum::TypeName(type_name)) => {
                if !first {
                    e.token(TokenKind::COMMA);
                    e.line(LineType::SoftOrSpace);
                }
                super::emit_type_name(e, type_name);
                first = false;
            }
            Some(NodeEnum::Integer(int_node)) => {
                // An integer signals ORDER BY boundary
                // Positive integer followed by more args means ORDER BY
                // Negative integers are skipped
                if int_node.ival >= 0 && i + 1 < args.len() {
                    // There are more args after this, so emit ORDER BY
                    e.space();
                    e.token(TokenKind::ORDER_KW);
                    e.space();
                    e.token(TokenKind::BY_KW);
                    e.space();
                    first = true; // Reset for ORDER BY args
                }
                // else: skip the integer sentinel
            }
            None => {
                // A None node in aggregate args represents * (any type)
                // e.g., CREATE AGGREGATE my_agg (*)
                if !first {
                    e.token(TokenKind::COMMA);
                    e.line(LineType::SoftOrSpace);
                }
                e.token(TokenKind::IDENT("*".to_string()));
                first = false;
            }
            Some(_) => {
                if !first {
                    e.token(TokenKind::COMMA);
                    e.line(LineType::SoftOrSpace);
                }
                super::emit_node(arg, e);
                first = false;
            }
        }
        i += 1;
    }
}
