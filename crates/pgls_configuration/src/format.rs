use crate::StringSet;
use bpaf::Bpaf;
use pgls_configuration_macros::{Merge, Partial};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Indentation style for the formatter.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Merge, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum IndentStyle {
    /// Use spaces for indentation (default).
    #[default]
    Spaces,
    /// Use tabs for indentation.
    Tabs,
}

impl FromStr for IndentStyle {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "spaces" => Ok(Self::Spaces),
            "tabs" => Ok(Self::Tabs),
            _ => Err("Value not supported for IndentStyle. Use 'spaces' or 'tabs'."),
        }
    }
}

impl From<IndentStyle> for pgls_pretty_print::renderer::IndentStyle {
    fn from(style: IndentStyle) -> Self {
        match style {
            IndentStyle::Spaces => Self::Spaces,
            IndentStyle::Tabs => Self::Tabs,
        }
    }
}

/// Keyword casing style for the formatter.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Merge, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum KeywordCase {
    /// Use uppercase keywords (SELECT, FROM, WHERE).
    Upper,
    /// Use lowercase keywords (select, from, where). Default.
    #[default]
    Lower,
}

impl FromStr for KeywordCase {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "upper" => Ok(Self::Upper),
            "lower" => Ok(Self::Lower),
            _ => Err("Value not supported for KeywordCase. Use 'upper' or 'lower'."),
        }
    }
}

impl From<KeywordCase> for pgls_pretty_print::renderer::KeywordCase {
    fn from(case: KeywordCase) -> Self {
        match case {
            KeywordCase::Upper => Self::Upper,
            KeywordCase::Lower => Self::Lower,
        }
    }
}

/// The configuration for SQL formatting.
#[derive(Clone, Debug, Deserialize, Eq, Partial, PartialEq, Serialize)]
#[partial(derive(Bpaf, Clone, Eq, PartialEq, Merge))]
#[partial(cfg_attr(feature = "schema", derive(schemars::JsonSchema)))]
#[partial(serde(rename_all = "camelCase", default, deny_unknown_fields))]
pub struct FormatConfiguration {
    /// If `false`, it disables the formatter. `true` by default.
    #[partial(bpaf(hide))]
    pub enabled: bool,
    /// Maximum line width before breaking. Default: 100.
    #[partial(bpaf(long("line-width")))]
    pub line_width: u16,
    /// Number of spaces (or tab width) for indentation. Default: 2.
    #[partial(bpaf(long("indent-size")))]
    pub indent_size: u8,
    /// Indentation style: "spaces" or "tabs". Default: "spaces".
    #[partial(bpaf(long("indent-style")))]
    pub indent_style: IndentStyle,
    /// Keyword casing: "upper" or "lower". Default: "lower".
    #[partial(bpaf(long("keyword-case")))]
    pub keyword_case: KeywordCase,
    /// Constant casing (NULL, TRUE, FALSE): "upper" or "lower". Default: "lower".
    #[partial(bpaf(long("constant-case")))]
    pub constant_case: KeywordCase,
    /// Data type casing (text, varchar, int): "upper" or "lower". Default: "lower".
    #[partial(bpaf(long("type-case")))]
    pub type_case: KeywordCase,
    /// If `true`, skip formatting of SQL function bodies (keep them verbatim). Default: `false`.
    #[partial(bpaf(long("skip-fn-bodies")))]
    pub skip_fn_bodies: bool,
    /// A list of Unix shell style patterns. The formatter will ignore files/folders that will match these patterns.
    #[partial(bpaf(hide))]
    pub ignore: StringSet,
    /// A list of Unix shell style patterns. The formatter will include files/folders that will match these patterns.
    #[partial(bpaf(hide))]
    pub include: StringSet,
}

impl Default for FormatConfiguration {
    fn default() -> Self {
        Self {
            enabled: true,
            line_width: 100,
            indent_size: 2,
            indent_style: IndentStyle::Spaces,
            keyword_case: KeywordCase::default(),
            constant_case: KeywordCase::default(),
            type_case: KeywordCase::default(),
            skip_fn_bodies: false,
            ignore: Default::default(),
            include: Default::default(),
        }
    }
}

impl FormatConfiguration {
    pub const fn is_disabled(&self) -> bool {
        !self.enabled
    }
}

impl PartialFormatConfiguration {
    pub const fn is_disabled(&self) -> bool {
        matches!(self.enabled, Some(false))
    }
}
