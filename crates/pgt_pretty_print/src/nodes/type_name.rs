use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
    nodes::node_list::emit_comma_separated_list,
};
use pgt_query::protobuf::{self, TypeName};

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
            Some(pgt_query::NodeEnum::String(s)) => Some(s.sval.clone()),
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
        e.token(TokenKind::IDENT((*word).to_string()));
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
        "bool" => Some(&["BOOLEAN"]),
        "bytea" => Some(&["BYTEA"]),
        "char" | "bpchar" => Some(&["CHAR"]),
        "name" => Some(&["NAME"]),
        "int2" => Some(&["SMALLINT"]),
        "int4" => Some(&["INT"]),
        "int8" => Some(&["BIGINT"]),
        "oid" => Some(&["OID"]),
        "tid" => Some(&["TID"]),
        "xid" => Some(&["XID"]),
        "cid" => Some(&["CID"]),
        "float4" => Some(&["REAL"]),
        "float8" => Some(&["DOUBLE", "PRECISION"]),
        "numeric" => Some(&["NUMERIC"]),
        "decimal" => Some(&["DECIMAL"]),
        "money" => Some(&["MONEY"]),
        "varchar" => Some(&["VARCHAR"]),
        "text" => Some(&["TEXT"]),
        "json" => Some(&["JSON"]),
        "jsonb" => Some(&["JSONB"]),
        "uuid" => Some(&["UUID"]),
        "xml" => Some(&["XML"]),
        "date" => Some(&["DATE"]),
        "time" => Some(&["TIME"]),
        "timetz" => Some(&["TIME", "WITH", "TIME", "ZONE"]),
        "timestamp" => Some(&["TIMESTAMP"]),
        "timestamptz" => Some(&["TIMESTAMP", "WITH", "TIME", "ZONE"]),
        "interval" => Some(&["INTERVAL"]),
        "bit" => Some(&["BIT"]),
        "varbit" => Some(&["BIT", "VARYING"]),
        "inet" => Some(&["INET"]),
        "cidr" => Some(&["CIDR"]),
        "macaddr" => Some(&["MACADDR"]),
        "macaddr8" => Some(&["MACADDR8"]),
        "regclass" => Some(&["REGCLASS"]),
        "regproc" => Some(&["REGPROC"]),
        "regprocedure" => Some(&["REGPROCEDURE"]),
        "regoper" => Some(&["REGOPER"]),
        "regoperator" => Some(&["REGOPERATOR"]),
        "regtype" => Some(&["REGTYPE"]),
        "regconfig" => Some(&["REGCONFIG"]),
        "regdictionary" => Some(&["REGDICTIONARY"]),
        "anyarray" => Some(&["ANYARRAY"]),
        "anyelement" => Some(&["ANYELEMENT"]),
        "anynonarray" => Some(&["ANYNONARRAY"]),
        "anyenum" => Some(&["ANYENUM"]),
        "anyrange" => Some(&["ANYRANGE"]),
        "pg_lsn" => Some(&["PG_LSN"]),
        "tsvector" => Some(&["TSVECTOR"]),
        "tsquery" => Some(&["TSQUERY"]),
        "gtsvector" => Some(&["GTSVECTOR"]),
        "txid_snapshot" => Some(&["TXID_SNAPSHOT"]),
        "int4range" => Some(&["INT4RANGE"]),
        "int8range" => Some(&["INT8RANGE"]),
        "numrange" => Some(&["NUMRANGE"]),
        "tsrange" => Some(&["TSRANGE"]),
        "tstzrange" => Some(&["TSTZRANGE"]),
        "daterange" => Some(&["DATERANGE"]),
        "record" => Some(&["RECORD"]),
        "void" => Some(&["VOID"]),
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
        if let Some(pgt_query::NodeEnum::Integer(int_bound)) = &bound.node {
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
        value if value == INTERVAL_MASK_YEAR => Some(&["YEAR"]),
        value if value == INTERVAL_MASK_MONTH => Some(&["MONTH"]),
        value if value == INTERVAL_MASK_DAY => Some(&["DAY"]),
        value if value == INTERVAL_MASK_HOUR => Some(&["HOUR"]),
        value if value == INTERVAL_MASK_MINUTE => Some(&["MINUTE"]),
        value if value == INTERVAL_MASK_SECOND => Some(&["SECOND"]),
        value if value == INTERVAL_MASK_YEAR | INTERVAL_MASK_MONTH => {
            Some(&["YEAR", "TO", "MONTH"])
        }
        value if value == INTERVAL_MASK_DAY | INTERVAL_MASK_HOUR => Some(&["DAY", "TO", "HOUR"]),
        value if value == INTERVAL_MASK_DAY | INTERVAL_MASK_HOUR | INTERVAL_MASK_MINUTE => {
            Some(&["DAY", "TO", "MINUTE"])
        }
        value
            if value
                == INTERVAL_MASK_DAY
                    | INTERVAL_MASK_HOUR
                    | INTERVAL_MASK_MINUTE
                    | INTERVAL_MASK_SECOND =>
        {
            Some(&["DAY", "TO", "SECOND"])
        }
        value if value == INTERVAL_MASK_HOUR | INTERVAL_MASK_MINUTE => {
            Some(&["HOUR", "TO", "MINUTE"])
        }
        value if value == INTERVAL_MASK_HOUR | INTERVAL_MASK_MINUTE | INTERVAL_MASK_SECOND => {
            Some(&["HOUR", "TO", "SECOND"])
        }
        value if value == INTERVAL_MASK_MINUTE | INTERVAL_MASK_SECOND => {
            Some(&["MINUTE", "TO", "SECOND"])
        }
        _ => None,
    }
}

fn extract_interval_typmod_int(node: &protobuf::Node) -> Option<i32> {
    match &node.node {
        Some(pgt_query::NodeEnum::AConst(a_const)) => match &a_const.val {
            Some(pgt_query::protobuf::a_const::Val::Ival(integer)) => Some(integer.ival),
            _ => None,
        },
        Some(pgt_query::NodeEnum::Integer(integer)) => Some(integer.ival),
        _ => None,
    }
}
