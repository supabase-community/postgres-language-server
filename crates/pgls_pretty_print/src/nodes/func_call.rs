use crate::{
    CommaStyle, TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
    nodes::node_list::{emit_comma_separated_list, emit_parenthesized_list},
};
use pgls_query::{
    Node, NodeEnum,
    protobuf::{CoercionForm, FuncCall, a_const::Val},
};

use super::string::{
    emit_identifier, needs_quoting, needs_quoting_as_column_name,
    needs_quoting_as_function_or_type_name, needs_quoting_as_label,
};

/// Get the last part of the function name (the actual function name without schema)
fn get_last_func_name(n: &FuncCall) -> Option<&str> {
    n.funcname.last().and_then(|node| {
        if let Some(pgls_query::NodeEnum::String(s)) = &node.node {
            Some(s.sval.as_str())
        } else {
            None
        }
    })
}

/// Functions whose arguments form key/value pairs by default, without configuration.
/// These are PostgreSQL's JSON object constructors: their arguments alternate between
/// keys and values, so a group size of two keeps each pair together when wrapping.
/// A configured group size always takes precedence, so `jsonb_build_object: 1` opts out.
const DEFAULT_GROUPED_FUNCTIONS: [&str; 2] = ["json_build_object", "jsonb_build_object"];
const DEFAULT_ARGUMENT_GROUP_SIZE: usize = 2;

fn configured_argument_group_size(e: &EventEmitter, n: &FuncCall) -> Option<usize> {
    let name = get_last_func_name(n)?;
    let groups = &e.config().function_argument_groups;

    // Fast path for the common case: nothing is configured and the function is not one of
    // the defaults. Avoids the lowercase allocation for every function call in the file.
    if groups.is_empty()
        && !DEFAULT_GROUPED_FUNCTIONS
            .iter()
            .any(|f| f.eq_ignore_ascii_case(name))
    {
        return None;
    }

    let name = name.to_lowercase();
    if let Some(size) = groups.get(&name) {
        return Some(*size);
    }
    DEFAULT_GROUPED_FUNCTIONS
        .contains(&name.as_str())
        .then_some(DEFAULT_ARGUMENT_GROUP_SIZE)
}

fn emit_grouped_arguments(e: &mut EventEmitter, args: &[pgls_query::Node], group_size: usize) {
    let leading = matches!(e.config().comma_style, CommaStyle::Leading);

    for (group_index, arguments) in args.chunks(group_size).enumerate() {
        if group_index > 0 {
            emit_grouped_comma(e, leading, arguments.first(), LineType::SoftOrSpace);
        }

        e.group_start(GroupKind::FunctionArgumentGroup);
        for (argument_index, argument) in arguments.iter().enumerate() {
            if argument_index > 0 {
                // `Fill` keeps the pair on one line when it fits and falls back to one
                // argument per line when it does not, instead of exceeding the line width.
                emit_grouped_comma(e, leading, Some(argument), LineType::Fill);
            }
            super::emit_node(argument, e);
        }
        e.group_end();
    }
}

/// Emit the comma before a grouped argument, honoring `commaStyle` the same way
/// `emit_comma_separated_list_with_spacing` does: leading puts the break opportunity
/// before the comma (`\n, column`), trailing puts it after.
fn emit_grouped_comma(
    e: &mut EventEmitter,
    leading: bool,
    argument: Option<&pgls_query::Node>,
    break_type: LineType,
) {
    if leading {
        if let Some(location) = argument
            .and_then(|arg| arg.node.as_ref())
            .and_then(|node| crate::codegen::node_location::node_location(&node.to_ref()))
        {
            e.take_own_line_leading_comments_at(location);
        }
        e.line(LineType::Soft);
        e.token(TokenKind::COMMA);
        e.space();
    } else {
        e.token(TokenKind::COMMA);
        e.line(break_type);
    }
}

