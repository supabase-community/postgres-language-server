use pgls_analyse::RuleFilter;
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
/// e.g. `lint/safety/banDropColumn`, or `lint/safety`, or just `lint`.
/// The format of a rule specifier string is `<category>(/<group>(/<rule>))`.
///
/// `RuleSpecifier` can only be constructed from a `&str` that matches a valid
/// [pgls_diagnostics::Category].
pub(crate) enum RuleSpecifier {
    Category(String),
    Group(String, String),
    Rule(String, String, String),
}

impl RuleSpecifier {
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
        // note: it is not possible to disable entire categories via the config
        let group = self.group();
        let rule = self.rule();

        disabled_rules.iter().any(|r| match r {
            RuleFilter::Group(gr) => group.is_some_and(|specifier_group| specifier_group == *gr),
            RuleFilter::Rule(gr, ru) => group.is_some_and(|specifier_group| {
                rule.is_some_and(|specifier_rule| specifier_group == *gr && specifier_rule == *ru)
            }),
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

impl TryFrom<&str> for RuleSpecifier {
    type Error = String;

    fn try_from(specifier_str: &str) -> Result<Self, Self::Error> {
        let cat = specifier_str
            .parse::<&Category>()
            .map_err(|_| "Invalid rule.".to_string())?;

        Ok(RuleSpecifier::from(cat))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Suppression {
    pub(crate) suppression_range: TextRange,
    pub(crate) kind: SuppressionKind,
    pub(crate) rule_specifier: RuleSpecifier,
    #[allow(unused)]
    pub(crate) explanation: Option<String>,
}

impl Suppression {
    /// Creates a suppression from a suppression comment line.
    /// The line start must match `-- pgt-ignore`, otherwise, this will panic.
    /// Leading whitespace is ignored.
    pub(crate) fn from_line(line: &str, offset: &TextSize) -> Result<Self, SuppressionDiagnostic> {
        let start_trimmed = line.trim_ascii_start();
        let leading_whitespace_offset = line.len() - start_trimmed.len();
        let trimmed = start_trimmed.trim_ascii_end();

        assert!(
            start_trimmed.starts_with("-- pgt-ignore"),
            "Only try parsing suppressions from lines starting with `-- pgt-ignore`."
        );

        let full_offset = *offset + TextSize::new(leading_whitespace_offset.try_into().unwrap());
        let span = TextRange::new(
            full_offset,
            pgls_text_size::TextSize::new(trimmed.len().try_into().unwrap()) + full_offset,
        );

        let (line, explanation) = match trimmed.split_once(':') {
            Some((suppr, explanation)) => (suppr, Some(explanation.trim())),
            None => (trimmed, None),
        };

        let mut parts = line.split_ascii_whitespace();

        let _ = parts.next();
        let kind = match parts.next().unwrap() {
            "pgt-ignore-all" => SuppressionKind::File,
            "pgt-ignore-start" => SuppressionKind::Start,
            "pgt-ignore-end" => SuppressionKind::End,
            "pgt-ignore" => SuppressionKind::Line,
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

        let rule_specifier =
            RuleSpecifier::try_from(specifier_str).map_err(|e| SuppressionDiagnostic {
                span,
                message: MessageAndDescription::from(e),
            })?;

        Ok(Self {
            rule_specifier,
            kind,
            suppression_range: span,
            explanation: explanation.map(|e| e.to_string()),
        })
    }

    pub(crate) fn matches(&self, diagnostic_specifier: &RuleSpecifier) -> bool {
        let d_category = diagnostic_specifier.category();
        let d_group = diagnostic_specifier.group();
        let d_rule = diagnostic_specifier.rule();

        match &self.rule_specifier {
            // Check if we suppress the entire category
            RuleSpecifier::Category(cat) if cat == d_category => return true,

            // Check if we suppress the category & group
            RuleSpecifier::Group(cat, group) => {
                if cat == d_category && Some(group.as_str()) == d_group {
                    return true;
                }
            }

            // Check if we suppress the category & group & specific rule
            RuleSpecifier::Rule(cat, group, rule) => {
                if cat == d_category
                    && Some(group.as_str()) == d_group
                    && Some(rule.as_str()) == d_rule
                {
                    return true;
                }
            }

            _ => {}
        }

        false
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
        let line = "-- pgt-ignore lint/safety/banDropColumn: explanation";
        let offset = &TextSize::new(0);
        let suppression = Suppression::from_line(line, offset).unwrap();

        assert_eq!(suppression.kind, SuppressionKind::Line);
        assert_eq!(
            suppression.rule_specifier,
            RuleSpecifier::Rule(
                "lint".to_string(),
                "safety".to_string(),
                "banDropColumn".to_string()
            )
        );
        assert_eq!(suppression.explanation.as_deref(), Some("explanation"));
    }

    #[test]
    fn test_suppression_from_line_group() {
        let line = "-- pgt-ignore lint/safety: explanation";
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
        let line = "-- pgt-ignore-all lint/safety/banDropColumn: explanation";
        let offset = &TextSize::new(0);
        let suppression = Suppression::from_line(line, offset).unwrap();

        assert_eq!(suppression.kind, SuppressionKind::File);
        assert_eq!(
            suppression.rule_specifier,
            RuleSpecifier::Rule(
                "lint".to_string(),
                "safety".to_string(),
                "banDropColumn".to_string()
            )
        );
        assert_eq!(suppression.explanation.as_deref(), Some("explanation"));
    }

    #[test]
    fn test_suppression_from_line_start_kind() {
        let line = "-- pgt-ignore-start lint/safety/banDropColumn: explanation";
        let offset = &TextSize::new(0);
        let suppression = Suppression::from_line(line, offset).unwrap();

        assert_eq!(suppression.kind, SuppressionKind::Start);
        assert_eq!(
            suppression.rule_specifier,
            RuleSpecifier::Rule(
                "lint".to_string(),
                "safety".to_string(),
                "banDropColumn".to_string()
            )
        );
        assert_eq!(suppression.explanation.as_deref(), Some("explanation"));
    }

    #[test]
    fn test_suppression_from_line_end_kind() {
        let line = "-- pgt-ignore-end lint/safety/banDropColumn: explanation";
        let offset = &TextSize::new(0);
        let suppression = Suppression::from_line(line, offset).unwrap();

        assert_eq!(suppression.kind, SuppressionKind::End);
        assert_eq!(
            suppression.rule_specifier,
            RuleSpecifier::Rule(
                "lint".to_string(),
                "safety".to_string(),
                "banDropColumn".to_string()
            )
        );
        assert_eq!(suppression.explanation.as_deref(), Some("explanation"));
    }

    #[test]
    fn test_suppression_span_with_offset() {
        let line = "    \n-- pgt-ignore lint/safety/banDropColumn: explanation";
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
            "-- pgt-ignore-foo lint/safety/banDropColumn: explanation",
            "-- pgt-ignore foo lint/safety/banDropColumn: explanation",
            "-- pgt-ignore xyz lint/safety/banDropColumn: explanation",
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
            ("-- pgt-ignore lint", "lint/safety/banDropNotNull", true),
            ("-- pgt-ignore lint", "lint/safety/banDropColumn", true),
            // the group works for all rules in that group
            (
                "-- pgt-ignore lint/safety",
                "lint/safety/banDropColumn",
                true,
            ),
            ("-- pgt-ignore lint", "typecheck", false),
            ("-- pgt-ignore lint/safety", "typecheck", false),
            // a specific supppression only works for that same rule
            (
                "-- pgt-ignore lint/safety/banDropColumn",
                "lint/safety/banDropColumn",
                true,
            ),
            (
                "-- pgt-ignore lint/safety/banDropColumn",
                "lint/safety/banDropTable",
                false,
            ),
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
}
