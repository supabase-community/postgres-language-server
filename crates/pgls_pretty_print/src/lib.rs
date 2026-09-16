mod codegen;
pub mod comments;
pub mod emitter;
pub mod nodes;
pub mod normalize;
pub mod renderer;

use std::collections::HashSet;

pub use crate::codegen::token_kind::TokenKind;
pub use crate::comments::{AttachedComments, Comment, attach_comments};
pub use crate::normalize::normalize_ast;
pub use crate::renderer::{IndentStyle, KeywordCase, RenderConfig};
use pgls_query::NodeEnum;
use thiserror::Error;

/// Error type for formatting operations.
#[derive(Debug, Error)]
pub enum FormatError {
    /// Parsing failed - the formatted output is not valid SQL
    #[error("Failed to parse formatted output: {message}")]
    ParseError { message: String },

    /// Rendering failed
    #[error("Failed to render: {message}")]
    RenderError { message: String },

    /// Beta safety check failed - formatted output has different semantics
    #[error(
        "Formatter (beta): This statement type is not fully supported yet. \
         Formatting may alter semantics. Please report: \
         https://github.com/supabase/postgres-language-server/issues\n\
         Details: {message}"
    )]
    BetaUnsupported { message: String },

    /// A comment could not be placed in the formatted output.
    #[error("Formatter: {count} comment(s) could not be placed, the statement was left as written")]
    UnplaceableComment { count: usize },

    /// Reformatting a statement with comments produced a cycle instead of a fixed layout.
    #[error(
        "Formatter: comment layout did not stabilize after {passes} passes, the statement was left as written"
    )]
    NonIdempotentCommentLayout { passes: usize },
}

/// Configuration for the SQL formatter.
#[derive(Debug, Clone)]
pub struct FormatConfig {
    /// Maximum line width before breaking. Default: 100.
    pub line_width: usize,
    /// Number of spaces (or tab width) for indentation. Default: 2.
    pub indent_size: usize,
    /// Whether to use spaces or tabs for indentation. Default: Spaces.
    pub indent_style: IndentStyle,
    /// Casing for SQL keywords (SELECT, FROM, WHERE). Default: Lower.
    pub keyword_case: KeywordCase,
    /// Casing for constants (NULL, TRUE, FALSE). Default: Lower.
    pub constant_case: KeywordCase,
    /// Casing for data types (text, varchar, int). Default: Lower.
    pub type_case: KeywordCase,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            line_width: 100,
            indent_size: 2,
            indent_style: IndentStyle::Spaces,
            keyword_case: KeywordCase::default(),
            constant_case: KeywordCase::default(),
            type_case: KeywordCase::default(),
        }
    }
}

impl From<FormatConfig> for RenderConfig {
    fn from(config: FormatConfig) -> Self {
        Self {
            max_line_length: config.line_width,
            indent_size: config.indent_size,
            indent_style: config.indent_style,
            keyword_case: config.keyword_case,
            constant_case: config.constant_case,
            type_case: config.type_case,
        }
    }
}

/// Result of formatting a SQL statement.
#[derive(Debug)]
pub struct FormatResult {
    /// The formatted SQL string.
    pub formatted: String,
}