pub(super) fn emit_func_call(e: &mut EventEmitter, n: &FuncCall) {
    e.group_start(GroupKind::FuncCall);
    if n.over.is_some() {
        e.group_start(GroupKind::List);
    }

    // The grammar turns SQL-standard syntax such as `EXTRACT(year FROM d)` or `ts AT TIME ZONE z`
    // into a call to a `pg_catalog` function, flagged as written in that syntax. Anything else was
    // written as a call, and is written back as one with its name exactly as qualified.
    let is_sql_syntax = n.funcformat == CoercionForm::CoerceSqlSyntax as i32;
    if !(is_sql_syntax && emit_sql_syntax_call(e, n)) {
        emit_function_name(e, &n.funcname);
        emit_standard_function(e, n);
    }

    if n.over.is_some() {
        e.group_end();
    }

    if n.agg_within_group {
        debug_assert!(
            !n.agg_order.is_empty(),
            "ordered-set aggregate is missing ORDER BY list"
        );

        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::WITHIN_KW);
        e.space();
        e.token(TokenKind::GROUP_KW);
        e.space();
        // Wrap in group for compact formatting
        e.group_start(GroupKind::List);
        e.token(TokenKind::L_PAREN);
        e.line(LineType::Soft);
        e.indent_start();
        e.token(TokenKind::ORDER_KW);
        e.space();
        e.token(TokenKind::BY_KW);
        e.space();
        emit_comma_separated_list(e, &n.agg_order, super::emit_node);
        e.indent_end();
        e.line(LineType::Soft);
        e.token(TokenKind::R_PAREN);
        e.group_end();
    }

    if let Some(ref filter) = n.agg_filter {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::FILTER_KW);
        e.space();
        e.token(TokenKind::L_PAREN);
        e.token(TokenKind::WHERE_KW);
        super::emit_clause_condition(e, filter);
        e.token(TokenKind::R_PAREN);
    }

    // Handle OVER clause (window functions)
    if let Some(ref over) = n.over {
        e.space();
        e.token(TokenKind::OVER_KW);
        e.space();
        e.group_start(GroupKind::List);
        super::emit_window_def(e, over);
        e.group_end();
    }

    e.group_end();
}

// Standard function call: func(arg1, arg2, ...)
fn emit_standard_function(e: &mut EventEmitter, n: &FuncCall) {
    // Emit opening parenthesis
    e.token(TokenKind::L_PAREN);

    // Check if we have any content
    let has_content =
        n.agg_star || !n.args.is_empty() || (!n.agg_order.is_empty() && !n.agg_within_group);

    if has_content {
        e.line(LineType::Soft);
        e.indent_start();

        // Handle DISTINCT if present
        if n.agg_distinct && !n.args.is_empty() {
            e.token(TokenKind::DISTINCT_KW);
            e.space();
        }

        // Emit arguments
        if n.agg_star {
            e.token(TokenKind::IDENT("*".to_string()));
        } else if !n.args.is_empty() {
            // Handle VARIADIC - if func_variadic is true, prepend VARIADIC to the last argument
            if n.func_variadic && !n.args.is_empty() {
                // Emit non-variadic args first
                for (idx, arg) in n.args.iter().take(n.args.len() - 1).enumerate() {
                    if idx > 0 {
                        e.token(TokenKind::COMMA);
                        e.line(LineType::SoftOrSpace);
                    }
                    super::emit_node(arg, e);
                }

                if n.args.len() > 1 {
                    e.token(TokenKind::COMMA);
                    e.line(LineType::SoftOrSpace);
                }

                // Emit VARIADIC with the last argument
                e.token(TokenKind::VARIADIC_KW);
                e.space();
                super::emit_node(n.args.last().unwrap(), e);
            } else if let Some(group_size) = configured_argument_group_size(e, n) {
                emit_grouped_arguments(e, &n.args, group_size);
            } else {
                emit_comma_separated_list(e, &n.args, super::emit_node);
            }
        }

        // Handle ORDER BY inside function (for aggregates not using WITHIN GROUP)
        if !n.agg_order.is_empty() && !n.agg_within_group {
            if !n.args.is_empty() {
                e.space();
            }
            e.token(TokenKind::ORDER_KW);
            e.space();
            e.token(TokenKind::BY_KW);
            e.space();
            e.indent_start();
            emit_comma_separated_list(e, &n.agg_order, |node, emitter| {
                super::emit_node(node, emitter)
            });
            e.indent_end();
        }

        e.indent_end();
        e.line(LineType::Soft);
    }

    e.token(TokenKind::R_PAREN);
}

/// Emits a function name as written. A name the grammar would read as something else unquoted, such
/// as `coalesce` or a reserved word, is quoted.
fn emit_function_name(e: &mut EventEmitter, funcname: &[Node]) {
    let parts: Vec<&str> = funcname
        .iter()
        .filter_map(|node| match &node.node {
            Some(NodeEnum::String(s)) => Some(s.sval.as_str()),
            _ => None,
        })
        .collect();

    // The grammar qualifies the plain-call form of `json_object(...)` itself.
    if let ["pg_catalog", "json_object"] = parts.as_slice() {
        e.token(TokenKind::IDENT("json_object".to_string()));
        return;
    }

    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            e.token(TokenKind::DOT);
        }

        let is_unqualified = parts.len() == 1;
        let needs_quoting = if is_unqualified {
            // `overlay(a, b)` and `substring(a, b)` have a plain-call form in the grammar that
            // keeps the name unqualified.
            needs_quoting_as_function_or_type_name(part)
                && !matches!(*part, "overlay" | "substring")
        } else if index == parts.len() - 1 {
            // A qualified function name can be any word, keywords included.
            needs_quoting_as_label(part)
        } else {
            needs_quoting_as_column_name(part)
        };

        if needs_quoting {
            emit_identifier(e, part);
        } else {
            e.token(TokenKind::IDENT((*part).to_string()));
        }
    }
}

