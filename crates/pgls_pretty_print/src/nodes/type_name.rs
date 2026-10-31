use pgls_query::protobuf;
use pgls_query::protobuf::TypeName;

use super::prelude::*;

const INTERVAL_MASK_MONTH: i32 = 1 << 1;
const INTERVAL_MASK_YEAR: i32 = 1 << 2;
const INTERVAL_MASK_DAY: i32 = 1 << 3;
const INTERVAL_MASK_HOUR: i32 = 1 << 10;
const INTERVAL_MASK_MINUTE: i32 = 1 << 11;
const INTERVAL_MASK_SECOND: i32 = 1 << 12;
const INTERVAL_FULL_RANGE: i32 = 0x7FFF;
const INTERVAL_FULL_PRECISION: i32 = 0xFFFF;

pub(super) fn emit_type_name(e: &mut EventEmitter, n: &TypeName) {
    // Column definitions and casts call this helper directly rather than emit_node, so this is the
    // only place that can emit the comments written next to a type.
    emit_with_comments_at(e, n.location, |e| emit_type_name_tokens(e, n));
}

fn emit_type_name_tokens(e: &mut EventEmitter, n: &TypeName) {
    e.group_start(GroupKind::TypeName);

    if n.setof {
        e.token(TokenKind::SETOF_KW);
        e.space();
    }

    let name_parts = collect_name_parts(n);

    if n.pct_type {
        emit_pct_type(e, &name_parts);
    } else if is_time_with_tz_type(&name_parts) {
        // Special handling for TIME/TIMESTAMP WITH TIME ZONE
        // Precision goes after TIME/TIMESTAMP, before WITH TIME ZONE
        emit_time_with_tz_type(e, n, &name_parts);
    } else if let Some(base) = verbatim_char_type(n, &name_parts) {
        emit_verbatim_char_type(e, &name_parts, base);
    } else {
        emit_normalized_type_name(e, &name_parts);
    }

    emit_type_modifiers(e, n, &name_parts);
    emit_array_bounds(e, n);

    e.group_end();
}

fn collect_name_parts(n: &TypeName) -> Vec<String> {
    n.names
        .iter()
        .filter_map(|node| match &node.node {
            Some(pgls_query::NodeEnum::String(s)) => Some(s.sval.clone()),
            _ => None,
        })
        .collect()
}

fn emit_pct_type(e: &mut EventEmitter, name_parts: &[String]) {
    if name_parts.is_empty() {
        // Fallback for unexpected AST shape; emit bare %TYPE
        e.token(TokenKind::IDENT("%".to_string()));
        e.token(TokenKind::TYPE_KW);
        return;
    }

    emit_dot_separated_name(e, name_parts);
    e.token(TokenKind::IDENT("%".to_string()));
    e.token(TokenKind::TYPE_KW);
}

fn emit_normalized_type_name(e: &mut EventEmitter, name_parts: &[String]) {
    if let Some(words) = builtin_type_keywords(name_parts) {
        emit_keyword_sequence(e, words);
    } else if !name_parts.is_empty() {
        emit_dot_separated_type_name(e, name_parts);
    } else {
        e.token(TokenKind::IDENT("<?>".to_string()));
    }
}

/// The internal one-byte type `"char"` and an unbounded `bpchar` share their spelling with the
/// `char` keyword, but `char` means `bpchar(1)`. Emitting the keyword for either type changes what
/// the statement does, so both are written as they appear in the catalog.
fn verbatim_char_type(n: &TypeName, name_parts: &[String]) -> Option<&'static str> {
    let base_name = match name_parts {
        [name] => name,
        [schema, name] if is_pg_catalog(schema) => name,
        _ => return None,
    };

    match base_name.as_str() {
        "char" => Some("char"),
        "bpchar" if n.typmods.is_empty() => Some("bpchar"),
        _ => None,
    }
}

fn emit_verbatim_char_type(e: &mut EventEmitter, name_parts: &[String], base: &str) {
    if let [schema, _] = name_parts {
        emit_identifier_maybe_quoted(e, schema);
        e.token(TokenKind::DOT);
    }

    if base == "char" {
        emit_identifier(e, base);
    } else {
        e.token(TokenKind::TYPE_IDENT(base.to_string()));
    }
}

