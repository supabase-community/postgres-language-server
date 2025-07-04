use std::{
    iter::{Enumerate, Peekable},
    str::Lines,
};

use pgt_diagnostics::MessageAndDescription;
use pgt_text_size::TextRange;

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