/// Emits a call written in SQL-standard syntax. Returns false, having emitted nothing, for a shape
/// it does not know, which is then written as a call to the `pg_catalog` function it stands for.
fn emit_sql_syntax_call(e: &mut EventEmitter, n: &FuncCall) -> bool {
    let name = match n.funcname.as_slice() {
        [schema, name] => match (&schema.node, &name.node) {
            (Some(NodeEnum::String(schema)), Some(NodeEnum::String(name)))
                if schema.sval == "pg_catalog" =>
            {
                name.sval.as_str()
            }
            _ => return false,
        },
        _ => return false,
    };
    let has_modifiers = n.agg_star
        || n.agg_distinct
        || n.func_variadic
        || !n.agg_order.is_empty()
        || n.agg_filter.is_some()
        || n.over.is_some();
    if has_modifiers {
        return false;
    }

    let args = n.args.as_slice();
    match (name, args) {
        ("extract", [field, source]) => {
            e.token(TokenKind::EXTRACT_KW);
            e.token(TokenKind::L_PAREN);
            emit_extract_field(e, field);
            e.space();
            e.token(TokenKind::FROM_KW);
            e.space();
            super::emit_node(source, e);
            e.token(TokenKind::R_PAREN);
        }
        ("overlay", [string, replacement, start, rest @ ..]) if rest.len() <= 1 => {
            e.token(TokenKind::OVERLAY_KW);
            e.token(TokenKind::L_PAREN);
            super::emit_node(string, e);
            e.space();
            e.token(TokenKind::PLACING_KW);
            e.space();
            super::emit_node(replacement, e);
            e.space();
            e.token(TokenKind::FROM_KW);
            e.space();
            super::emit_node(start, e);
            if let [length] = rest {
                e.space();
                e.token(TokenKind::FOR_KW);
                e.space();
                super::emit_node(length, e);
            }
            e.token(TokenKind::R_PAREN);
        }
        // Stored as position(string, substring), written POSITION(substring IN string). Both
        // operands are `b_expr`, which an operator expression is not.
        ("position", [string, substring]) => {
            e.token(TokenKind::POSITION_KW);
            e.token(TokenKind::L_PAREN);
            emit_parenthesized_unless_simple(e, substring);
            e.space();
            e.token(TokenKind::IN_KW);
            e.space();
            emit_parenthesized_unless_simple(e, string);
            e.token(TokenKind::R_PAREN);
        }
        ("substring", [string, start, rest @ ..]) if rest.len() <= 1 => {
            e.token(TokenKind::SUBSTRING_KW);
            e.token(TokenKind::L_PAREN);
            super::emit_node(string, e);
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::FROM_KW);
            e.space();
            super::emit_node(start, e);
            if let [length] = rest {
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::FOR_KW);
                e.space();
                super::emit_node(length, e);
            }
            e.token(TokenKind::R_PAREN);
        }
        // Stored as btrim(string[, characters]), written TRIM(BOTH characters FROM string).
        ("btrim" | "ltrim" | "rtrim", [string, rest @ ..]) if rest.len() <= 1 => {
            e.token(TokenKind::TRIM_KW);
            e.token(TokenKind::L_PAREN);
            match name {
                "ltrim" => {
                    e.token(TokenKind::LEADING_KW);
                    e.space();
                }
                "rtrim" => {
                    e.token(TokenKind::TRAILING_KW);
                    e.space();
                }
                _ => {}
            }
            if let [characters] = rest {
                super::emit_node(characters, e);
                e.space();
                e.token(TokenKind::FROM_KW);
                e.space();
            }
            super::emit_node(string, e);
            e.token(TokenKind::R_PAREN);
        }
        ("normalize", [string, rest @ ..]) => {
            let Some(form) = optional_normalization_form(rest) else {
                return false;
            };
            e.token(TokenKind::NORMALIZE_KW);
            e.token(TokenKind::L_PAREN);
            super::emit_node(string, e);
            if let Some(form) = form {
                e.token(TokenKind::COMMA);
                e.space();
                e.token(form);
            }
            e.token(TokenKind::R_PAREN);
        }
        ("is_normalized", [string, rest @ ..]) => {
            let Some(form) = optional_normalization_form(rest) else {
                return false;
            };
            e.token(TokenKind::L_PAREN);
            emit_parenthesized_unless_simple(e, string);
            e.space();
            e.token(TokenKind::IS_KW);
            e.space();
            if let Some(form) = form {
                e.token(form);
                e.space();
            }
            e.token(TokenKind::NORMALIZED_KW);
            e.token(TokenKind::R_PAREN);
        }
        // Stored as timezone(zone, timestamp), written `timestamp AT TIME ZONE zone`.
        ("timezone", [zone, timestamp]) => {
            e.token(TokenKind::L_PAREN);
            emit_parenthesized_unless_simple(e, timestamp);
            e.space();
            e.token(TokenKind::AT_KW);
            e.space();
            e.token(TokenKind::TIME_KW);
            e.space();
            e.token(TokenKind::ZONE_KW);
            e.space();
            emit_parenthesized_unless_simple(e, zone);
            e.token(TokenKind::R_PAREN);
        }
        ("timezone", [timestamp]) => {
            e.token(TokenKind::L_PAREN);
            emit_parenthesized_unless_simple(e, timestamp);
            e.space();
            e.token(TokenKind::AT_KW);
            e.space();
            e.token(TokenKind::LOCAL_KW);
            e.token(TokenKind::R_PAREN);
        }
        ("overlaps", [_, _, _, _]) => {
            e.token(TokenKind::L_PAREN);
            emit_parenthesized_list(e, &args[..2], super::emit_node);
            e.space();
            e.token(TokenKind::OVERLAPS_KW);
            e.space();
            emit_parenthesized_list(e, &args[2..], super::emit_node);
            e.token(TokenKind::R_PAREN);
        }
        ("pg_collation_for", [expr]) => {
            e.token(TokenKind::COLLATION_KW);
            e.space();
            e.token(TokenKind::FOR_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            super::emit_node(expr, e);
            e.token(TokenKind::R_PAREN);
        }
        // The XML document is a `c_expr`.
        ("xmlexists", [xpath, document]) => {
            e.token(TokenKind::XMLEXISTS_KW);
            e.token(TokenKind::L_PAREN);
            super::emit_node(xpath, e);
            e.space();
            e.token(TokenKind::PASSING_KW);
            e.space();
            emit_parenthesized_unless_simple(e, document);
            e.token(TokenKind::R_PAREN);
        }
        ("system_user", []) => e.token(TokenKind::SYSTEM_USER_KW),
        _ => return false,
    }

    true
}

