use std::{
    iter::{Enumerate, Peekable},
    str::Lines,
};

use pgt_analyse::RuleCategory;
use pgt_diagnostics::{Diagnostic, Location, MessageAndDescription};
use pgt_text_size::{TextRange, TextSize};

pub mod line_index;

use line_index::LineIndex;

/// A specialized diagnostic for the typechecker.
///
/// Type diagnostics are always **errors**.
#[derive(Clone, Debug, Diagnostic)]
#[diagnostic(category = "lint", severity = Warning)]
pub struct SuppressionDiagnostic {
    #[location(span)]
    span: TextRange,
    #[description]
    #[message]
    message: MessageAndDescription,
}

#[derive(Debug)]
pub enum SuppressionKind {
    File,
    Line,
    Start,
    End,
}

#[derive(Debug, PartialEq)]
enum RuleSpecifier {
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

#[derive(Debug)]
pub struct Suppression {
    suppression_range: TextRange,
    /// `None` means that all categories are suppressed
    kind: SuppressionKind,
    rule_specifier: RuleSpecifier,
    explanation: Option<String>,
}

impl Suppression {
    fn from_line(line: &str, offset: &TextSize) -> Result<Self, SuppressionDiagnostic> {
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
                    message: MessageAndDescription::from(format!(
                        "You must specify which lints to suppress."
                    )),
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

    fn matches(&self, diagnostic_specifier: &RuleSpecifier) -> bool {
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
}

#[derive(Debug)]
pub struct RangeSuppression {
    suppressed_range: TextRange,
    start_suppression: Suppression,
}

type Line = usize;

#[derive(Debug, Default)]
pub struct Suppressions {
    file_suppressions: Vec<Suppression>,
    line_suppressions: std::collections::HashMap<Line, Suppression>,
    range_suppressions: Vec<RangeSuppression>,
    pub diagnostics: Vec<SuppressionDiagnostic>,
    line_index: LineIndex,
}

impl Suppressions {
    pub fn is_suppressed<D: Diagnostic>(&self, diagnostic: &D) -> bool {
        let location = diagnostic.location();

        diagnostic
            .category()
            .map(|c| match RuleSpecifier::try_from(c.name()) {
                Ok(specifier) => {
                    self.by_file_suppression(&specifier)
                        || self.by_range_suppression(location, &specifier)
                        || self.by_line_suppression(location, &specifier)
                }
                Err(_) => false,
            })
            .unwrap_or(false)
    }

    fn by_file_suppression(&self, specifier: &RuleSpecifier) -> bool {
        self.file_suppressions.iter().any(|s| s.matches(specifier))
    }

    fn by_line_suppression(&self, location: Location<'_>, specifier: &RuleSpecifier) -> bool {
        location
            .span
            .and_then(|span| self.line_index.line_for_offset(span.start()))
            .filter(|line_no| *line_no > 0)
            .is_some_and(|mut line_no| {
                let mut eligible_suppressions = vec![];

                // one-for-one, we're checking the lines above a diagnostic location
                // until there are no more diagnostics
                line_no -= 1;
                while let Some(suppr) = self.line_suppressions.get(&line_no) {
                    eligible_suppressions.push(suppr);
                    line_no -= 1;
                }

                eligible_suppressions.iter().any(|s| s.matches(specifier))
            })
    }

    fn by_range_suppression(&self, location: Location<'_>, specifier: &RuleSpecifier) -> bool {
        self.range_suppressions.iter().any(|range_suppr| {
            range_suppr.start_suppression.matches(specifier)
                && location
                    .span
                    .is_some_and(|sp| range_suppr.suppressed_range.contains_range(sp))
        })
    }
}

#[derive(Debug)]
pub struct SuppressionsParser<'a> {
    file_suppressions: Vec<Suppression>,
    line_suppressions: std::collections::HashMap<Line, Suppression>,
    range_suppressions: Vec<RangeSuppression>,
    diagnostics: Vec<SuppressionDiagnostic>,
    lines: Peekable<Enumerate<Lines<'a>>>,
    line_index: LineIndex,

    start_suppressions_stack: Vec<Suppression>,
}

impl<'a> SuppressionsParser<'a> {
    pub fn new(doc: &'a str) -> Self {
        let lines = doc.lines().enumerate().peekable();

        Self {
            file_suppressions: vec![],
            line_suppressions: std::collections::HashMap::default(),
            range_suppressions: vec![],
            diagnostics: vec![],
            lines,
            line_index: LineIndex::new(doc),
            start_suppressions_stack: vec![],
        }
    }

    pub fn parse(doc: &str) -> Suppressions {
        let mut parser = SuppressionsParser::new(doc);

        parser.parse_file_suppressions();
        parser.parse_suppressions();
        parser.handle_unmatched_start_suppressions();

        Suppressions {
            file_suppressions: parser.file_suppressions,
            line_suppressions: parser.line_suppressions,
            range_suppressions: parser.range_suppressions,
            diagnostics: parser.diagnostics,
            line_index: LineIndex::new(doc),
        }
    }

