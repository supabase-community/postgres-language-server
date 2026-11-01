use pgls_query::protobuf::a_const::Val;
use pgls_query::protobuf::{CoercionForm, FuncCall};

use crate::CommaStyle;

use super::prelude::*;

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
    // Within a group, trailing commas fill. Leading commas break before every argument of a group
    // that doesn't fit on its line.
    let within_group = match e.config().comma_style {
        CommaStyle::Leading => ListSeparatorSpacing::SoftOrSpace,
        CommaStyle::Trailing => ListSeparatorSpacing::Fill,
    };

    for (group_index, arguments) in args.chunks(group_size).enumerate() {
        if group_index > 0 {
            emit_list_comma(e, arguments.first(), ListSeparatorSpacing::SoftOrSpace);
        }

        e.group_start();
        for (argument_index, argument) in arguments.iter().enumerate() {
            if argument_index > 0 {
                // `Fill` keeps the pair on one line when it fits and falls back to one
                // argument per line when it does not, instead of exceeding the line width.
                emit_list_comma(e, Some(argument), within_group);
            }
            emit_node(e, argument);
        }
        e.group_end();
    }
}

pub(super) fn emit_func_call(e: &mut EventEmitter, n: &FuncCall) {
    e.group_start();
    // The name and arguments have a group of their own when a clause follows them, so that
    // `count(*)` stays together and the line breaks before FILTER, WITHIN GROUP or OVER first.
    let has_suffix = n.over.is_some() || n.agg_filter.is_some() || n.agg_within_group;
    if has_suffix {
        e.group_start();
    }

    // The grammar turns SQL-standard syntax such as `EXTRACT(year FROM d)` or `ts AT TIME ZONE z`
    // into a call to a `pg_catalog` function, flagged as written in that syntax. Anything else was
    // written as a call, and is written back as one with its name exactly as qualified.
    let is_sql_syntax = n.funcformat == CoercionForm::CoerceSqlSyntax as i32;
    if !(is_sql_syntax && emit_sql_syntax_call(e, n)) {
        emit_function_name(e, &n.funcname);
        emit_standard_function(e, n);
    }

    if has_suffix {
        e.group_end();
    }

    if n.agg_within_group {
        debug_assert!(
            !n.agg_order.is_empty(),
            "ordered-set aggregate is missing ORDER BY list"
        );

        // A clause that breaks onto its own line continues the call, one level in.
        e.indent_start();
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::WITHIN_KW);
        e.space();
        e.token(TokenKind::GROUP_KW);
        e.space();
        // Wrap in group for compact formatting
        e.group_start();
        e.token(TokenKind::L_PAREN);
        e.line(LineType::Soft);
        e.indent_start();
        e.token(TokenKind::ORDER_KW);
        e.space();
        e.token(TokenKind::BY_KW);
        e.space();
        emit_comma_separated_list(e, &n.agg_order, emit_node);
        e.indent_end();
        e.line(LineType::Soft);
        e.token(TokenKind::R_PAREN);
        e.group_end();
        e.indent_end();
    }

    if let Some(ref filter) = n.agg_filter {
        e.indent_start();
        e.line(LineType::SoftOrSpace);
        emit_filter_clause(e, filter);
        e.indent_end();
    }

    // Handle OVER clause (window functions)
    if let Some(ref over) = n.over {
        e.space();
        e.token(TokenKind::OVER_KW);
        e.space();
        e.group_start();
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
            if n.func_variadic
                && let Some((variadic, leading)) = n.args.split_last()
            {
                // Emit non-variadic args first
                for (idx, arg) in leading.iter().enumerate() {
                    if idx > 0 {
                        e.token(TokenKind::COMMA);
                        e.line(LineType::SoftOrSpace);
                    }
                    emit_node(e, arg);
                }

                if !leading.is_empty() {
                    e.token(TokenKind::COMMA);
                    e.line(LineType::SoftOrSpace);
                }

                // Emit VARIADIC with the last argument
                e.token(TokenKind::VARIADIC_KW);
                e.space();
                emit_node(e, variadic);
            } else if let Some(group_size) = configured_argument_group_size(e, n) {
                emit_grouped_arguments(e, &n.args, group_size);
            } else {
                emit_comma_separated_list(e, &n.args, emit_node);
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
            emit_comma_separated_list(e, &n.agg_order, emit_node);
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
            open_call_arguments(e);
            // The arguments stay together on the indented line when they fit there.
            e.group_start();
            emit_extract_field(e, field);
            e.space();
            e.token(TokenKind::FROM_KW);
            e.line(LineType::SoftOrSpace);
            emit_node(e, source);
            e.group_end();
            close_call_arguments(e);
        }
        ("overlay", [string, replacement, start, rest @ ..]) if rest.len() <= 1 => {
            e.token(TokenKind::OVERLAY_KW);
            open_call_arguments(e);
            // The arguments stay together on the indented line when they fit there.
            e.group_start();
            emit_node(e, string);
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::PLACING_KW);
            e.space();
            emit_node(e, replacement);
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::FROM_KW);
            e.space();
            emit_node(e, start);
            if let [length] = rest {
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::FOR_KW);
                e.space();
                emit_node(e, length);
            }
            e.group_end();
            close_call_arguments(e);
        }
        // Stored as position(string, substring), written POSITION(substring IN string). Both
        // operands are `b_expr`, which an operator expression is not.
        ("position", [string, substring]) => {
            e.token(TokenKind::POSITION_KW);
            open_call_arguments(e);
            // The arguments stay together on the indented line when they fit there.
            e.group_start();
            emit_parenthesized_unless_simple(e, substring);
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::IN_KW);
            e.space();
            emit_parenthesized_unless_simple(e, string);
            e.group_end();
            close_call_arguments(e);
        }
        ("substring", [string, start, rest @ ..]) if rest.len() <= 1 => {
            e.token(TokenKind::SUBSTRING_KW);
            open_call_arguments(e);
            // The arguments stay together on the indented line when they fit there.
            e.group_start();
            emit_node(e, string);
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::FROM_KW);
            e.space();
            emit_node(e, start);
            if let [length] = rest {
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::FOR_KW);
                e.space();
                emit_node(e, length);
            }
            e.group_end();
            close_call_arguments(e);
        }
        // Stored as btrim(string[, characters]), written TRIM(BOTH characters FROM string).
        ("btrim" | "ltrim" | "rtrim", [string, rest @ ..]) if rest.len() <= 1 => {
            e.token(TokenKind::TRIM_KW);
            open_call_arguments(e);
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
                emit_node(e, characters);
                e.space();
                e.token(TokenKind::FROM_KW);
                e.space();
            }
            emit_node(e, string);
            close_call_arguments(e);
        }
        ("normalize", [string, rest @ ..]) => {
            let Some(form) = optional_normalization_form(rest) else {
                return false;
            };
            e.token(TokenKind::NORMALIZE_KW);
            open_call_arguments(e);
            emit_node(e, string);
            if let Some(form) = form {
                e.token(TokenKind::COMMA);
                e.line(LineType::SoftOrSpace);
                e.token(form);
            }
            close_call_arguments(e);
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
            emit_parenthesized_list(e, &args[..2], emit_node);
            e.space();
            e.token(TokenKind::OVERLAPS_KW);
            e.space();
            emit_parenthesized_list(e, &args[2..], emit_node);
            e.token(TokenKind::R_PAREN);
        }
        ("pg_collation_for", [expr]) => {
            e.token(TokenKind::COLLATION_KW);
            e.space();
            e.token(TokenKind::FOR_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            emit_node(e, expr);
            e.token(TokenKind::R_PAREN);
        }
        // The XML document is a `c_expr`.
        ("xmlexists", [xpath, document]) => {
            e.token(TokenKind::XMLEXISTS_KW);
            open_call_arguments(e);
            // The arguments stay together on the indented line when they fit there.
            e.group_start();
            emit_node(e, xpath);
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::PASSING_KW);
            e.space();
            emit_parenthesized_unless_simple(e, document);
            e.group_end();
            close_call_arguments(e);
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
        emit_keyword(e, &field.sval);
        return;
    }

    emit_node(e, node);
}

/// Emits an operand of SQL-standard syntax, in parentheses unless it is a single term that binds
/// tighter than any operator.
fn emit_parenthesized_unless_simple(e: &mut EventEmitter, node: &Node) {
    if needs_parentheses(node, Position::Postfix) {
        e.token(TokenKind::L_PAREN);
        emit_node(e, node);
        e.token(TokenKind::R_PAREN);
    } else {
        emit_node(e, node);
    }
}
