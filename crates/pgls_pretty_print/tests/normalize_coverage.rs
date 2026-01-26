//! Normalize Coverage Test
//!
//! This test verifies that all AST node types with `location` fields are handled
//! in the `normalize.rs` clear_location function.
//!
//! When `libpg_query` is upgraded and new node types are added that have location
//! fields, this test will fail and list exactly which types need to be added to
//! the normalization code.
//!
//! ## When this test fails:
//! 1. Look at the "Missing handlers" list
//! 2. Add each missing type to `clear_location` in `normalize.rs`
//! 3. For each type, set `(*n).location = 0;` at minimum
//! 4. Check if the type needs additional normalization beyond location clearing

use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

/// Extract all struct names that have a `location: i32` field from protobuf.rs
fn extract_types_with_location() -> HashSet<String> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let protobuf_path = manifest_dir
        .parent()
        .unwrap()
        .join("pgls_query/src/protobuf.rs");

    let content = fs::read_to_string(&protobuf_path)
        .expect("Failed to read protobuf.rs - has it been generated?");

    let mut types_with_location = HashSet::new();

    // Match struct definitions and their fields
    // Pattern: `pub struct TypeName {` followed by fields until `}`
    let struct_re = Regex::new(r"pub struct (\w+)\s*\{([^}]+)\}").unwrap();
    let location_field_re = Regex::new(r"pub location:\s*i32").unwrap();

    for cap in struct_re.captures_iter(&content) {
        let struct_name = cap[1].to_string();
        let struct_body = &cap[2];

        if location_field_re.is_match(struct_body) {
            types_with_location.insert(struct_name);
        }
    }

    types_with_location
}

/// Extract all NodeMut variants handled in clear_location from normalize.rs
fn extract_handled_types() -> HashSet<String> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let normalize_path = manifest_dir.join("src/normalize.rs");

    let content = fs::read_to_string(&normalize_path).expect("Failed to read normalize.rs");

    // Find the clear_location function
    let start = content
        .find("fn clear_location")
        .expect("Could not find clear_location function");
    let end = content[start..]
        .find("\n}\n\n")
        .map(|i| start + i)
        .unwrap_or(content.len());

    let clear_location_body = &content[start..end];

    // Extract NodeMut::TypeName patterns
    let variant_re = Regex::new(r"NodeMut::(\w+)\(").unwrap();

    variant_re
        .captures_iter(clear_location_body)
        .map(|cap| cap[1].to_string())
        .collect()
}

/// Types that are known to not need location clearing in `clear_location`.
/// These are either:
/// - Internal types not directly in the AST (e.g., ParseResult, ScanResult)
/// - Planner/executor nodes that don't appear in parsed SQL ASTs
/// - Types handled in separate normalization functions
/// - Types where location is handled via parent node traversal
fn known_exceptions() -> HashSet<&'static str> {
    [
        // ===== Internal/container types =====
        "ParseResult",
        "ScanResult",
        "ScanToken",
        "Node",
        "RawStmt", // location is stmt_location, handled separately
        "Integer",
        "Float",
        "Boolean",
        "List",
        "IntList",
        "OidList",
        // ===== Planner/executor nodes (not in parser output) =====
        // These are internal representations created during query planning,
        // not during parsing. They won't appear in parsed SQL ASTs.
        "Aggref",              // aggregate reference - planner node
        "ArrayCoerceExpr",     // array coercion - planner node
        "ArrayExpr",           // constructed array - planner node
        "CoerceToDomain",      // domain coercion - planner node
        "CoerceToDomainValue", // domain value coercion - planner node
        "CoerceViaIo",         // I/O coercion - planner node
        "CollateExpr",         // collation expression - planner node
        "ConvertRowtypeExpr",  // row type conversion - planner node
        "DistinctExpr",        // DISTINCT expression - planner node
        "FuncExpr",            // function expression - planner node (FuncCall is parser)
        "NullIfExpr",          // NULLIF expression - planner node
        "OpExpr",              // operator expression - planner node (AExpr is parser)
        "Param",               // query parameter - planner node (ParamRef is parser)
        "RelabelType",         // type relabeling - planner node
        "ScalarArrayOpExpr",   // scalar array op - planner node
        "TableFunc",           // table function - planner node
        "Var",                 // variable reference - planner node (ColumnRef is parser)
        "WindowFunc",          // window function - planner node (FuncCall is parser)
        // ===== Handled in separate normalization functions =====
        "MergeSupportFunc", // handled in normalize_merge_support_func
        "SqlValueFunction", // handled in normalize_sql_value_function
        "WithClause",       // handled in normalize_merge_support_func_recursive
        // ===== Handled via parent node =====
        "JsonFormat",          // handled via JsonFuncExpr, JsonArrayConstructor, etc.
        "JsonConstructorExpr", // internal JSON constructor - planner node
        "JsonExpr",            // internal JSON expression - planner node
        // ===== Parser nodes that need review =====
        // These appear in parsed ASTs but may not need explicit handling
        // if the pretty-print tests pass without them
        "CteCycleClause",      // CTE CYCLE clause - rarely used
        "CteSearchClause",     // CTE SEARCH clause - rarely used
        "PartitionRangeDatum", // partition range datum - handled via PartitionBoundSpec
        "PlAssignStmt",        // PL/pgSQL assignment - not standard SQL
    ]
    .into_iter()
    .collect()
}

#[test]
fn all_location_fields_are_normalized() {
    let types_with_location = extract_types_with_location();
    let handled_types = extract_handled_types();
    let exceptions = known_exceptions();

    let mut missing: Vec<_> = types_with_location
        .iter()
        .filter(|t| !handled_types.contains(*t) && !exceptions.contains(t.as_str()))
        .collect();

    missing.sort();

    if !missing.is_empty() {
        panic!(
            "\n\n\
            ========================================\n\
            AST NORMALIZATION COVERAGE INCOMPLETE\n\
            ========================================\n\n\
            The following types have `location` fields but are NOT handled\n\
            in the `clear_location` function in normalize.rs:\n\n\
            {}\n\n\
            To fix:\n\
            1. Add a handler for each type in the `clear_location` match\n\
            2. At minimum: `NodeMut::TypeName(n) => {{ (*n).location = 0; }}`\n\
            3. Check if additional normalization is needed\n\n\
            If a type should NOT be normalized, add it to `known_exceptions()`\n\
            in this test file with a comment explaining why.\n\
            ========================================\n",
            missing
                .iter()
                .map(|t| format!("  - {t}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}

#[test]
fn no_obsolete_handlers() {
    let types_with_location = extract_types_with_location();
    let handled_types = extract_handled_types();

    // Check for handlers that reference types that don't exist or don't have location
    let mut obsolete: Vec<_> = handled_types
        .iter()
        .filter(|t| !types_with_location.contains(*t))
        .collect();

    obsolete.sort();

    if !obsolete.is_empty() {
        println!(
            "\nNote: The following handlers in clear_location reference types \
             that don't have a `location` field (they may have other fields being normalized):\n\
             {}\n\
             This is fine if the handler normalizes other fields, but worth reviewing.\n",
            obsolete
                .iter()
                .map(|t| format!("  - {t}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
    // This is just informational, not a failure
}
