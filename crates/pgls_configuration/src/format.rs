use crate::{Merge as MergeTrait, StringSet};
use bpaf::Bpaf;
use pgls_configuration_macros::{Merge, Partial};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    num::{NonZeroU8, NonZeroU16},
    ops::{Deref, DerefMut},
    str::FromStr,
};

/// Function names mapped to the number of adjacent arguments that form one logical unit.
///
/// Names match the final, unqualified function identifier case-insensitively. For example,
/// `{ "my_pair_func": 2 }` keeps each key/value pair together when the argument list wraps.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(transparent))]
#[serde(transparent)]
pub struct FunctionArgumentGroups(BTreeMap<String, NonZeroU8>);

impl Deref for FunctionArgumentGroups {
    type Target = BTreeMap<String, NonZeroU8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FunctionArgumentGroups {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromStr for FunctionArgumentGroups {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut groups = BTreeMap::new();

        for entry in value.split(',').filter(|entry| !entry.trim().is_empty()) {
            let (name, size) = entry
                .split_once(':')
                .ok_or_else(|| format!("expected function:size, got '{entry}'"))?;
            let size = size
                .trim()
                .parse::<NonZeroU8>()
                .map_err(|_| format!("group size must be between 1 and 255, got '{size}'"))?;
            groups.insert(name.trim().to_lowercase(), size);
        }

        Ok(Self(groups))
    }
}

impl MergeTrait for FunctionArgumentGroups {
    fn merge_with(&mut self, other: Self) {
        self.0.extend(other.0);
    }
}

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
            _ => Err("expected 'spaces' or 'tabs'"),
        }
    }
}

impl From<IndentStyle> for pgls_pretty_print::IndentStyle {
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
            _ => Err("expected 'upper' or 'lower'"),
        }
    }
}

impl From<KeywordCase> for pgls_pretty_print::KeywordCase {
    fn from(case: KeywordCase) -> Self {
        match case {
            KeywordCase::Upper => Self::Upper,
            KeywordCase::Lower => Self::Lower,
        }
    }
}

/// Where a comma sits when a list breaks across lines.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Merge, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum CommaStyle {
    #[default]
    Trailing,
    Leading,
}

impl FromStr for CommaStyle {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "trailing" => Ok(Self::Trailing),
            "leading" => Ok(Self::Leading),
            _ => Err("expected 'trailing' or 'leading'"),
        }
    }
}

impl From<CommaStyle> for pgls_pretty_print::CommaStyle {
    fn from(style: CommaStyle) -> Self {
        match style {
            CommaStyle::Trailing => Self::Trailing,
            CommaStyle::Leading => Self::Leading,
        }
    }
}

/// Where a boolean operator sits when a condition breaks across lines.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Merge, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum LogicalOperatorPlacement {
    #[default]
    Trailing,
    Leading,
}

impl FromStr for LogicalOperatorPlacement {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "trailing" => Ok(Self::Trailing),
            "leading" => Ok(Self::Leading),
            _ => Err("expected 'trailing' or 'leading'"),
        }
    }
}

impl From<LogicalOperatorPlacement> for pgls_pretty_print::LogicalOperatorPlacement {
    fn from(placement: LogicalOperatorPlacement) -> Self {
        match placement {
            LogicalOperatorPlacement::Trailing => Self::Trailing,
            LogicalOperatorPlacement::Leading => Self::Leading,
        }
    }
}

/// How a statement is laid out across lines.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Merge, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum Layout {
    #[default]
    Fit,
    Expanded,
}

impl FromStr for Layout {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "fit" => Ok(Self::Fit),
            "expanded" => Ok(Self::Expanded),
            _ => Err("expected 'fit' or 'expanded'"),
        }
    }
}

impl From<Layout> for pgls_pretty_print::Layout {
    fn from(layout: Layout) -> Self {
        match layout {
            Layout::Fit => Self::Fit,
            Layout::Expanded => Self::Expanded,
        }
    }
}

/// How an explicit cast is spelled.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Merge, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum CastStyle {
    #[default]
    Cast,
    Operator,
}

impl FromStr for CastStyle {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cast" => Ok(Self::Cast),
            "operator" => Ok(Self::Operator),
            _ => Err("expected 'cast' or 'operator'"),
        }
    }
}

impl From<CastStyle> for pgls_pretty_print::CastStyle {
    fn from(style: CastStyle) -> Self {
        match style {
            CastStyle::Cast => Self::Cast,
            CastStyle::Operator => Self::Operator,
        }
    }
}

/// Where the body of a clause starts.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Merge, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum ClauseBodyStyle {
    #[default]
    Break,
    Compact,
}

impl FromStr for ClauseBodyStyle {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "break" => Ok(Self::Break),
            "compact" => Ok(Self::Compact),
            _ => Err("expected 'break' or 'compact'"),
        }
    }
}

