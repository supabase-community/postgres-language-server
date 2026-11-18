use pgls_analyse::{RegistryVisitor, RuleFilter, RuleGroup, RuleMeta};
use pgls_diagnostics::{Category, Diagnostic, MessageAndDescription};
use pgls_text_size::{TextRange, TextSize};

/// A specialized diagnostic for the typechecker.
///
/// Type diagnostics are always **errors**.
#[derive(Clone, Debug, Diagnostic, PartialEq)]
#[diagnostic(category = "lint", severity = Warning)]
pub struct SuppressionDiagnostic {
    #[location(span)]
    pub span: TextRange,
    #[description]
    #[message]
    pub message: MessageAndDescription,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SuppressionKind {
    File,
    Line,
    Start,
    End,
}

#[derive(Debug, PartialEq, Clone, Eq)]
/// Represents the suppressed rule, as written in the suppression comment.
/// e.g. `banDropColumn`, or `lint/safety`, or just `lint`.
/// The format of a rule specifier string is `<category>(/<group>(/<rule>))`.
///
/// `RuleSpecifier` can only be constructed from a `&str` that matches a valid
/// [pgls_diagnostics::Category].
pub(crate) enum RuleSpecifier {
    Category(String),
    Group(String, String),
    Rule(String, String, String),
}

/// Whether the analyser has a group named `name`. It visits the groups but not their rules, and
/// doesn't allocate.
struct GroupLookup<'a> {
    name: &'a str,
    found: bool,
}

impl RegistryVisitor for GroupLookup<'_> {
    fn record_group<G: RuleGroup>(&mut self) {
        self.found |= G::NAME == self.name;
    }

    fn record_rule<R>(&mut self)
    where
        R: RuleMeta + 'static,
    {
    }
}

impl RuleSpecifier {
    /// The rule specifier of a diagnostic's category, if it has one that a suppression can name.
    pub(crate) fn of<D: Diagnostic>(diagnostic: &D) -> Option<Self> {
        Self::try_from(diagnostic.category()?.name()).ok()
    }

    fn is_lint_group(group: &str) -> bool {
        let mut lookup = GroupLookup {
            name: group,
            found: false,
        };
        pgls_analyser::visit_registry(&mut lookup);
        lookup.found || matches!(group, "typecheck" | "nursery")
    }

    pub(crate) fn category(&self) -> &str {
        match self {
            RuleSpecifier::Category(rule_category) => rule_category,
            RuleSpecifier::Group(rule_category, _) => rule_category,
            RuleSpecifier::Rule(rule_category, _, _) => rule_category,
        }
    }

    pub(crate) fn group(&self) -> Option<&str> {
        match self {
            RuleSpecifier::Category(_) => None,
            RuleSpecifier::Group(_, gr) => Some(gr),
            RuleSpecifier::Rule(_, gr, _) => Some(gr),
        }
    }

    pub(crate) fn rule(&self) -> Option<&str> {
        match self {
            RuleSpecifier::Rule(_, _, ru) => Some(ru),
            _ => None,
        }
    }

