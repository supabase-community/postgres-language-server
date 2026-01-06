use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
    nodes::node_list::emit_comma_separated_list,
};
use pgls_query::protobuf::FuncCall;

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

pub(super) fn emit_func_call(e: &mut EventEmitter, n: &FuncCall) {
    e.group_start(GroupKind::FuncCall);

    // Determine if this is a special function that needs uppercase treatment
    // For normalize, we need to check if it matches the built-in pattern
    let is_builtin_normalize_call = get_last_func_name(n)
        .map(|name| name.to_lowercase() == "normalize" && is_builtin_normalize(n))
        .unwrap_or(false);

    // Emit function name (could be qualified like schema.func)
    let mut name_parts = Vec::new();

    for (i, node) in n.funcname.iter().enumerate() {
        if let Some(pgls_query::NodeEnum::String(s)) = &node.node {
            // Skip pg_catalog schema for built-in functions
            if i == 0 && s.sval.to_lowercase() == "pg_catalog" {
                continue;
            }

            // Normalize common function names to uppercase
            // For normalize, only uppercase if it matches built-in pattern
            let name = match s.sval.to_lowercase().as_str() {
                "now" => "NOW",
                "current_timestamp" => "CURRENT_TIMESTAMP",
                "current_date" => "CURRENT_DATE",
                "current_time" => "CURRENT_TIME",
                "localtime" => "LOCALTIME",
                "localtimestamp" => "LOCALTIMESTAMP",
                // Window functions
                "row_number" => "ROW_NUMBER",
                "rank" => "RANK",
                "dense_rank" => "DENSE_RANK",
                "percent_rank" => "PERCENT_RANK",
                "cume_dist" => "CUME_DIST",
                "ntile" => "NTILE",
                "lag" => "LAG",
                "lead" => "LEAD",
                "first_value" => "FIRST_VALUE",
                "last_value" => "LAST_VALUE",
                "nth_value" => "NTH_VALUE",
                // Aggregate functions
                "sum" => "SUM",
                "count" => "COUNT",
                "avg" => "AVG",
                "min" => "MIN",
                "max" => "MAX",
                // Special SQL functions
                "extract" => "EXTRACT",
                "overlay" => "OVERLAY",
                "position" => "POSITION",
                "substring" => "SUBSTRING",
                "trim" => "TRIM",
                "normalize" if is_builtin_normalize_call => "NORMALIZE",
                // SQL value functions without parens
                "system_user" => "SYSTEM_USER",
                "session_user" => "SESSION_USER",
                "current_user" => "CURRENT_USER",
                "current_role" => "CURRENT_ROLE",
                "current_catalog" => "CURRENT_CATALOG",
                "current_schema" => "CURRENT_SCHEMA",
                "user" => "USER",
                _ => &s.sval,
            };
            name_parts.push(name.to_string());
        }
    }

    // Check if this is a user-defined function that shadows a built-in
    // These need to be quoted to avoid being interpreted as the built-in
    let needs_explicit_quoting = !is_builtin_normalize_call
        && get_last_func_name(n)
            .map(|name| name.to_lowercase() == "normalize")
            .unwrap_or(false);

    // Emit function name with dots
    for (i, part) in name_parts.iter().enumerate() {
        if i > 0 {
            e.token(TokenKind::DOT);
        }
        // Quote function names that shadow built-ins
        if needs_explicit_quoting && i == name_parts.len() - 1 {
            super::emit_identifier(e, part);
        } else {
            // Emit function name directly without quoting check
            // Function names are case-insensitive and we've already normalized
            // to uppercase for built-ins, so just emit as-is
            e.token(TokenKind::IDENT(part.clone()));
        }
    }

    let function_name = name_parts.last().map(|s| s.as_str()).unwrap_or("");

    // Handle special SQL standard function syntax
    match function_name.to_lowercase().as_str() {
        "extract" => {
            emit_extract_function(e, n);
        }
        "overlay" => {
            emit_overlay_function(e, n);
        }
        "position" => {
            emit_position_function(e, n);
        }
        "substring" => {
            emit_substring_function(e, n);
        }
        "trim" => {
            emit_trim_function(e, n);
        }
        "normalize" => {
            // NORMALIZE has special syntax only when second arg is a valid form
            // If it's a user-defined function with same name, use standard call
            if is_builtin_normalize(n) {
                emit_normalize_function(e, n);
            } else {
                // This is a user-defined function, use standard call
                emit_standard_function(e, n);
            }
        }
        "xmlexists" => {
            emit_xmlexists_function(e, n);
        }
        // SQL value functions that don't use parentheses
        "system_user" | "session_user" | "current_user" | "current_role" | "current_catalog"
        | "current_schema" | "user" => {
            // These SQL standard functions are called without parentheses
            // Already emitted the name above, no parens needed
        }
        _ => {
            // Standard function call with comma-separated arguments
            emit_standard_function(e, n);
        }
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
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::OVER_KW);
        e.space();
        super::emit_window_def(e, over);
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

// EXTRACT(field FROM source)
fn emit_extract_function(e: &mut EventEmitter, n: &FuncCall) {
    assert!(
        n.args.len() == 2,
        "EXTRACT function expects 2 arguments, got {}",
        n.args.len()
    );

    e.token(TokenKind::L_PAREN);

    // First arg is the field (epoch, year, month, etc.)
    super::emit_node(&n.args[0], e);

    e.space();
    e.token(TokenKind::FROM_KW);
    e.space();
    // Second arg is the source expression
    super::emit_node(&n.args[1], e);

    e.token(TokenKind::R_PAREN);
}

// OVERLAY(string PLACING newstring FROM start [FOR length])
fn emit_overlay_function(e: &mut EventEmitter, n: &FuncCall) {
    assert!(
        n.args.len() == 3 || n.args.len() == 4,
        "OVERLAY function expects 3 or 4 arguments, got {}",
        n.args.len()
    );

    e.token(TokenKind::L_PAREN);

    // First arg: string
    super::emit_node(&n.args[0], e);

    e.space();
    e.token(TokenKind::PLACING_KW);
    e.space();
    // Second arg: newstring
    super::emit_node(&n.args[1], e);

    e.space();
    e.token(TokenKind::FROM_KW);
    e.space();
    // Third arg: start position
    super::emit_node(&n.args[2], e);

    if n.args.len() == 4 {
        e.space();
        e.token(TokenKind::FOR_KW);
        e.space();
        // Fourth arg: length
        super::emit_node(&n.args[3], e);
    }

    e.token(TokenKind::R_PAREN);
}

// POSITION(substring IN string)
// PostgreSQL stores args as: [string, substring] (reversed from syntax)
fn emit_position_function(e: &mut EventEmitter, n: &FuncCall) {
    assert!(
        n.args.len() == 2,
        "POSITION function expects 2 arguments, got {}",
        n.args.len()
    );

    e.token(TokenKind::L_PAREN);

    // First syntactic arg: substring (stored as second arg in AST)
    super::emit_node(&n.args[1], e);

    e.space();
    e.token(TokenKind::IN_KW);
    e.space();
    // Second syntactic arg: string (stored as first arg in AST)
    super::emit_node(&n.args[0], e);

    e.token(TokenKind::R_PAREN);
}

// SUBSTRING(string FROM start [FOR length])
fn emit_substring_function(e: &mut EventEmitter, n: &FuncCall) {
    assert!(
        n.args.len() == 2 || n.args.len() == 3,
        "SUBSTRING function expects 2 or 3 arguments, got {}",
        n.args.len()
    );

    e.token(TokenKind::L_PAREN);

    // First arg: string
    super::emit_node(&n.args[0], e);

    e.line(LineType::SoftOrSpace);
    e.token(TokenKind::FROM_KW);
    e.space();
    // Second arg: start position
    super::emit_node(&n.args[1], e);

    if n.args.len() == 3 {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::FOR_KW);
        e.space();
        // Third arg: length
        super::emit_node(&n.args[2], e);
    }

    e.token(TokenKind::R_PAREN);
}