    /// Will parse the suppressions at the start of the file.
    /// As soon as anything is encountered that's not a `pgt-ignore-all`
    /// suppression, this will stop.
    fn parse_file_suppressions(&mut self) {
        while let Some((_, preview)) = self.lines.peek() {
            if !preview.trim().starts_with("-- pgt-ignore-all") {
                return;
            }

            let (idx, line) = self.lines.next().unwrap();

            let offset = self.line_index.offset_for_line(idx).unwrap();

            match Suppression::from_line(line, offset) {
                Ok(suppr) => self.file_suppressions.push(suppr),
                Err(diag) => self.diagnostics.push(diag),
            }
        }
    }

    fn parse_suppressions(&mut self) {
        while let Some((idx, line)) = self.lines.next() {
            if !line.trim().starts_with("-- pgt-ignore") {
                continue;
            }

            let offset = self.line_index.offset_for_line(idx).unwrap();

            let suppr = match Suppression::from_line(line, offset) {
                Ok(suppr) => suppr,
                Err(diag) => {
                    self.diagnostics.push(diag);
                    continue;
                }
            };

            match suppr.kind {
                SuppressionKind::File => {
                    self.diagnostics.push(SuppressionDiagnostic {
                        span: suppr.suppression_range,
                        message: MessageAndDescription::from(format!(
                            "File suppressions should be at the top of the file."
                        )),
                    });
                }

                SuppressionKind::Line => {
                    self.line_suppressions.insert(idx, suppr);
                }

                SuppressionKind::Start => self.start_suppressions_stack.push(suppr),
                SuppressionKind::End => {
                    let matching_start_idx = self
                        .start_suppressions_stack
                        .iter_mut()
                        .rev()
                        .position(|start| start.rule_specifier == suppr.rule_specifier);

                    if let Some(start_idx) = matching_start_idx {
                        let start = self.start_suppressions_stack.remove(start_idx);

                        let full_range = TextRange::new(
                            start.suppression_range.start(),
                            suppr.suppression_range.end(),
                        );

                        self.range_suppressions.push(RangeSuppression {
                            suppressed_range: full_range,
                            start_suppression: start,
                        });
                    } else {
                        self.diagnostics.push(SuppressionDiagnostic {
                            span: suppr.suppression_range,
                            message: MessageAndDescription::from(format!(
                                "This end suppression does not have a matching start."
                            )),
                        });
                    }
                }
            }
        }
    }