fn emit_keyword_sequence(e: &mut EventEmitter, words: &[&'static str]) {
    for (index, word) in words.iter().enumerate() {
        if index > 0 {
            e.space();
        }
        e.token(TokenKind::TYPE_IDENT((*word).to_string()));
    }
}

fn emit_dot_separated_name(e: &mut EventEmitter, name_parts: &[String]) {
    for (index, part) in name_parts.iter().enumerate() {
        if index > 0 {
            e.token(TokenKind::DOT);
        }
        emit_identifier_maybe_quoted(e, part);
    }
}

fn emit_dot_separated_type_name(e: &mut EventEmitter, name_parts: &[String]) {
    if let [name] = name_parts
        && needs_quoting_as_function_or_type_name(name)
    {
        // Unquoted, `"numeric"` would be the keyword, which means `pg_catalog.numeric`.
        emit_identifier(e, name);
        return;
    }

    for (index, part) in name_parts.iter().enumerate() {
        if index > 0 {
            e.token(TokenKind::DOT);
        }
        emit_type_identifier_maybe_quoted(e, part);
    }
}

/// The keywords spelling a type the grammar qualifies with `pg_catalog` itself, `int` for
/// `pg_catalog.int4` for instance.
///
/// Only these collapse to keywords. Any other name is written as it is stored: `int4` stays `int4`
/// and an explicit `pg_catalog.text` keeps its schema, since dropping it lets the search path pick
/// another type.
fn builtin_type_keywords(name_parts: &[String]) -> Option<&'static [&'static str]> {
    let [schema, name] = name_parts else {
        return None;
    };
    if !is_pg_catalog(schema) {
        return None;
    }

    match name.as_str() {
        "bool" => Some(&["boolean"]),
        "bpchar" => Some(&["char"]),
        "int2" => Some(&["smallint"]),
        "int4" => Some(&["int"]),
        "int8" => Some(&["bigint"]),
        "float4" => Some(&["real"]),
        "float8" => Some(&["double", "precision"]),
        "numeric" => Some(&["numeric"]),
        "varchar" => Some(&["varchar"]),
        "json" => Some(&["json"]),
        "time" => Some(&["time"]),
        "timetz" => Some(&["time", "with", "time", "zone"]),
        "timestamp" => Some(&["timestamp"]),
        "timestamptz" => Some(&["timestamp", "with", "time", "zone"]),
        "interval" => Some(&["interval"]),
        "bit" => Some(&["bit"]),
        "varbit" => Some(&["bit", "varying"]),
        _ => None,
    }
}

fn is_pg_catalog(value: &str) -> bool {
    value.eq_ignore_ascii_case("pg_catalog")
}

fn emit_type_modifiers(e: &mut EventEmitter, n: &TypeName, name_parts: &[String]) {
    if is_interval_type(name_parts) && emit_interval_type_modifiers(e, n) {
        return;
    }

    // For TIME/TIMESTAMP WITH TIME ZONE types, modifiers are already emitted inline
    // in emit_time_with_tz_type
    if is_time_with_tz_type(name_parts) {
        return;
    }

    if !n.typmods.is_empty() {
        emit_parenthesized_list(e, &n.typmods, emit_node);
    }
}

fn emit_array_bounds(e: &mut EventEmitter, n: &TypeName) {
    for bound in &n.array_bounds {
        if let Some(pgls_query::NodeEnum::Integer(int_bound)) = &bound.node {
            e.token(TokenKind::L_BRACK);
            if int_bound.ival != -1 {
                e.token(TokenKind::IDENT(int_bound.ival.to_string()));
            }
            e.token(TokenKind::R_BRACK);
        }
    }
}

fn is_interval_type(name_parts: &[String]) -> bool {
    matches!(name_parts, [schema, name] if is_pg_catalog(schema) && name == "interval")
}

fn is_time_with_tz_type(name_parts: &[String]) -> bool {
    matches!(
        name_parts,
        [schema, name] if is_pg_catalog(schema) && matches!(name.as_str(), "timetz" | "timestamptz")
    )
}