// TRIM([LEADING|TRAILING|BOTH [chars] FROM] string)
fn emit_trim_function(e: &mut EventEmitter, n: &FuncCall) {
    assert!(
        !n.args.is_empty() && n.args.len() <= 3,
        "TRIM function expects 1-3 arguments, got {}",
        n.args.len()
    );

    e.token(TokenKind::L_PAREN);

    if n.args.len() == 1 {
        // Simple TRIM(string)
        super::emit_node(&n.args[0], e);
    } else if n.args.len() == 2 {
        // TRIM(chars FROM string) or TRIM(LEADING/TRAILING/BOTH string)
        // Second arg is the string, first arg is chars or mode
        super::emit_node(&n.args[0], e);
        e.space();
        e.token(TokenKind::FROM_KW);
        e.space();
        super::emit_node(&n.args[1], e);
    } else {
        // n.args.len() == 3
        // TRIM(LEADING/TRAILING/BOTH chars FROM string)
        // First arg: mode (LEADING/TRAILING/BOTH)
        super::emit_node(&n.args[0], e);
        e.space();
        // Second arg: chars
        super::emit_node(&n.args[1], e);
        e.space();
        e.token(TokenKind::FROM_KW);
        e.space();
        // Third arg: string
        super::emit_node(&n.args[2], e);
    }

    e.token(TokenKind::R_PAREN);
}