/// Format a single SQL statement from its AST.
///
/// This function takes an already-parsed AST, formats it according to the
/// configuration, and returns the formatted output.
///
/// During beta, this function performs semantic verification by
/// comparing normalized ASTs. If the formatted output would have
/// different semantics, it returns a `BetaUnsupported` error.
///
/// # Arguments
///
/// * `ast` - The parsed AST of the SQL statement
/// * `config` - Formatting configuration options
///
/// # Returns
///
/// * `Ok(FormatResult)` - The formatted SQL
/// * `Err(FormatError)` - If formatting fails or beta safety check fails
pub fn format_statement(
    ast: &NodeEnum,
    sql: &str,
    config: &FormatConfig,
) -> Result<FormatResult, FormatError> {
    const MAX_COMMENT_FORMAT_PASSES: usize = 6;

    let attached = comments::attach_comments(sql, ast);
    let has_comments = !attached.leading_by_location.is_empty()
        || !attached.trailing_by_location.is_empty()
        || !attached.unattached.is_empty();

    if !has_comments {
        return format_statement_once(ast, config, attached);
    }

    let mut current_sql = sql.to_string();
    let mut current_ast = ast.clone();
    let mut attached = attached;
    let mut seen_layouts = HashSet::from([current_sql.clone()]);

    for pass in 1..=MAX_COMMENT_FORMAT_PASSES {
        let result = format_statement_once(&current_ast, config, attached)?;
        if result.formatted == current_sql {
            return Ok(result);
        }

        if !seen_layouts.insert(result.formatted.clone()) {
            return Err(FormatError::NonIdempotentCommentLayout { passes: pass });
        }

        current_sql = result.formatted;
        current_ast = pgls_query::parse(&current_sql)
            .map_err(|e| FormatError::ParseError {
                message: format!("Formatted SQL failed to parse: {e}"),
            })?
            .into_root()
            .ok_or_else(|| FormatError::ParseError {
                message: "No root node in parsed output (expected single statement)".to_string(),
            })?;
        attached = comments::attach_comments(&current_sql, &current_ast);
    }

    Err(FormatError::NonIdempotentCommentLayout {
        passes: MAX_COMMENT_FORMAT_PASSES,
    })
}

