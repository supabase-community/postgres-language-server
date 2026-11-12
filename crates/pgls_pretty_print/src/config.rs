//! The formatter options, and the part of them the renderer reads.

use std::collections::BTreeMap;

use crate::renderer::RenderConfig;

/// Whether indentation is written with spaces or tabs.
#[derive(Debug, Clone)]
pub enum IndentStyle {
    Spaces,
    Tabs,
}

/// The case of keywords, constants or type names.
#[derive(Debug, Clone, Default)]
pub enum KeywordCase {
    Upper,
    #[default]
    Lower,
}

/// How a statement is laid out across lines.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Layout {
    /// Break only when a line would exceed the line width.
    #[default]
    Fit,
    /// Always break between the clauses of a statement, whatever the width. The line width then
    /// only governs breaking inside a clause.
    Expanded,
}

/// How an explicit cast is spelled.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum CastStyle {
    /// `CAST(expr AS type)`.
    #[default]
    Cast,
    /// `expr::type`.
    Operator,
}

/// Where the body of a clause starts.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum ClauseBodyStyle {
    /// The body starts on the line after the keyword.
    #[default]
    Break,
    /// The first element of the body stays on the keyword line: `FROM plan`. A select list of a
    /// single item stays on the SELECT line: `SELECT 1`.
    Compact,
}

/// Where a comma sits when a list breaks across lines.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum CommaStyle {
    /// `a,` at the end of the line.
    #[default]
    Trailing,
    /// `, a` at the start of the continuation line.
    Leading,
}

/// Where a boolean operator sits when a condition breaks across lines.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum LogicalOperatorPlacement {
    /// `a = 1 AND` at the end of the line.
    #[default]
    Trailing,
    /// `AND a = 1` at the start of the continuation line.
    Leading,
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
    /// Where a comma sits when a list breaks. Default: Trailing.
    pub comma_style: CommaStyle,
    /// Where a boolean operator sits when a condition breaks. Default: Trailing.
    pub logical_operator_placement: LogicalOperatorPlacement,
    /// How a statement is laid out across lines. Default: Fit.
    pub layout: Layout,
    /// How VALUES rows are laid out: fit within the line width or one row per line.
    /// Default: Fit.
    pub values_row_layout: Layout,
    /// How an explicit cast is spelled. Default: Cast.
    pub cast_style: CastStyle,
    /// Where the body of a clause starts. Default: Break.
    pub clause_body_style: ClauseBodyStyle,
    /// Put the terminating semicolon on its own line when the statement spans several lines.
    /// Default: false.
    pub isolate_semicolon: bool,
    /// Function names mapped to the number of adjacent arguments in one logical group.
    pub function_argument_groups: BTreeMap<String, usize>,
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
            comma_style: CommaStyle::default(),
            logical_operator_placement: LogicalOperatorPlacement::default(),
            layout: Layout::default(),
            values_row_layout: Layout::default(),
            cast_style: CastStyle::default(),
            clause_body_style: ClauseBodyStyle::default(),
            isolate_semicolon: false,
            function_argument_groups: BTreeMap::new(),
        }
    }
}

/// The options the renderer reads. The others decide which layout events the emitter writes.
impl From<&FormatConfig> for RenderConfig {
    fn from(config: &FormatConfig) -> Self {
        let FormatConfig {
            line_width,
            indent_size,
            indent_style,
            keyword_case,
            constant_case,
            type_case,
            comma_style: _,
            logical_operator_placement: _,
            layout: _,
            values_row_layout: _,
            cast_style: _,
            clause_body_style: _,
            isolate_semicolon,
            function_argument_groups: _,
        } = config;

        Self {
            max_line_length: *line_width,
            indent_size: *indent_size,
            indent_style: indent_style.clone(),
            keyword_case: keyword_case.clone(),
            constant_case: constant_case.clone(),
            type_case: type_case.clone(),
            isolate_semicolon: *isolate_semicolon,
        }
    }
}