/// Check if this is a call to the built-in NORMALIZE function
/// (as opposed to a user-defined function with the same name)
fn is_builtin_normalize(n: &FuncCall) -> bool {
    // Built-in NORMALIZE takes 1 or 2 args
    if n.args.is_empty() || n.args.len() > 2 {
        return false;
    }

    // With 2 args, second must be a known normalization form
    if n.args.len() == 2 {
        if let Some(pgls_query::NodeEnum::AConst(a_const)) = &n.args[1].node {
            if let Some(pgls_query::protobuf::a_const::Val::Sval(s)) = &a_const.val {
                return matches!(s.sval.as_str(), "NFC" | "NFD" | "NFKC" | "NFKD");
            }
        }
        return false;
    }

    // Single argument is fine for built-in
    true
}

// NORMALIZE(string [, form])
// The form argument (NFC/NFD/NFKC/NFKD) is an identifier, not a string
fn emit_normalize_function(e: &mut EventEmitter, n: &FuncCall) {
    assert!(
        !n.args.is_empty() && n.args.len() <= 2,
        "NORMALIZE function expects 1 or 2 arguments, got {}",
        n.args.len()
    );

    e.token(TokenKind::L_PAREN);

    // First arg: string to normalize
    super::emit_node(&n.args[0], e);

    if n.args.len() == 2 {
        e.token(TokenKind::COMMA);
        e.space();
        // Second arg: normalization form (NFC/NFD/NFKC/NFKD)
        // This should be emitted as an identifier, not a string literal
        // The form is stored as an AConst node with a string value
        let a_const = assert_node_variant!(AConst, &n.args[1]);
        if let Some(pgls_query::protobuf::a_const::Val::Sval(s)) = &a_const.val {
            // Only emit as identifier if it's a known normalization form
            match s.sval.as_str() {
                "NFC" | "NFD" | "NFKC" | "NFKD" => {
                    e.token(TokenKind::IDENT(s.sval.clone()));
                }
                _ => {
                    // Not a known form, emit as string literal
                    super::emit_node(&n.args[1], e);
                }
            }
        } else {
            super::emit_node(&n.args[1], e);
        }
    }

    e.token(TokenKind::R_PAREN);
}

// XMLEXISTS(xpath PASSING [BY REF|VALUE] xml)
fn emit_xmlexists_function(e: &mut EventEmitter, n: &FuncCall) {
    assert!(
        n.args.len() == 2,
        "XMLEXISTS function expects 2 arguments, got {}",
        n.args.len()
    );

    e.token(TokenKind::L_PAREN);

    // First arg: xpath expression
    super::emit_node(&n.args[0], e);

    e.space();
    e.token(TokenKind::PASSING_KW);
    e.space();

    // Second arg: xml value
    super::emit_node(&n.args[1], e);

    e.token(TokenKind::R_PAREN);
}