impl From<ClauseBodyStyle> for pgls_pretty_print::ClauseBodyStyle {
    fn from(style: ClauseBodyStyle) -> Self {
        match style {
            ClauseBodyStyle::Break => Self::Break,
            ClauseBodyStyle::Compact => Self::Compact,
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
    #[partial(bpaf(long("line-width"), argument("NUMBER")))]
    pub line_width: NonZeroU16,
    /// Number of spaces (or tab width) for indentation. Default: 2.
    #[partial(bpaf(long("indent-size"), argument("NUMBER")))]
    pub indent_size: NonZeroU8,
    /// Indentation style: "spaces" or "tabs". Default: "spaces".
    #[partial(bpaf(long("indent-style"), argument("spaces|tabs")))]
    pub indent_style: IndentStyle,
    /// Keyword casing: "upper" or "lower". Default: "lower".
    #[partial(bpaf(long("keyword-case"), argument("upper|lower")))]
    pub keyword_case: KeywordCase,
    /// Constant casing (NULL, TRUE, FALSE): "upper" or "lower". Default: "lower".
    #[partial(bpaf(long("constant-case"), argument("upper|lower")))]
    pub constant_case: KeywordCase,
    /// Data type casing (text, varchar, int): "upper" or "lower". Default: "lower".
    #[partial(bpaf(long("type-case"), argument("upper|lower")))]
    pub type_case: KeywordCase,
    /// Where a comma sits when a list breaks: "trailing" or "leading". Default: "trailing".
    #[partial(bpaf(long("comma-style"), argument("trailing|leading")))]
    pub comma_style: CommaStyle,
    /// Where a boolean operator sits when a condition breaks: "trailing" or "leading".
    /// Default: "trailing".
    #[partial(bpaf(long("logical-operator-placement"), argument("trailing|leading")))]
    pub logical_operator_placement: LogicalOperatorPlacement,
    /// How a statement is laid out: "fit" breaks only when a line would exceed the line width,
    /// "expanded" always breaks between clauses, between the items of a select list, before
    /// every AND/OR of a condition and before every CASE branch. Default: "fit".
    #[partial(bpaf(long("layout"), argument("fit|expanded")))]
    pub layout: Layout,
    /// How VALUES rows are laid out: "fit" breaks only past the line width,
    /// "expanded" puts each row on its own indented line. Default: "fit".
    #[partial(bpaf(long("values-row-layout"), argument("fit|expanded")))]
    pub values_row_layout: Layout,
    /// How an explicit cast is spelled: "cast" for `CAST(x AS t)`, "operator" for `x::t`.
    /// Default: "cast".
    #[partial(bpaf(long("cast-style"), argument("cast|operator")))]
    pub cast_style: CastStyle,
    /// Where the body of a clause starts: "break" for a new line, "compact" to keep the first
    /// element on the keyword line. Compact also keeps a select list of a single item on the
    /// select line. Default: "break".
    #[partial(bpaf(long("clause-body-style"), argument("break|compact")))]
    pub clause_body_style: ClauseBodyStyle,
    /// If `true`, the terminating semicolon goes on its own line when the statement spans several
    /// lines. Default: `false`.
    #[partial(bpaf(long("isolate-semicolon"), argument("true|false")))]
    pub isolate_semicolon: bool,
    /// Function names mapped to the number of adjacent arguments in one logical group.
    /// `json_build_object` and `jsonb_build_object` are grouped in pairs by default;
    /// configure a size of `1` to opt out.
    #[partial(bpaf(hide))]
    pub function_argument_groups: FunctionArgumentGroups,
    /// If `true`, skip formatting of SQL function bodies (keep them verbatim). Default: `false`.
    #[partial(bpaf(long("skip-fn-bodies"), argument("true|false")))]
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
            line_width: NonZeroU16::new(100).unwrap(),
            indent_size: NonZeroU8::new(2).unwrap(),
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
            function_argument_groups: FunctionArgumentGroups::default(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_values_row_layout_and_rejects_unknown_values() {
        let config: PartialFormatConfiguration =
            serde_json::from_str(r#"{"valuesRowLayout": "expanded"}"#).unwrap();
        assert_eq!(config.values_row_layout, Some(Layout::Expanded));
        assert!(
            serde_json::from_str::<PartialFormatConfiguration>(r#"{"valuesRowLayout": "unknown"}"#)
                .is_err()
        );
    }

    #[test]
    fn rejects_a_zero_line_width_and_indent_size() {
        for json in [r#"{"lineWidth": 0}"#, r#"{"indentSize": 0}"#] {
            assert!(serde_json::from_str::<PartialFormatConfiguration>(json).is_err());
        }
        let config: PartialFormatConfiguration =
            serde_json::from_str(r#"{"lineWidth": 80, "indentSize": 4}"#).unwrap();
        assert_eq!(config.line_width.map(NonZeroU16::get), Some(80));
        assert_eq!(config.indent_size.map(NonZeroU8::get), Some(4));
    }

    #[test]
    fn deserializes_function_argument_groups() {
        let config: PartialFormatConfiguration = serde_json::from_str(
            r#"{
                "functionArgumentGroups": {
                    "json_build_object": 2,
                    "jsonb_build_object": 2
                }
            }"#,
        )
        .expect("function argument groups should deserialize");

        let groups = config
            .function_argument_groups
            .expect("function argument groups should be configured");

        assert_eq!(groups["json_build_object"].get(), 2);
        assert_eq!(groups["jsonb_build_object"].get(), 2);
    }

    #[test]
    fn parses_function_argument_groups_from_str() {
        let groups = FunctionArgumentGroups::from_str("json_build_object:2,jsonb_build_object:2")
            .expect("function argument groups should parse");

        assert_eq!(groups["json_build_object"].get(), 2);
        assert_eq!(groups["jsonb_build_object"].get(), 2);
    }

    #[test]
    fn rejects_zero_sized_function_argument_groups() {
        let result: Result<PartialFormatConfiguration, _> = serde_json::from_str(
            r#"{
                "functionArgumentGroups": {
                    "jsonb_build_object": 0
                }
            }"#,
        );

        assert!(result.is_err());
    }
}
