use pgt_analyse::{RuleCategory, RuleFilter};
use pgt_diagnostics::{Diagnostic, MessageAndDescription};
use pgt_text_size::{TextRange, TextSize};

/// A specialized diagnostic for the typechecker.
///
/// Type diagnostics are always **errors**.
#[derive(Clone, Debug, Diagnostic)]
#[diagnostic(category = "lint", severity = Warning)]
pub struct SuppressionDiagnostic {
    #[location(span)]
    pub span: TextRange,
    #[description]
    #[message]
    pub message: MessageAndDescription,
}

#[derive(Debug, Clone)]
pub(crate) enum SuppressionKind {
    File,
    Line,
    Start,
    End,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum RuleSpecifier {
    Category(RuleCategory),
    Group(RuleCategory, String),
    Rule(RuleCategory, String, String),
}

impl RuleSpecifier {
    fn category(&self) -> &RuleCategory {
        match self {
            RuleSpecifier::Category(rule_category) => rule_category,
            RuleSpecifier::Group(rule_category, _) => rule_category,
            RuleSpecifier::Rule(rule_category, _, _) => rule_category,
        }
    }

    fn group(&self) -> Option<&str> {
        match self {
            RuleSpecifier::Category(_) => None,
            RuleSpecifier::Group(_, gr) => Some(gr),
            RuleSpecifier::Rule(_, gr, _) => Some(gr),
        }
    }

    fn rule(&self) -> Option<&str> {
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

impl TryFrom<&str> for RuleSpecifier {
    type Error = String;

    fn try_from(specifier_str: &str) -> Result<Self, Self::Error> {
        let mut specifiers = specifier_str.split('/').map(|s| s.to_string());

        let rule_category: RuleCategory = specifiers.next().unwrap().try_into()?;

        let group = specifiers.next();
        let rule = specifiers.next();

        match (group, rule) {
            (Some(g), Some(r)) => Ok(RuleSpecifier::Rule(rule_category, g, r)),
            (Some(g), None) => Ok(RuleSpecifier::Group(rule_category, g)),
            (None, None) => Ok(RuleSpecifier::Category(rule_category)),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Suppression {
    pub(crate) suppression_range: TextRange,
    pub(crate) kind: SuppressionKind,
    pub(crate) rule_specifier: RuleSpecifier,
    #[allow(unused)]
    explanation: Option<String>,
}

impl Suppression {
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
            pgt_text_size::TextSize::new(trimmed.len().try_into().unwrap()) + full_offset,
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
                        "'{}' is not a valid suppression tag.",
                        k,
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

    pub(crate) fn to_disabled_diagnostic(self) -> SuppressionDiagnostic {
        SuppressionDiagnostic {
            span: self.suppression_range,
            message: MessageAndDescription::from(
                "This rule has been disabled via the configuration. The suppression has no effect."
                    .to_string(),
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RangeSuppression {
    pub(crate) suppressed_range: TextRange,
    pub(crate) start_suppression: Suppression,
}
