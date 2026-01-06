use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
    nodes::node_list::emit_comma_separated_list,
};
use pgls_query::protobuf::{self, TypeName};

use super::string::emit_identifier_maybe_quoted;

const INTERVAL_MASK_MONTH: i32 = 1 << 1;
const INTERVAL_MASK_YEAR: i32 = 1 << 2;
const INTERVAL_MASK_DAY: i32 = 1 << 3;
const INTERVAL_MASK_HOUR: i32 = 1 << 10;
const INTERVAL_MASK_MINUTE: i32 = 1 << 11;
const INTERVAL_MASK_SECOND: i32 = 1 << 12;
const INTERVAL_FULL_RANGE: i32 = 0x7FFF;
const INTERVAL_FULL_PRECISION: i32 = 0xFFFF;

pub(super) fn emit_type_name(e: &mut EventEmitter, n: &TypeName) {
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
        emit_dot_separated_name(e, name_parts);
    } else {
        e.token(TokenKind::IDENT("<?>".to_string()));
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

fn builtin_type_keywords(name_parts: &[String]) -> Option<&'static [&'static str]> {
    if name_parts.is_empty() {
        return None;
    }

    let base_name = match name_parts {
        [name] => name,
        [schema, name] if is_pg_catalog(schema) => name,
        _ => return None,
    };

    let lowered = base_name.to_ascii_lowercase();

    match lowered.as_str() {
        "bool" => Some(&["boolean"]),
        "bytea" => Some(&["bytea"]),
        "char" | "bpchar" => Some(&["char"]),
        "name" => Some(&["name"]),
        "int2" => Some(&["smallint"]),
        "int4" => Some(&["int"]),
        "int8" => Some(&["bigint"]),
        "oid" => Some(&["oid"]),
        "tid" => Some(&["tid"]),
        "xid" => Some(&["xid"]),
        "cid" => Some(&["cid"]),
        "float4" => Some(&["real"]),
        "float8" => Some(&["double", "precision"]),
        "numeric" => Some(&["numeric"]),
        "decimal" => Some(&["decimal"]),
        "money" => Some(&["money"]),
        "varchar" => Some(&["varchar"]),
        "text" => Some(&["text"]),
        "json" => Some(&["json"]),
        "jsonb" => Some(&["jsonb"]),
        "uuid" => Some(&["uuid"]),
        "xml" => Some(&["xml"]),
        "date" => Some(&["date"]),
        "time" => Some(&["time"]),
        "timetz" => Some(&["time", "with", "time", "zone"]),
        "timestamp" => Some(&["timestamp"]),
        "timestamptz" => Some(&["timestamp", "with", "time", "zone"]),
        "interval" => Some(&["interval"]),
        "bit" => Some(&["bit"]),
        "varbit" => Some(&["bit", "varying"]),
        "inet" => Some(&["inet"]),
        "cidr" => Some(&["cidr"]),
        "macaddr" => Some(&["macaddr"]),
        "macaddr8" => Some(&["macaddr8"]),
        "regclass" => Some(&["regclass"]),
        "regproc" => Some(&["regproc"]),
        "regprocedure" => Some(&["regprocedure"]),
        "regoper" => Some(&["regoper"]),
        "regoperator" => Some(&["regoperator"]),
        "regtype" => Some(&["regtype"]),
        "regconfig" => Some(&["regconfig"]),
        "regdictionary" => Some(&["regdictionary"]),
        "anyarray" => Some(&["anyarray"]),
        "anyelement" => Some(&["anyelement"]),
        "anynonarray" => Some(&["anynonarray"]),
        "anyenum" => Some(&["anyenum"]),
        "anyrange" => Some(&["anyrange"]),
        "pg_lsn" => Some(&["pg_lsn"]),
        "tsvector" => Some(&["tsvector"]),
        "tsquery" => Some(&["tsquery"]),
        "gtsvector" => Some(&["gtsvector"]),
        "txid_snapshot" => Some(&["txid_snapshot"]),
        "int4range" => Some(&["int4range"]),
        "int8range" => Some(&["int8range"]),
        "numrange" => Some(&["numrange"]),
        "tsrange" => Some(&["tsrange"]),
        "tstzrange" => Some(&["tstzrange"]),
        "daterange" => Some(&["daterange"]),
        "record" => Some(&["record"]),
        "void" => Some(&["void"]),
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
        e.token(TokenKind::L_PAREN);
        emit_comma_separated_list(e, &n.typmods, |node, emitter| {
            super::emit_node(node, emitter)
        });
        e.token(TokenKind::R_PAREN);
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
    name_parts
        .last()
        .map(|name| name.eq_ignore_ascii_case("interval"))
        .unwrap_or(false)
}

fn is_time_with_tz_type(name_parts: &[String]) -> bool {
    let base_name = match name_parts {
        [name] => name,
        [schema, name] if is_pg_catalog(schema) => name,
        _ => return false,
    };

    let lowered = base_name.to_ascii_lowercase();
    matches!(lowered.as_str(), "timetz" | "timestamptz")
}

fn emit_time_with_tz_type(e: &mut EventEmitter, n: &TypeName, name_parts: &[String]) {
    let base_name = match name_parts {
        [name] => name,
        [schema, name] if is_pg_catalog(schema) => name,
        _ => {
            emit_normalized_type_name(e, name_parts);
            return;
        }
    };

    let lowered = base_name.to_ascii_lowercase();
    match lowered.as_str() {
        "timetz" => {
            e.token(TokenKind::TIME_KW);
            // Emit precision if any
            if !n.typmods.is_empty() {
                e.token(TokenKind::L_PAREN);
                emit_comma_separated_list(e, &n.typmods, |node, emitter| {
                    super::emit_node(node, emitter)
                });
                e.token(TokenKind::R_PAREN);
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
                e.token(TokenKind::L_PAREN);
                emit_comma_separated_list(e, &n.typmods, |node, emitter| {
                    super::emit_node(node, emitter)
                });
                e.token(TokenKind::R_PAREN);
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

    if let Some(precision) = precision_value {
        if precision != INTERVAL_FULL_PRECISION {
            e.token(TokenKind::L_PAREN);
            e.token(TokenKind::IDENT(precision.to_string()));
            e.token(TokenKind::R_PAREN);
        }
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
