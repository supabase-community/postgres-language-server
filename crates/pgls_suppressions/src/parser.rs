use std::{
    iter::{Enumerate, Peekable},
    str::Lines,
};

use pgls_diagnostics::MessageAndDescription;
use pgls_text_size::TextRange;

use crate::{
    Suppressions,
    line_index::LineIndex,
    suppression::{RangeSuppression, Suppression, SuppressionDiagnostic, SuppressionKind},
};

#[derive(Debug)]
pub(crate) struct SuppressionsParser<'a> {
    file_suppressions: Vec<Suppression>,
    line_suppressions: std::collections::HashMap<usize, Suppression>,
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
            line_index: parser.line_index,
        }
    }

    /// Will parse the suppressions at the start of the file.
    /// As soon as anything is encountered that's not a `pgt-ignore-all`
    /// suppression or an empty line, this will stop.
    fn parse_file_suppressions(&mut self) {
        while let Some((_, preview)) = self.lines.peek() {
            if preview.trim().is_empty() {
                self.lines.next();
                continue;
            }

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
        for (idx, line) in self.lines.by_ref() {
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
                        message: MessageAndDescription::from(
                            "File suppressions should be at the top of the file.".to_string(),
                        ),
                    });
                }

                SuppressionKind::Line => {
                    self.line_suppressions.insert(idx, suppr);
                }

                SuppressionKind::Start => self.start_suppressions_stack.push(suppr),
                SuppressionKind::End => {
                    let matching_start_idx = self
                        .start_suppressions_stack
                        .iter()
                        .enumerate()
                        .filter_map(|(idx, s)| {
                            if s.rule_specifier == suppr.rule_specifier {
                                Some(idx)
                            } else {
                                None
                            }
                        })
                        .next_back();

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
                            message: MessageAndDescription::from(
                                "This end suppression does not have a matching start.".to_string(),
                            ),
                        });
                    }
                }
            }
        }
    }

    /// If we have `pgt-ignore-start` suppressions without matching end tags after parsing the entire file,
    /// we'll report diagnostics for those.
    fn handle_unmatched_start_suppressions(&mut self) {
        let start_suppressions = std::mem::take(&mut self.start_suppressions_stack);

        for suppr in start_suppressions {
            self.diagnostics.push(SuppressionDiagnostic {
                span: suppr.suppression_range,
                message: MessageAndDescription::from(
                    "This start suppression does not have a matching end.".to_string(),
                ),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::suppression::{RuleSpecifier, SuppressionKind};

    #[test]
    fn test_parse_line_suppressions() {
        let doc = r#"
SELECT 1;
-- pgt-ignore lint/safety/banDropColumn
SELECT 2;
"#;
        let suppressions = SuppressionsParser::parse(doc);

        // Should have a line suppression on line 1 (0-based index)
        let suppression = suppressions
            .line_suppressions
            .get(&2)
            .expect("no suppression found");

        assert_eq!(suppression.kind, SuppressionKind::Line);
        assert_eq!(
            suppression.rule_specifier,
            RuleSpecifier::Rule(
                "lint".to_string(),
                "safety".to_string(),
                "banDropColumn".to_string()
            )
        );
    }

    #[test]
    fn test_parse_multiple_line_suppressions() {
        let doc = r#"
SELECT 1;
-- pgt-ignore lint/safety/banDropColumn
-- pgt-ignore lint/safety/banDropTable
-- pgt-ignore lint/safety/banDropNotNull
"#;

        let suppressions = SuppressionsParser::parse(doc);

        assert_eq!(suppressions.line_suppressions.len(), 3);

        assert_eq!(
            suppressions
                .line_suppressions
                .get(&2)
                .unwrap()
                .rule_specifier
                .rule(),
            Some("banDropColumn")
        );

        assert_eq!(
            suppressions
                .line_suppressions
                .get(&3)
                .unwrap()
                .rule_specifier
                .rule(),
            Some("banDropTable")
        );

        assert_eq!(
            suppressions
                .line_suppressions
                .get(&4)
                .unwrap()
                .rule_specifier
                .rule(),
            Some("banDropNotNull")
        );
    }

    #[test]
    fn parses_file_level_suppressions() {
        let doc = r#"
-- pgt-ignore-all lint
-- pgt-ignore-all typecheck

SELECT 1;
-- pgt-ignore-all lint/safety
"#;

        let suppressions = SuppressionsParser::parse(doc);

        assert_eq!(suppressions.diagnostics.len(), 1);
        assert_eq!(suppressions.file_suppressions.len(), 2);

        assert_eq!(
            suppressions.file_suppressions[0].rule_specifier,
            RuleSpecifier::Category("lint".to_string())
        );
        assert_eq!(
            suppressions.file_suppressions[1].rule_specifier,
            RuleSpecifier::Category("typecheck".to_string())
        );

        assert_eq!(
            suppressions.diagnostics[0].message.to_string(),
            String::from("File suppressions should be at the top of the file.")
        );
    }

    #[test]
    fn parses_range_suppressions() {
        let doc = r#"
-- pgt-ignore-start lint/safety/banDropTable
drop table users;
drop table auth;
drop table posts;
-- pgt-ignore-end lint/safety/banDropTable
"#;

        let suppressions = SuppressionsParser::parse(doc);

        assert_eq!(suppressions.range_suppressions.len(), 1);

        assert_eq!(
            suppressions.range_suppressions[0],
            RangeSuppression {
                suppressed_range: TextRange::new(1.into(), 141.into()),
                start_suppression: Suppression {
                    kind: SuppressionKind::Start,
                    rule_specifier: RuleSpecifier::Rule(
                        "lint".to_string(),
                        "safety".to_string(),
                        "banDropTable".to_string()
                    ),
                    suppression_range: TextRange::new(1.into(), 45.into()),
                    explanation: None,
                },
            }
        );
    }

    #[test]
    fn parses_range_suppressions_with_errors() {
        let doc = r#"
-- pgt-ignore-start lint/safety/banDropTable
drop table users;
-- pgt-ignore-start lint/safety/banDropTable
drop table auth;
drop table posts;
-- pgt-ignore-end lint/safety/banDropTable
-- pgt-ignore-end lint/safety/banDropColumn
"#;

        let suppressions = SuppressionsParser::parse(doc);

        assert_eq!(suppressions.range_suppressions.len(), 1);
        assert_eq!(suppressions.diagnostics.len(), 2);

        // the inner, nested start/end combination is recognized.
        assert_eq!(
            suppressions.range_suppressions[0],
            RangeSuppression {
                suppressed_range: TextRange::new(64.into(), 186.into()),
                start_suppression: Suppression {
                    kind: SuppressionKind::Start,
                    rule_specifier: RuleSpecifier::Rule(
                        "lint".to_string(),
                        "safety".to_string(),
                        "banDropTable".to_string()
                    ),
                    suppression_range: TextRange::new(64.into(), 108.into()),
                    explanation: None,
                },
            }
        );

        // the outer end is an error
        assert_eq!(
            suppressions.diagnostics[0].message.to_string(),
            String::from("This end suppression does not have a matching start.")
        );

        // the outer start is an error
        assert_eq!(
            suppressions.diagnostics[1].message.to_string(),
            String::from("This start suppression does not have a matching end.")
        );
    }
}
