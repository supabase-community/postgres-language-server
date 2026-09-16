mod codegen;
pub mod comments;
pub mod emitter;
pub mod nodes;
pub mod normalize;
pub mod renderer;

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
    // A comment that no node follows cannot be placed. Refusing here preserves the original text,
    // which is safer than emitting a statement that would silently drop it.
    let attached = comments::attach_comments(sql, ast);
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
    fn a_statement_with_an_unplaceable_comment_is_refused() {
        let sql = "SELECT 1 FROM s.t -- trailing";
        let ast = pgls_query::parse(sql).unwrap().into_root().unwrap();

        let error = format_statement(&ast, sql, &FormatConfig::default())
            .expect_err("the trailing comment has no node after it");

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
}
