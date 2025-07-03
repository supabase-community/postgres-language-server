use std::{
    collections::HashMap,
    iter::{Enumerate, Peekable},
    str::Lines,
};

use pgt_analyse::{RuleCategory, RuleFilter};
use pgt_diagnostics::{Diagnostic, MessageAndDescription};
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

#[derive(Debug, Clone)]
pub enum SuppressionKind {
    File,
    Line,
    Start,
    End,
}

#[derive(Debug, PartialEq, Clone)]
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

    fn is_disabled(&self, disabled_rules: &[RuleFilter<'_>]) -> bool {
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
pub struct Suppression {
    suppression_range: TextRange,
    kind: SuppressionKind,
    rule_specifier: RuleSpecifier,
    #[allow(unused)]
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

    fn to_disabled_diagnostic(self) -> SuppressionDiagnostic {
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
pub struct RangeSuppression {
    suppressed_range: TextRange,
    start_suppression: Suppression,
}

type Line = usize;

#[derive(Debug, Default, Clone)]
pub struct Suppressions {
    file_suppressions: Vec<Suppression>,
    line_suppressions: std::collections::HashMap<Line, Suppression>,
    range_suppressions: Vec<RangeSuppression>,
    pub diagnostics: Vec<SuppressionDiagnostic>,
    line_index: LineIndex,
}

impl Suppressions {
    /// Some diagnostics can be turned off via the configuration.
    /// This will mark suppressions that try to suppress these disabled diagnostics as errors.
    #[must_use]
    pub fn with_disabled_rules(mut self, disabled_rules: &[RuleFilter<'_>]) -> Self {
        {
            let (enabled, disabled) = self
                .file_suppressions
                .into_iter()
                .partition(|s| !s.rule_specifier.is_disabled(disabled_rules));

            self.file_suppressions = enabled;

            for suppr in disabled {
                self.diagnostics.push(suppr.to_disabled_diagnostic());
            }
        }

        {
            let (enabled, disabled) = self
                .line_suppressions
                .into_iter()
                .partition(|(_, s)| !s.rule_specifier.is_disabled(disabled_rules));

            self.line_suppressions = enabled;

            for (_, suppr) in disabled {
                self.diagnostics.push(suppr.to_disabled_diagnostic());
            }
        }

        {
            let (enabled, disabled) = self.range_suppressions.into_iter().partition(|s| {
                !s.start_suppression
                    .rule_specifier
                    .is_disabled(disabled_rules)
            });

            self.range_suppressions = enabled;

            for range_suppr in disabled {
                self.diagnostics
                    .push(range_suppr.start_suppression.to_disabled_diagnostic());
            }
        }

        self
    }

    #[must_use]
    pub fn with_unused_suppressions_as_errors<D: Diagnostic>(mut self, diagnostics: &[D]) -> Self {
        let mut diagnostics_by_line: HashMap<usize, Vec<&D>> = HashMap::new();
        for diag in diagnostics {
            if let Some(line) = diag
                .location()
                .span
                .and_then(|sp| self.line_index.line_for_offset(sp.start()))
            {
                let entry = diagnostics_by_line.entry(line);
                entry
                    .and_modify(|current| {
                        current.push(diag);
                    })
                    .or_insert(vec![diag]);
            }
        }

        for (line, suppr) in &self.line_suppressions {
            let mut expected_diagnostic_line = line + 1;
            while self
                .line_suppressions
                .contains_key(&expected_diagnostic_line)
            {
                expected_diagnostic_line += 1;
            }

            if diagnostics_by_line
                .get(&expected_diagnostic_line)
                .is_some_and(|diags| {
                    diags.iter().any(|d| {
                        d.category()
                            .is_some_and(|cat| match RuleSpecifier::try_from(cat.name()) {
                                Ok(spec) => suppr.matches(&spec),
                                Err(_) => false,
                            })
                    })
                })
            {
                continue;
            } else {
                self.diagnostics.push(SuppressionDiagnostic {
                    span: suppr.suppression_range,
                    message: MessageAndDescription::from(
                        "This suppression has no effect.".to_string(),
                    ),
                })
            }
        }

        self
    }

    pub fn is_suppressed<D: Diagnostic>(&self, diagnostic: &D) -> bool {
        diagnostic
            .category()
            .map(|c| match RuleSpecifier::try_from(c.name()) {
                Ok(specifier) => {
                    self.by_file_suppression(&specifier)
                        || self.by_range_suppression(diagnostic, &specifier)
                        || self.by_line_suppression(diagnostic, &specifier)
                }
                Err(_) => false,
            })
            .unwrap_or(false)
    }

    fn by_file_suppression(&self, specifier: &RuleSpecifier) -> bool {
        self.file_suppressions.iter().any(|s| s.matches(specifier))
    }

    fn by_line_suppression<D: Diagnostic>(
        &self,
        diagnostic: &D,
        specifier: &RuleSpecifier,
    ) -> bool {
        self.get_eligible_line_suppressions_for_diagnostic(diagnostic)
            .iter()
            .any(|s| s.matches(specifier))
    }

    fn by_range_suppression<D: Diagnostic>(
        &self,
        diagnostic: &D,
        specifier: &RuleSpecifier,
    ) -> bool {
        self.range_suppressions.iter().any(|range_suppr| {
            range_suppr.start_suppression.matches(specifier)
                && diagnostic
                    .location()
                    .span
                    .is_some_and(|sp| range_suppr.suppressed_range.contains_range(sp))
        })
    }

    fn get_eligible_line_suppressions_for_diagnostic<D: Diagnostic>(
        &self,
        diagnostic: &D,
    ) -> Vec<&Suppression> {
        diagnostic
            .location()
            .span
            .and_then(|span| self.line_index.line_for_offset(span.start()))
            .filter(|line_no| *line_no > 0)
            .map(|mut line_no| {
                let mut eligible = vec![];

                // one-for-one, we're checking the lines above a diagnostic location
                // until there are no more diagnostics
                line_no -= 1;
                while let Some(suppr) = self.line_suppressions.get(&line_no) {
                    eligible.push(suppr);
                    line_no -= 1;
                }

                eligible
            })
            .unwrap_or_default()
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