    /// If we have `pgt-ignore-start` suppressions without matching end tags after parsing the entire file,
    /// we'll report diagnostics for those.
    fn handle_unmatched_start_suppressions(&mut self) {
        let start_suppressions = std::mem::replace(&mut self.start_suppressions_stack, vec![]);

        for suppr in start_suppressions {
            self.diagnostics.push(SuppressionDiagnostic {
                span: suppr.suppression_range,
                message: MessageAndDescription::from(format!(
                    "This start suppression does not have a matching end."
                )),
            });
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     fn parse(doc: &str) -> Suppressions {
//         SuppressionsParser::parse(doc)
//     }

//     #[test]
//     fn test_ignore_with_extra_colons_in_explanation() {
//         let doc = "// pgt-ignore lint/safety: reason: with: colons";
//         let sups = parse(doc);
//         let suppr = sups.line_suppressions.values().next().unwrap();
//         assert_eq!(suppr.explanation, Some("reason: with: colons"));
//     }

//     #[test]
//     fn test_ignore_with_trailing_whitespace() {
//         let doc = "// pgt-ignore lint/safety   ";
//         let sups = parse(doc);
//         assert_eq!(sups.line_suppressions.len(), 1);
//         assert!(sups.diagnostics.is_empty());
//     }

//     #[test]
//     fn test_ignore_with_leading_whitespace() {
//         let doc = "   // pgt-ignore lint/safety";
//         let sups = parse(doc);
//         assert_eq!(sups.line_suppressions.len(), 1);
//         assert!(sups.diagnostics.is_empty());
//     }

//     #[test]
//     fn test_multiple_unmatched_ends() {
//         let doc = r#"
//                     // pgt-ignore-end lint/safety
//                     // pgt-ignore-end lint/performance
//                 "#;
//         let sups = parse(doc);
//         assert_eq!(sups.diagnostics.len(), 2);
//         for diag in sups.diagnostics {
//             assert!(
//                 diag.message
//                     .to_string()
//                     .contains("does not have a matching start")
//             );
//         }
//     }

//     #[test]
//     fn test_multiple_unmatched_starts() {
//         let doc = r#"
//                     // pgt-ignore-start lint/safety
//                     // pgt-ignore-start lint/performance
//                 "#;
//         let sups = parse(doc);
//         assert_eq!(sups.diagnostics.len(), 2);
//         for diag in sups.diagnostics {
//             assert!(
//                 diag.message
//                     .to_string()
//                     .contains("does not have a matching end")
//             );
//         }
//     }

//     #[test]
//     fn test_ignore_with_invalid_tag_and_valid_tag() {
//         let doc = r#"
//                     // pgt-ignore-foo lint/safety
//                     // pgt-ignore lint/safety
//                 "#;
//         let sups = parse(doc);
//         assert_eq!(sups.diagnostics.len(), 1);
//         assert_eq!(sups.line_suppressions.len(), 1);
//     }

//     #[test]
//     fn test_ignore_with_missing_category_and_valid_tag() {
//         let doc = r#"
//                     // pgt-ignore
//                     // pgt-ignore lint/safety
//                 "#;
//         let sups = parse(doc);
//         assert_eq!(sups.diagnostics.len(), 1);
//         assert_eq!(sups.line_suppressions.len(), 1);
//     }

//     #[test]
//     fn test_ignore_with_group_and_rule_and_explanation() {
//         let doc = "// pgt-ignore lint/safety/banDropColumn: explanation";
//         let sups = parse(doc);
//         let suppr = sups.line_suppressions.values().next().unwrap();
//         assert_eq!(suppr.explanation, Some("explanation"));
//         match suppr.rule_filter {
//             Some(RuleFilter::Rule(group, rule)) => {
//                 assert_eq!(group, "safety");
//                 assert_eq!(rule, "banDropColumn");
//             }
//             _ => panic!("Expected RuleFilter::Rule"),
//         }
//     }

//     #[test]
//     fn test_ignore_with_group_only_and_explanation() {
//         let doc = "// pgt-ignore lint/safety: explanation";
//         let sups = parse(doc);
//         let suppr = sups.line_suppressions.values().next().unwrap();
//         assert_eq!(suppr.explanation, Some("explanation"));
//         match suppr.rule_filter {
//             Some(RuleFilter::Group(group)) => {
//                 assert_eq!(group, "safety");
//             }
//             _ => panic!("Expected RuleFilter::Group"),
//         }
//     }

//     #[test]
//     fn test_ignore_with_no_group_or_rule_and_explanation() {
//         let doc = "// pgt-ignore lint: explanation";
//         let sups = parse(doc);
//         let suppr = sups.line_suppressions.values().next().unwrap();
//         assert_eq!(suppr.explanation, Some("explanation"));
//         assert!(suppr.rule_filter.is_none());
//     }

//     #[test]
//     fn test_ignore_with_empty_explanation() {
//         let doc = "// pgt-ignore lint/safety:";
//         let sups = parse(doc);
//         let suppr = sups.line_suppressions.values().next().unwrap();
//         assert_eq!(suppr.explanation, Some(""));
//     }

//     #[test]
//     fn test_ignore_with_multiple_colons_and_spaces() {
//         let doc = "// pgt-ignore lint/safety:   explanation: with spaces  ";
//         let sups = parse(doc);
//         let suppr = sups.line_suppressions.values().next().unwrap();
//         assert_eq!(suppr.explanation, Some("explanation: with spaces"));
//     }

//     #[test]
//     fn test_ignore_with_invalid_category() {
//         let doc = "// pgt-ignore foo/safety";
//         let sups = parse(doc);
//         assert_eq!(sups.line_suppressions.len(), 0);
//         assert_eq!(sups.diagnostics.len(), 1);
//         let diag = &sups.diagnostics[0];
//         assert_eq!(diag.message.to_string(), "Invalid Rule Category: foo");
//     }

//     #[test]
//     fn test_ignore_with_missing_specifier() {
//         let doc = "// pgt-ignore";
//         let sups = parse(doc);
//         assert_eq!(sups.line_suppressions.len(), 0);
//         assert_eq!(sups.diagnostics.len(), 1);
//         let diag = &sups.diagnostics[0];
//         assert!(
//             diag.message
//                 .to_string()
//                 .contains("must specify which lints to suppress")
//                 || diag.message.to_string().contains("must specify")
//         );
//     }

//     #[test]
//     fn test_range_suppression_basic() {
//         let doc = r#"
//             // pgt-ignore-start lint/safety/banDropColumn: start explanation
//             SELECT * FROM foo;
//             // pgt-ignore-end lint/safety/banDropColumn: end explanation
//         "#;
//         let sups = parse(doc);
//         // Should have one range suppression
//         assert_eq!(sups.range_suppressions.len(), 1);
//         let range = &sups.range_suppressions[0];
//         assert_eq!(range.rule_category, RuleCategory::Lint);
//         assert_eq!(
//             range.rule_filter,
//             Some(RuleFilter::Rule("safety", "banDropColumn"))
//         );
//         assert_eq!(range.explanation, Some("start explanation"));
//         // The start and end suppressions should be present and correct
//         assert_eq!(
//             range.start_suppression.explanation,
//             Some("start explanation")
//         );
//         assert_eq!(range.end_suppression.explanation, Some("end explanation"));
//     }
// }