fn emit_time_with_tz_type(e: &mut EventEmitter, n: &TypeName, name_parts: &[String]) {
    let [_, name] = name_parts else {
        emit_normalized_type_name(e, name_parts);
        return;
    };

    match name.as_str() {
        "timetz" => {
            e.token(TokenKind::TIME_KW);
            // Emit precision if any
            if !n.typmods.is_empty() {
                emit_parenthesized_list(e, &n.typmods, emit_node);
            }
            e.space();
            e.token(TokenKind::WITH_KW);
            e.space();
            e.token(TokenKind::TIME_KW);
            e.space();
            e.token(TokenKind::ZONE_KW);
        }
        "timestamptz" => {
            e.token(TokenKind::TIMESTAMP_KW);
            // Emit precision if any
            if !n.typmods.is_empty() {
                emit_parenthesized_list(e, &n.typmods, emit_node);
            }
            e.space();
            e.token(TokenKind::WITH_KW);
            e.space();
            e.token(TokenKind::TIME_KW);
            e.space();
            e.token(TokenKind::ZONE_KW);
        }
        _ => {
            emit_normalized_type_name(e, name_parts);
        }
    }
}

fn emit_interval_type_modifiers(e: &mut EventEmitter, n: &TypeName) -> bool {
    if n.typmods.is_empty() {
        return true;
    }

    if n.typmods.len() > 2 {
        return false;
    }

    let range_value = match n.typmods.first().and_then(extract_interval_typmod_int) {
        Some(value) => value,
        None => return false,
    };

    let precision_value = match n.typmods.get(1) {
        Some(node) => Some(match extract_interval_typmod_int(node) {
            Some(value) => value,
            None => return false,
        }),
        None => None,
    };

    let field_words = match interval_field_keywords(range_value) {
        Some(words) => words,
        None => return false,
    };

    if !field_words.is_empty() {
        e.space();
        emit_keyword_sequence(e, field_words);
    }

    if let Some(precision) = precision_value
        && precision != INTERVAL_FULL_PRECISION
    {
        e.token(TokenKind::L_PAREN);
        e.token(TokenKind::IDENT(precision.to_string()));
        e.token(TokenKind::R_PAREN);
    }

    true
}

fn interval_field_keywords(range: i32) -> Option<&'static [&'static str]> {
    match range {
        INTERVAL_FULL_RANGE => Some(&[]),
        value if value == INTERVAL_MASK_YEAR => Some(&["year"]),
        value if value == INTERVAL_MASK_MONTH => Some(&["month"]),
        value if value == INTERVAL_MASK_DAY => Some(&["day"]),
        value if value == INTERVAL_MASK_HOUR => Some(&["hour"]),
        value if value == INTERVAL_MASK_MINUTE => Some(&["minute"]),
        value if value == INTERVAL_MASK_SECOND => Some(&["second"]),
        value if value == INTERVAL_MASK_YEAR | INTERVAL_MASK_MONTH => {
            Some(&["year", "to", "month"])
        }
        value if value == INTERVAL_MASK_DAY | INTERVAL_MASK_HOUR => Some(&["day", "to", "hour"]),
        value if value == INTERVAL_MASK_DAY | INTERVAL_MASK_HOUR | INTERVAL_MASK_MINUTE => {
            Some(&["day", "to", "minute"])
        }
        value
            if value
                == INTERVAL_MASK_DAY
                    | INTERVAL_MASK_HOUR
                    | INTERVAL_MASK_MINUTE
                    | INTERVAL_MASK_SECOND =>
        {
            Some(&["day", "to", "second"])
        }
        value if value == INTERVAL_MASK_HOUR | INTERVAL_MASK_MINUTE => {
            Some(&["hour", "to", "minute"])
        }
        value if value == INTERVAL_MASK_HOUR | INTERVAL_MASK_MINUTE | INTERVAL_MASK_SECOND => {
            Some(&["hour", "to", "second"])
        }
        value if value == INTERVAL_MASK_MINUTE | INTERVAL_MASK_SECOND => {
            Some(&["minute", "to", "second"])
        }
        _ => None,
    }
}

fn extract_interval_typmod_int(node: &protobuf::Node) -> Option<i32> {
    match &node.node {
        Some(pgls_query::NodeEnum::AConst(a_const)) => match &a_const.val {
            Some(pgls_query::protobuf::a_const::Val::Ival(integer)) => Some(integer.ival),
            _ => None,
        },
        Some(pgls_query::NodeEnum::Integer(integer)) => Some(integer.ival),
        _ => None,
    }
}