/// Formats a statement exactly once, including semantic verification.
fn format_statement_once(
    ast: &NodeEnum,
    config: &FormatConfig,
    attached: AttachedComments,
) -> Result<FormatResult, FormatError> {
    // A comment that no node follows cannot be placed. Refusing here preserves the original text,
    // which is safer than emitting a statement that would silently drop it.
    if !attached.unattached.is_empty() {
        return Err(FormatError::UnplaceableComment {
            count: attached.unattached.len(),
        });
    }

    let mut emitter = emitter::EventEmitter::with_comments(
        attached.leading_by_location,
        attached.trailing_by_location,
    );
    nodes::emit_node_enum(ast, &mut emitter);
    let pending = emitter.pending_comments();
    if pending > 0 {
        return Err(FormatError::UnplaceableComment { count: pending });
    }

    // Render to string
    let render_config = RenderConfig {
        max_line_length: config.line_width,
        indent_size: config.indent_size,
        indent_style: config.indent_style.clone(),
        keyword_case: config.keyword_case.clone(),
        constant_case: config.constant_case.clone(),
        type_case: config.type_case.clone(),
    };

    let mut output = String::new();
    let mut renderer = renderer::Renderer::new(&mut output, render_config);
    renderer
        .render(emitter.events)
        .map_err(|e| FormatError::RenderError {
            message: e.to_string(),
        })?;

    // BETA: Verify formatted output parses to semantically equivalent AST
    let parsed_output = pgls_query::parse(&output).map_err(|e| FormatError::ParseError {
        message: format!("Formatted SQL failed to parse: {e}"),
    })?;

    let mut output_ast = parsed_output
        .into_root()
        .ok_or_else(|| FormatError::ParseError {
            message: "No root node in parsed output (expected single statement)".to_string(),
        })?;

    let mut original_ast = ast.clone();

    // Normalize both ASTs for semantic comparison
    normalize::normalize_ast(&mut output_ast);
    normalize::normalize_ast(&mut original_ast);

    if output_ast != original_ast {
        return Err(FormatError::BetaUnsupported {
            message: "Normalized ASTs differ after formatting".to_string(),
        });
    }

    Ok(FormatResult { formatted: output })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_simple_select() {
        let sql = "SELECT id, name FROM users WHERE active = true";
        let parsed = pgls_query::parse(sql).unwrap();
        let ast = parsed.into_root().unwrap();

        let config = FormatConfig::default();
        let result = format_statement(&ast, sql, &config).unwrap();

        assert!(!result.formatted.is_empty());
        // Default keyword_case is Lower, so check for lowercase
        assert!(result.formatted.contains("select"));
    }

    #[test]
    fn test_format_config_defaults() {
        let config = FormatConfig::default();
        assert_eq!(config.line_width, 100);
        assert_eq!(config.indent_size, 2);
    }

    #[test]
    fn a_statement_with_a_comment_is_formatted() {
        let sql = "SELECT\n-- pick the magic value\n1 FROM s.t";
        let ast = pgls_query::parse(sql).unwrap().into_root().unwrap();

        let result = format_statement(&ast, sql, &FormatConfig::default()).expect("formatted");

        assert!(result.formatted.contains("-- pick the magic value"));
        assert!(result.formatted.contains("select"));
    }

    #[test]
    fn a_comment_after_the_statement_terminator_is_refused() {
        let sql = "SELECT 1 FROM s.t; -- trailing";
        let ast = pgls_query::parse(sql).unwrap().into_root().unwrap();

        let error = format_statement(&ast, sql, &FormatConfig::default())
            .expect_err("the comment belongs to the next statement");

        assert!(matches!(error, FormatError::UnplaceableComment { .. }));
    }

    #[test]
    fn formatting_a_trailing_comment_is_idempotent() {
        let sql = "SELECT * FROM t WHERE a = 1 -- keep condition context\nAND b = 2;";
        let ast = pgls_query::parse(sql).unwrap().into_root().unwrap();
        let config = FormatConfig::default();

        let first = format_statement(&ast, sql, &config)
            .expect("first pass")
            .formatted;
        let reparsed = pgls_query::parse(&first).unwrap().into_root().unwrap();
        let second = format_statement(&reparsed, &first, &config)
            .expect("second pass")
            .formatted;

        assert!(first.contains("1 -- keep condition context"));
        assert_eq!(first, second);
    }

    #[test]
    fn formatting_a_comment_before_an_order_by_clause_is_idempotent() {
        let sql = "SELECT * FROM t\n-- WHERE\n--  a is active\nORDER BY a;";
        let ast = pgls_query::parse(sql).unwrap().into_root().unwrap();
        let config = FormatConfig::default();

        let first = format_statement(&ast, sql, &config)
            .expect("first pass")
            .formatted;
        let reparsed = pgls_query::parse(&first).unwrap().into_root().unwrap();
        let second = format_statement(&reparsed, &first, &config)
            .expect("second pass")
            .formatted;

        assert!(first.contains("order by -- WHERE"));
        assert_eq!(first, second);
    }

    #[test]
    fn formatting_a_trailing_comment_after_a_right_hand_operand_is_idempotent() {
        let sql = "SELECT * FROM t WHERE type_de_variable <> '011' -- exclude VAT\n;";
        let ast = pgls_query::parse(sql).unwrap().into_root().unwrap();
        let config = FormatConfig::default();

        let first = format_statement(&ast, sql, &config)
            .expect("first pass")
            .formatted;
        let reparsed = pgls_query::parse(&first).unwrap().into_root().unwrap();
        let second = format_statement(&reparsed, &first, &config)
            .expect("second pass")
            .formatted;

        assert!(first.contains("'011' -- exclude VAT"));
        assert_eq!(first, second);
    }

    #[test]
    fn formatting_a_comment_after_a_list_separator_is_idempotent() {
        let sql = "SELECT a, -- temporarily omit b\nb FROM t;";
        let ast = pgls_query::parse(sql).unwrap().into_root().unwrap();
        let config = FormatConfig::default();

        let first = format_statement(&ast, sql, &config)
            .expect("first pass")
            .formatted;
        let reparsed = pgls_query::parse(&first).unwrap().into_root().unwrap();
        let second = format_statement(&reparsed, &first, &config)
            .expect("second pass")
            .formatted;

        assert!(first.contains("-- temporarily omit b"));
        assert_eq!(first, second);
    }

    #[test]
    fn formatting_a_comment_before_a_conjunction_is_idempotent() {
        let sql = "SELECT * FROM s.t WHERE a = 1\n-- keep b out for now\nAND b = 2;";
        let ast = pgls_query::parse(sql).unwrap().into_root().unwrap();
        let config = FormatConfig {
            indent_size: 4,
            indent_style: IndentStyle::Tabs,
            keyword_case: KeywordCase::Upper,
            ..Default::default()
        };

        let first = format_statement(&ast, sql, &config)
            .expect("first pass")
            .formatted;
        let reparsed = pgls_query::parse(&first).unwrap().into_root().unwrap();
        let second = format_statement(&reparsed, &first, &config)
            .expect("second pass")
            .formatted;

        assert!(first.contains("-- keep b out for now"));
        assert_eq!(first, second);
    }

    #[test]
    fn a_comment_after_an_update_target_relation_is_kept() {
        let sql = "UPDATE s.t AS x -- remove the strays\nSET a = 1";
        let ast = pgls_query::parse(sql).unwrap().into_root().unwrap();

        let result = format_statement(&ast, sql, &FormatConfig::default()).expect("formatted");

        assert!(result.formatted.contains("-- remove the strays"));
    }

    #[test]
    fn a_comment_after_a_delete_target_relation_is_kept() {
        let sql = "DELETE FROM s.t -- only the strays\nWHERE a = 1";
        let ast = pgls_query::parse(sql).unwrap().into_root().unwrap();

        let result = format_statement(&ast, sql, &FormatConfig::default()).expect("formatted");

        assert!(result.formatted.contains("-- only the strays"));
    }

    #[test]
    fn formatting_a_comment_after_a_target_relation_is_idempotent() {
        let sql = "UPDATE s.t AS x -- remove the strays\nSET a = 1";
        let ast = pgls_query::parse(sql).unwrap().into_root().unwrap();
        let config = FormatConfig::default();

        let first = format_statement(&ast, sql, &config)
            .expect("first pass")
            .formatted;
        let reparsed = pgls_query::parse(&first).unwrap().into_root().unwrap();
        let second = format_statement(&reparsed, &first, &config)
            .expect("second pass")
            .formatted;

        assert_eq!(first, second);
    }

    #[test]
    fn a_comment_in_an_insert_column_list_is_kept() {
        let sql = "INSERT INTO s.t\n(\n  a\n, b -- the management type\n, c\n)\nVALUES (1, 2, 3)";
        let ast = pgls_query::parse(sql).unwrap().into_root().unwrap();

        let result = format_statement(&ast, sql, &FormatConfig::default()).expect("formatted");

        assert!(result.formatted.contains("-- the management type"));
    }

    #[test]
    fn formatting_a_comment_in_an_insert_column_list_is_idempotent() {
        let sql = "INSERT INTO s.t\n(\n  a\n, b -- the management type\n, c\n)\nVALUES (1, 2, 3)";
        let ast = pgls_query::parse(sql).unwrap().into_root().unwrap();
        let config = FormatConfig::default();

        let first = format_statement(&ast, sql, &config)
            .expect("first pass")
            .formatted;
        let reparsed = pgls_query::parse(&first).unwrap().into_root().unwrap();
        let second = format_statement(&reparsed, &first, &config)
            .expect("second pass")
            .formatted;

        assert_eq!(first, second);
    }

    #[test]
    fn a_comment_after_a_column_type_is_kept() {
        let sql = "CREATE TABLE s.t (\n\ta int -- the magic column\n\t, b int\n)";
        let ast = pgls_query::parse(sql).unwrap().into_root().unwrap();

        let result = format_statement(&ast, sql, &FormatConfig::default()).expect("formatted");

        assert!(result.formatted.contains("-- the magic column"));
    }

    #[test]
    fn formatting_a_comment_after_a_column_type_is_idempotent() {
        let sql = "CREATE TABLE s.t (\n\ta int -- the magic column\n\t, b int\n)";
        let ast = pgls_query::parse(sql).unwrap().into_root().unwrap();
        let config = FormatConfig::default();

        let first = format_statement(&ast, sql, &config)
            .expect("first pass")
            .formatted;
        let reparsed = pgls_query::parse(&first).unwrap().into_root().unwrap();
        let second = format_statement(&reparsed, &first, &config)
            .expect("second pass")
            .formatted;

        assert_eq!(first, second);
    }

    #[test]
    fn a_comment_closing_a_statement_is_kept() {
        let sql = "SELECT 1 FROM s.t -- trailing";
        let ast = pgls_query::parse(sql).unwrap().into_root().unwrap();

        let result = format_statement(&ast, sql, &FormatConfig::default()).expect("formatted");

        assert!(result.formatted.contains("-- trailing"));
    }

    #[test]
    fn a_comment_closing_a_column_list_is_kept() {
        let sql = "CREATE TABLE s.t (\n\tid uuid,\n\tkind text\n--\t\"createdAt\" timestamp\n)";
        let ast = pgls_query::parse(sql).unwrap().into_root().unwrap();

        let result = format_statement(&ast, sql, &FormatConfig::default()).expect("formatted");

        assert!(result.formatted.contains("\"createdAt\" timestamp"));
    }

    #[test]
    fn formatting_a_comment_closing_a_values_list_is_idempotent() {
        let sql = "INSERT INTO s.t VALUES\n  ('a', 'b')\n, ('c', 'd') -- the last one";
        let ast = pgls_query::parse(sql).unwrap().into_root().unwrap();
        let config = FormatConfig::default();

        let first = format_statement(&ast, sql, &config)
            .expect("first pass")
            .formatted;
        let reparsed = pgls_query::parse(&first).unwrap().into_root().unwrap();
        let second = format_statement(&reparsed, &first, &config)
            .expect("second pass")
            .formatted;

        assert!(first.contains("-- the last one"));
        assert_eq!(first, second);
    }

    #[test]
    fn a_comment_in_an_update_set_list_is_kept() {
        let sql =
            "UPDATE s.t SET\n-- the reason is deducted from the WHERE below\na = 1 WHERE b = 2";
        let ast = pgls_query::parse(sql).unwrap().into_root().unwrap();

        let result = format_statement(&ast, sql, &FormatConfig::default()).expect("formatted");

        assert!(
            result
                .formatted
                .contains("-- the reason is deducted from the WHERE below")
        );
    }

    #[test]
    fn formatting_a_comment_in_an_update_set_list_is_idempotent() {
        let sql =
            "UPDATE s.t SET\n-- the reason is deducted from the WHERE below\na = 1 WHERE b = 2";
        let ast = pgls_query::parse(sql).unwrap().into_root().unwrap();
        let config = FormatConfig::default();

        let first = format_statement(&ast, sql, &config)
            .expect("first pass")
            .formatted;
        let reparsed = pgls_query::parse(&first).unwrap().into_root().unwrap();
        let second = format_statement(&reparsed, &first, &config)
            .expect("second pass")
            .formatted;

        assert_eq!(first, second);
    }

    #[test]
    fn a_comment_before_a_window_definition_is_kept() {
        let sql = "SELECT bool_or(a <> b) -- has_decimal\nOVER (PARTITION BY c) FROM s.t";
        let ast = pgls_query::parse(sql).unwrap().into_root().unwrap();

        let result = format_statement(&ast, sql, &FormatConfig::default()).expect("formatted");

        assert!(result.formatted.contains("-- has_decimal"));
    }

    #[test]
    fn formatting_a_comment_before_a_window_definition_is_idempotent() {
        let sql = "SELECT bool_or(a <> b) -- has_decimal\nOVER (PARTITION BY c) FROM s.t";
        let ast = pgls_query::parse(sql).unwrap().into_root().unwrap();
        let config = FormatConfig::default();

        let first = format_statement(&ast, sql, &config)
            .expect("first pass")
            .formatted;
        let reparsed = pgls_query::parse(&first).unwrap().into_root().unwrap();
        let second = format_statement(&reparsed, &first, &config)
            .expect("second pass")
            .formatted;

        assert_eq!(first, second);
    }
}