/// The optional normalization form argument of `NORMALIZE` and `IS NORMALIZED`: `Some(None)` when
/// there is none, `None` when the arguments are not a form the syntax can spell.
fn optional_normalization_form(args: &[Node]) -> Option<Option<TokenKind>> {
    match args {
        [] => Some(None),
        [form] => normalization_form(form).map(Some),
        _ => None,
    }
}

/// The keyword of a Unicode normalization form, stored as the string `NFC`, `NFD`, `NFKC` or `NFKD`.
fn normalization_form(node: &Node) -> Option<TokenKind> {
    let Some(NodeEnum::AConst(a_const)) = &node.node else {
        return None;
    };
    let Some(Val::Sval(form)) = &a_const.val else {
        return None;
    };
    match form.sval.as_str() {
        "NFC" => Some(TokenKind::NFC_KW),
        "NFD" => Some(TokenKind::NFD_KW),
        "NFKC" => Some(TokenKind::NFKC_KW),
        "NFKD" => Some(TokenKind::NFKD_KW),
        _ => None,
    }
}

/// Emits the field of `EXTRACT`, stored as a string, as the word it was written as when it is one.
fn emit_extract_field(e: &mut EventEmitter, node: &Node) {
    if let Some(NodeEnum::AConst(a_const)) = &node.node
        && let Some(Val::Sval(field)) = &a_const.val
        && !needs_quoting(&field.sval)
    {
        super::emit_keyword(e, &field.sval);
        return;
    }

    super::emit_node(node, e);
}

/// Emits an operand of SQL-standard syntax, in parentheses unless it is a single term that binds
/// tighter than any operator.
fn emit_parenthesized_unless_simple(e: &mut EventEmitter, node: &Node) {
    if super::type_cast::needs_parentheses(node) {
        e.token(TokenKind::L_PAREN);
        super::emit_node(node, e);
        e.token(TokenKind::R_PAREN);
    } else {
        super::emit_node(node, e);
    }
}