    pub(crate) fn is_disabled(&self, disabled_rules: &[RuleFilter<'_>]) -> bool {
        let (group, rule) = match self {
            RuleSpecifier::Rule(category, group, rule) if category == "lint" => {
                let actual_group: &str = if group.is_empty() {
                    pgls_analyser::METADATA.group_of(rule).unwrap_or("")
                } else {
                    group.as_str()
                };
                (Some(actual_group), Some(rule.as_str()))
            }
            RuleSpecifier::Group(category, group) if category == "lint" => {
                (Some(group.as_str()), None)
            }
            _ => (None, None),
        };
        disabled_rules.iter().any(|filter| match filter {
            RuleFilter::Group(g) => group == Some(*g),
            RuleFilter::Rule(g, r) => group == Some(*g) && rule == Some(*r),
        })
    }
}

impl From<&Category> for RuleSpecifier {
    fn from(category: &Category) -> Self {
        let mut specifiers = category.name().split('/').map(|s| s.to_string());

        let category_str = specifiers.next();
        let group = specifiers.next();
        let rule = specifiers.next();

        match (category_str, group, rule) {
            (Some(c), Some(g), Some(r)) => RuleSpecifier::Rule(c, g, r),
            (Some(c), Some(g), None) => RuleSpecifier::Group(c, g),
            (Some(c), None, None) => RuleSpecifier::Category(c),
            _ => unreachable!(),
        }
    }
}

/// The rule that replaces `rule`, if it was merged into another rule, or `rule` itself.
fn removed(rule: &str) -> &str {
    pgls_analyser::replacement_of_removed_rule(rule).unwrap_or(rule)
}

/// The flat form of a specifier from before rule IDs were flat, if `value` is one.
///
/// All rules used to be in `lint/safety`, so `lint/safety` stands for every lint rule. The
/// `safety` group of today is written without the `lint/` prefix.
fn legacy_replacement(value: &str) -> Option<String> {
    match value.split('/').collect::<Vec<_>>().as_slice() {
        ["lint", "safety"] => Some("lint".into()),
        ["lint", group] if RuleSpecifier::is_lint_group(group) => Some((*group).into()),
        ["lint", _group, rule] => Some(removed(rule).into()),
        ["lint", rule] | [rule] if removed(rule) != *rule => Some(removed(rule).into()),
        _ => None,
    }
}

impl TryFrom<&str> for RuleSpecifier {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let invalid = || "Invalid rule.".to_string();
        let parts: Vec<_> = value.split('/').collect();
        match parts.as_slice() {
            ["lint"] => Ok(Self::Category("lint".into())),
            ["lint", group] if RuleSpecifier::is_lint_group(group) => {
                Ok(Self::Group("lint".into(), (*group).into()))
            }
            ["lint", rule] => {
                let rule = removed(rule);
                if pgls_analyser::METADATA.group_of(rule).is_none() {
                    return Err(invalid());
                }
                Ok(Self::Rule("lint".into(), String::new(), rule.into()))
            }
            ["lint", _group, rule] => {
                let rule = removed(rule);
                if pgls_analyser::METADATA.group_of(rule).is_none() {
                    return Err(invalid());
                }
                Ok(Self::Rule("lint".into(), String::new(), rule.into()))
            }
            ["typecheck"] => Ok(Self::Category("typecheck".into())),
            [group] if RuleSpecifier::is_lint_group(group) => {
                Ok(Self::Group("lint".into(), (*group).into()))
            }
            [rule] if pgls_analyser::METADATA.group_of(removed(rule)).is_some() => Ok(Self::Rule(
                "lint".into(),
                String::new(),
                removed(rule).into(),
            )),
            [_] | [_, _] | [_, _, _] => value
                .parse::<&Category>()
                .map(RuleSpecifier::from)
                .map_err(|_| invalid()),
            _ => Err(invalid()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Suppression {
    pub(crate) suppression_range: TextRange,
    pub(crate) kind: SuppressionKind,
    pub(crate) rule_specifier: RuleSpecifier,
    #[allow(dead_code, reason = "parsed and tested, but not shown to the user yet")]
    pub(crate) explanation: Option<String>,
    /// The flat form of a legacy specifier.
    pub(crate) legacy_replacement: Option<String>,
}

impl Suppression {
    /// The suppression comment on `line`: the line without its leading whitespace, if it then
    /// starts with `-- pgt-ignore` or `-- pgls-ignore`. Whitespace is any Unicode whitespace, so
    /// that a comment indented with a non-breaking space (e.g. pasted from a web page) is a
    /// suppression too. The parser and [`Suppression::from_line`] both use this, so they agree on
    /// which lines are suppressions.
    pub(crate) fn comment_on(line: &str) -> Option<&str> {
        let comment = line.trim_start();
        (comment.starts_with("-- pgt-ignore") || comment.starts_with("-- pgls-ignore"))
            .then_some(comment)
    }

    /// Creates a suppression from a suppression comment line, which `line` must start with after
    /// leading whitespace (see [`Suppression::comment_on`]). Returns an error for any other line.
    pub(crate) fn from_line(line: &str, offset: &TextSize) -> Result<Self, SuppressionDiagnostic> {
        let Some(start_trimmed) = Self::comment_on(line) else {
            return Err(SuppressionDiagnostic {
                span: TextRange::at(*offset, TextSize::of(line)),
                message: MessageAndDescription::from(
                    "This is not a suppression comment.".to_string(),
                ),
            });
        };
        let leading_whitespace_offset = TextSize::of(line) - TextSize::of(start_trimmed);
        let trimmed = start_trimmed.trim_end();

        let full_offset = *offset + leading_whitespace_offset;
        let span = TextRange::at(full_offset, TextSize::of(trimmed));

        let (line, explanation) = match trimmed.split_once(':') {
            Some((suppr, explanation)) => (suppr, Some(explanation.trim())),
            None => (trimmed, None),
        };

        let mut parts = line.split_whitespace();

        let _ = parts.next();
        // The line starts with `-- pgt-ignore` or `-- pgls-ignore`, so the tag is there.
        let kind = match parts.next().unwrap_or_default() {
            "pgt-ignore-all" | "pgls-ignore-all" => SuppressionKind::File,
            "pgt-ignore-start" | "pgls-ignore-start" => SuppressionKind::Start,
            "pgt-ignore-end" | "pgls-ignore-end" => SuppressionKind::End,
            "pgt-ignore" | "pgls-ignore" => SuppressionKind::Line,
            k => {
                return Err(SuppressionDiagnostic {
                    span,
                    message: MessageAndDescription::from(format!(
                        "'{k}' is not a valid suppression tag.",
                    )),
                });
            }
        };

        let specifier_str = match parts.next() {
            Some(it) => it,
            None => {
                return Err(SuppressionDiagnostic {
                    span,
                    message: MessageAndDescription::from(
                        "You must specify which lints to suppress.".to_string(),
                    ),
                });
            }
        };

        let legacy_replacement = legacy_replacement(specifier_str);
        let rule_specifier = match legacy_replacement.as_deref() {
            Some("lint") => RuleSpecifier::Category("lint".into()),
            _ => RuleSpecifier::try_from(specifier_str).map_err(|e| SuppressionDiagnostic {
                span,
                message: MessageAndDescription::from(e),
            })?,
        };

        Ok(Self {
            rule_specifier,
            kind,
            suppression_range: span,
            explanation: explanation.map(|e| e.to_string()),
            legacy_replacement,
        })
    }

    /// A deprecation warning if the suppression uses a legacy specifier.
    pub(crate) fn to_legacy_diagnostic(&self) -> Option<SuppressionDiagnostic> {
        let replacement = self.legacy_replacement.as_ref()?;
        Some(SuppressionDiagnostic {
            span: self.suppression_range,
            message: MessageAndDescription::from(format!(
                "This rule specifier is deprecated. Use `{replacement}` instead."
            )),
        })
    }

    pub(crate) fn matches(&self, diagnostic_specifier: &RuleSpecifier) -> bool {
        let d_category = diagnostic_specifier.category();
        let d_group = diagnostic_specifier.group();
        let d_rule = diagnostic_specifier.rule();

        match &self.rule_specifier {
            RuleSpecifier::Category(cat) if cat == "typecheck" => {
                d_category == "typecheck"
                    || (d_category == "lint"
                        && (d_group == Some("typecheck")
                            || d_rule.is_some_and(|rule| {
                                pgls_analyser::METADATA.group_of(rule) == Some("typecheck")
                            })))
            }
            RuleSpecifier::Category(cat) => cat == d_category,
            RuleSpecifier::Group(cat, group) if cat == "lint" => {
                (d_category == "typecheck" && group == "typecheck")
                    || (d_category == "lint"
                        && (d_group == Some(group.as_str())
                            || d_rule.is_some_and(|rule| {
                                pgls_analyser::METADATA.group_of(rule) == Some(group.as_str())
                            })))
            }
            RuleSpecifier::Group(cat, group) => {
                cat == d_category && Some(group.as_str()) == d_group
            }
            RuleSpecifier::Rule(cat, _, rule) if cat == "lint" => {
                d_category == "lint" && d_rule == Some(rule.as_str())
            }
            RuleSpecifier::Rule(cat, group, rule) => {
                cat == d_category
                    && Some(group.as_str()) == d_group
                    && Some(rule.as_str()) == d_rule
            }
        }
    }

    pub(crate) fn to_disabled_diagnostic(&self) -> SuppressionDiagnostic {
        SuppressionDiagnostic {
            span: self.suppression_range,
            message: MessageAndDescription::from(
                "This rule has been disabled via the configuration. The suppression has no effect."
                    .to_string(),
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RangeSuppression {
    pub(crate) suppressed_range: TextRange,
    pub(crate) start_suppression: Suppression,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pgls_text_size::{TextRange, TextSize};

    #[test]
    fn test_suppression_from_line_rule() {
        let line = "-- pgt-ignore banDropColumn: explanation";
        let offset = &TextSize::new(0);
        let suppression = Suppression::from_line(line, offset).unwrap();

        assert_eq!(suppression.kind, SuppressionKind::Line);
        assert_eq!(
            suppression.rule_specifier,
            RuleSpecifier::Rule(
                "lint".to_string(),
                "".to_string(),
                "banDropColumn".to_string()
            )
        );
        assert_eq!(suppression.explanation.as_deref(), Some("explanation"));
    }

    #[test]
    fn test_suppression_from_line_group() {
        let line = "-- pgt-ignore safety: explanation";
        let offset = &TextSize::new(0);
        let suppression = Suppression::from_line(line, offset).unwrap();

        assert_eq!(suppression.kind, SuppressionKind::Line);
        assert_eq!(
            suppression.rule_specifier,
            RuleSpecifier::Group("lint".to_string(), "safety".to_string())
        );
        assert_eq!(suppression.explanation.as_deref(), Some("explanation"));
    }

    #[test]
    fn test_suppression_from_line_category() {
        let line = "-- pgt-ignore lint";
        let offset = &TextSize::new(0);
        let suppression = Suppression::from_line(line, offset).unwrap();

        assert_eq!(suppression.kind, SuppressionKind::Line);
        assert_eq!(
            suppression.rule_specifier,
            RuleSpecifier::Category("lint".to_string())
        );
    }

    #[test]
    fn test_suppression_from_line_category_with_explanation() {
        let line = "-- pgt-ignore lint: explanation";
        let offset = &TextSize::new(0);
        let suppression = Suppression::from_line(line, offset).unwrap();

        assert_eq!(suppression.kind, SuppressionKind::Line);
        assert_eq!(
            suppression.rule_specifier,
            RuleSpecifier::Category("lint".to_string())
        );
        assert_eq!(suppression.explanation.as_deref(), Some("explanation"));
    }

    #[test]
    fn test_suppression_from_line_file_kind() {
        let line = "-- pgt-ignore-all banDropColumn: explanation";
        let offset = &TextSize::new(0);
        let suppression = Suppression::from_line(line, offset).unwrap();

        assert_eq!(suppression.kind, SuppressionKind::File);
        assert_eq!(
            suppression.rule_specifier,
            RuleSpecifier::Rule(
                "lint".to_string(),
                "".to_string(),
                "banDropColumn".to_string()
            )
        );
        assert_eq!(suppression.explanation.as_deref(), Some("explanation"));
    }

    #[test]
    fn test_suppression_from_line_start_kind() {
        let line = "-- pgt-ignore-start banDropColumn: explanation";
        let offset = &TextSize::new(0);
        let suppression = Suppression::from_line(line, offset).unwrap();

        assert_eq!(suppression.kind, SuppressionKind::Start);
        assert_eq!(
            suppression.rule_specifier,
            RuleSpecifier::Rule(
                "lint".to_string(),
                "".to_string(),
                "banDropColumn".to_string()
            )
        );
        assert_eq!(suppression.explanation.as_deref(), Some("explanation"));
    }

    #[test]
    fn test_suppression_from_line_end_kind() {
        let line = "-- pgt-ignore-end banDropColumn: explanation";
        let offset = &TextSize::new(0);
        let suppression = Suppression::from_line(line, offset).unwrap();

        assert_eq!(suppression.kind, SuppressionKind::End);
        assert_eq!(
            suppression.rule_specifier,
            RuleSpecifier::Rule(
                "lint".to_string(),
                "".to_string(),
                "banDropColumn".to_string()
            )
        );
        assert_eq!(suppression.explanation.as_deref(), Some("explanation"));
    }

    #[test]
    fn test_suppression_from_line_non_ascii_whitespace() {
        let line = "\u{00A0}-- pgls-ignore\u{00A0}banDropColumn";
        let suppression = Suppression::from_line(line, &TextSize::new(10)).unwrap();

        assert_eq!(suppression.kind, SuppressionKind::Line);
        assert_eq!(suppression.rule_specifier.rule(), Some("banDropColumn"));
        assert_eq!(
            suppression.suppression_range,
            TextRange::new(TextSize::new(12), TextSize::new(10) + TextSize::of(line))
        );

        // Any other line is an error, not a panic.
        assert!(
            Suppression::from_line("select 1; -- pgls-ignore lint", &TextSize::new(0)).is_err()
        );
        assert!(Suppression::from_line("", &TextSize::new(0)).is_err());
    }

    #[test]
    fn test_suppression_span_with_offset() {
        let line = "    \n-- pgt-ignore banDropColumn: explanation";
        let offset = TextSize::new(5);
        let suppression = Suppression::from_line(line, &offset).unwrap();

        let expected_start = offset + TextSize::new(5);
        let expected_len = TextSize::new(line.trim_ascii().len() as u32);

        let expected_end = expected_start + expected_len;
        let expected_span = TextRange::new(expected_start, expected_end);

        assert_eq!(suppression.suppression_range, expected_span);
    }

    #[test]
    fn test_suppression_from_line_invalid_tag_and_missing_specifier() {
        let lines = vec![
            "-- pgt-ignore-foo banDropColumn: explanation",
            "-- pgt-ignore foo banDropColumn: explanation",
            "-- pgt-ignore xyz banDropColumn: explanation",
            "-- pgt-ignore",
        ];
        let offset = &TextSize::new(0);
        for line in lines {
            let result = Suppression::from_line(line, offset);
            assert!(result.is_err(), "Expected error for line: {line}");
        }
    }

    #[test]
    fn test_suppression_matches() {
        let cases = vec![
            // the category works for all groups & rules
            ("-- pgt-ignore lint", "lint/banDropNotNull", true),
            ("-- pgt-ignore lint", "lint/banDropColumn", true),
            // the group works for all rules in that group
            ("-- pgt-ignore lint/destructive", "lint/banDropColumn", true),
            ("-- pgt-ignore lint", "typecheck", false),
            ("-- pgt-ignore lint/safety", "typecheck", false),
            // a specific supppression only works for that same rule
            ("-- pgt-ignore banDropColumn", "lint/banDropColumn", true),
            ("-- pgt-ignore banDropColumn", "lint/banDropTable", false),
        ];

        let offset = &TextSize::new(0);

        for (suppr_line, specifier_str, expected) in cases {
            let suppression = Suppression::from_line(suppr_line, offset).unwrap();
            let specifier = RuleSpecifier::try_from(specifier_str).unwrap();
            assert_eq!(
                suppression.matches(&specifier),
                expected,
                "Suppression line '{suppr_line}' vs specifier '{specifier_str}' should be {expected}"
            );
        }
    }

    #[test]
    fn flat_and_legacy_specifiers_are_normalized_and_validated() {
        assert_eq!(
            RuleSpecifier::try_from("banDropColumn").unwrap(),
            RuleSpecifier::try_from("lint/banDropColumn").unwrap()
        );
        assert_eq!(
            RuleSpecifier::try_from("lint/safety/banDropColumn").unwrap(),
            RuleSpecifier::try_from("banDropColumn").unwrap()
        );
        assert_eq!(
            RuleSpecifier::try_from("preferBigintOverInt").unwrap(),
            RuleSpecifier::try_from("preferBigInt").unwrap()
        );
        assert_eq!(
            RuleSpecifier::try_from("preferBigintOverSmallint").unwrap(),
            RuleSpecifier::try_from("preferBigInt").unwrap()
        );
        assert_eq!(
            RuleSpecifier::try_from("lint/safety/concurrentRefreshMatviewLock").unwrap(),
            RuleSpecifier::try_from("requireConcurrentRefreshMatview").unwrap()
        );
        assert!(RuleSpecifier::try_from("notARule").is_err());
        assert_eq!(
            RuleSpecifier::try_from("destructive").unwrap(),
            RuleSpecifier::Group("lint".into(), "destructive".into())
        );
        assert_eq!(
            RuleSpecifier::try_from("lint/nursery").unwrap(),
            RuleSpecifier::Group("lint".into(), "nursery".into())
        );
    }

    #[test]
    fn legacy_specifiers_are_deprecated() {
        let cases = [
            (
                "-- pgls-ignore lint/safety/banDropColumn",
                Some("banDropColumn"),
            ),
            (
                "-- pgls-ignore lint/destructive/banDropColumn",
                Some("banDropColumn"),
            ),
            ("-- pgls-ignore preferBigintOverInt", Some("preferBigInt")),
            (
                "-- pgls-ignore lint/preferBigintOverSmallint",
                Some("preferBigInt"),
            ),
            ("-- pgls-ignore lint/safety", Some("lint")),
            ("-- pgls-ignore lint/destructive", Some("destructive")),
            ("-- pgls-ignore banDropColumn", None),
            ("-- pgls-ignore lint/banDropColumn", None),
            ("-- pgls-ignore safety", None),
            ("-- pgls-ignore lint", None),
            ("-- pgls-ignore typecheck", None),
        ];
        for (line, replacement) in cases {
            let suppression = Suppression::from_line(line, &TextSize::new(0)).unwrap();
            assert_eq!(
                suppression.legacy_replacement.as_deref(),
                replacement,
                "{line}"
            );
            assert_eq!(
                suppression.to_legacy_diagnostic().is_some(),
                replacement.is_some(),
                "{line}"
            );
        }
    }

    #[test]
    fn legacy_safety_group_suppresses_every_lint_rule() {
        // All rules used to be in `lint/safety`.
        let legacy =
            Suppression::from_line("-- pgls-ignore lint/safety", &TextSize::new(0)).unwrap();
        let group = Suppression::from_line("-- pgls-ignore safety", &TextSize::new(0)).unwrap();
        let destructive = RuleSpecifier::try_from("lint/banDropColumn").unwrap();
        assert!(legacy.matches(&destructive));
        assert!(!group.matches(&destructive));
    }

    #[test]
    fn typecheck_specifier_matches_category_and_lint_group() {
        let category = RuleSpecifier::try_from("typecheck").unwrap();
        let group = RuleSpecifier::try_from("lint/typecheck").unwrap();
        let explain_diagnostic = RuleSpecifier::Category("typecheck".into());
        let lint_diagnostic =
            RuleSpecifier::Rule("lint".into(), "typecheck".into(), "futureRule".into());
        let category_suppression =
            Suppression::from_line("-- pgls-ignore typecheck", &TextSize::new(0)).unwrap();
        let group_suppression =
            Suppression::from_line("-- pgls-ignore lint/typecheck", &TextSize::new(0)).unwrap();
        assert!(category_suppression.matches(&explain_diagnostic));
        assert!(category_suppression.matches(&lint_diagnostic));
        assert!(group_suppression.matches(&explain_diagnostic));
        assert!(group_suppression.matches(&lint_diagnostic));
        assert_eq!(category, RuleSpecifier::Category("typecheck".into()));
        assert_eq!(
            group,
            RuleSpecifier::Group("lint".into(), "typecheck".into())
        );
    }

    #[test]
    fn test_rule_specifier_is_disabled() {
        use pgls_analyse::RuleFilter;

        // Group filter disables all rules in that group
        let spec = RuleSpecifier::Rule(
            "lint".to_string(),
            "safety".to_string(),
            "banDropColumn".to_string(),
        );
        let disabled = vec![RuleFilter::Group("safety")];
        assert!(spec.is_disabled(&disabled));

        let spec2 = RuleSpecifier::Rule(
            "lint".to_string(),
            "safety".to_string(),
            "banDropColumn".to_string(),
        );
        let disabled2 = vec![RuleFilter::Rule("safety", "banDropColumn")];
        assert!(spec2.is_disabled(&disabled2));

        let disabled3 = vec![RuleFilter::Rule("safety", "otherRule")];
        assert!(!spec2.is_disabled(&disabled3));

        let disabled4 = vec![RuleFilter::Group("perf")];
        assert!(!spec.is_disabled(&disabled4));

        // one match is enough
        let disabled5 = vec![
            RuleFilter::Group("perf"),
            RuleFilter::Rule("safety", "banDropColumn"),
        ];
        assert!(spec.is_disabled(&disabled5));
    }

    #[test]
    fn test_pgls_prefix_line_suppressions() {
        let line = "-- pgls-ignore banDropColumn: explanation";
        let offset = &TextSize::new(0);
        let suppression = Suppression::from_line(line, offset).unwrap();

        assert_eq!(suppression.kind, SuppressionKind::Line);
        assert_eq!(
            suppression.rule_specifier,
            RuleSpecifier::Rule(
                "lint".to_string(),
                "".to_string(),
                "banDropColumn".to_string()
            )
        );
        assert_eq!(suppression.explanation.as_deref(), Some("explanation"));
    }

    #[test]
    fn test_pgls_prefix_file_kind() {
        let line = "-- pgls-ignore-all safety: explanation";
        let offset = &TextSize::new(0);
        let suppression = Suppression::from_line(line, offset).unwrap();

        assert_eq!(suppression.kind, SuppressionKind::File);
        assert_eq!(
            suppression.rule_specifier,
            RuleSpecifier::Group("lint".to_string(), "safety".to_string())
        );
        assert_eq!(suppression.explanation.as_deref(), Some("explanation"));
    }

    #[test]
    fn test_pgls_prefix_start_and_end_kind() {
        let start_line = "-- pgls-ignore-start typecheck";
        let end_line = "-- pgls-ignore-end typecheck";
        let offset = &TextSize::new(0);

        let start_suppression = Suppression::from_line(start_line, offset).unwrap();
        assert_eq!(start_suppression.kind, SuppressionKind::Start);
        assert_eq!(
            start_suppression.rule_specifier,
            RuleSpecifier::Category("typecheck".to_string())
        );

        let end_suppression = Suppression::from_line(end_line, offset).unwrap();
        assert_eq!(end_suppression.kind, SuppressionKind::End);
        assert_eq!(
            end_suppression.rule_specifier,
            RuleSpecifier::Category("typecheck".to_string())
        );
    }
}
