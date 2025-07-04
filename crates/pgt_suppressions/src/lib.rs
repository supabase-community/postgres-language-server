use std::collections::HashMap;
pub mod parser;
pub mod suppression;

use pgt_analyse::RuleFilter;
use pgt_diagnostics::{Diagnostic, MessageAndDescription};

pub mod line_index;

use line_index::LineIndex;

use crate::{
    parser::SuppressionsParser,
    suppression::{RangeSuppression, RuleSpecifier, Suppression, SuppressionDiagnostic},
};

type Line = usize;

#[derive(Debug, Default, Clone)]
pub struct Suppressions {
    file_suppressions: Vec<Suppression>,
    line_suppressions: std::collections::HashMap<Line, Suppression>,
    range_suppressions: Vec<RangeSuppression>,
    pub diagnostics: Vec<SuppressionDiagnostic>,
    line_index: LineIndex,
}

impl From<&str> for Suppressions {
    fn from(doc: &str) -> Self {
        SuppressionsParser::parse(doc)
    }
}
impl From<String> for Suppressions {
    fn from(doc: String) -> Self {
        SuppressionsParser::parse(doc.as_str())
    }
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

#[cfg(test)]
mod tests {
    use pgt_diagnostics::{Diagnostic, MessageAndDescription};
    use pgt_text_size::TextRange;

    use crate::suppression::SuppressionDiagnostic;

    #[derive(Clone, Debug, Diagnostic)]
    #[diagnostic(category = "lint", severity = Error)]
    pub struct TestDiagnostic {
        #[location(span)]
        pub span: TextRange,
    }

    #[test]
    fn correctly_suppresses_diagnostics_at_top_level() {
        let doc = r#"
        -- pgt-ignore-all lint

        select 1;
        "#;

        let len_doc: u32 = doc.len().try_into().unwrap();

        let suppressions = super::Suppressions::from(doc);

        assert!(suppressions.is_suppressed(&TestDiagnostic {
            span: TextRange::new((len_doc - 10).into(), len_doc.into()),
        }));
    }

    #[test]
    fn correctly_suppresses_diagnostics_at_line() {
        let doc = r#"
            select 2;

            -- pgt-ignore lint
            select 1;
            "#;

        let suppressions = super::Suppressions::from(doc);

        assert!(suppressions.is_suppressed(&TestDiagnostic {
            span: TextRange::new(67.into(), 76.into()),
        }));
    }

    #[test]
    fn correctly_suppresses_with_multiple_line_diagnostics() {
        let doc = r#"
            select 2;

            -- pgt-ignore lint
            -- pgt-ignore action
            select 1;
            "#;

        let suppressions = super::Suppressions::from(doc);

        assert!(suppressions.is_suppressed(&TestDiagnostic {
            span: TextRange::new(100.into(), 109.into()),
        }));
    }

    #[test]
    fn correctly_suppresses_diagnostics_with_ranges() {
        let doc = r#"
            select 2;

            -- pgt-ignore-start lint
            select 1;
            -- pgt-ignore-end lint
            "#;

        let suppressions = super::Suppressions::from(doc);

        assert!(suppressions.is_suppressed(&TestDiagnostic {
            span: TextRange::new(73.into(), 82.into()),
        }));
    }

    #[test]
    fn marks_disabled_rule_suppressions_as_errors() {
        let doc = r#"
            select 2;

            -- pgt-ignore lint/safety/banDropTable
            select 1;
            "#;

        let suppressions = super::Suppressions::from(doc)
            .with_disabled_rules(&[pgt_analyse::RuleFilter::Group("safety")]);

        assert!(!suppressions.is_suppressed(&TestDiagnostic {
            span: TextRange::new(89.into(), 98.into()),
        }));

        assert_eq!(suppressions.diagnostics.len(), 1);

        assert_eq!(
            suppressions.diagnostics[0],
            SuppressionDiagnostic {
                span: TextRange::new(36.into(), 74.into()),
                message: MessageAndDescription::from("This rule has been disabled via the configuration. The suppression has no effect.".to_string())
            }
        );
    }

    #[test]
    fn marks_unused_suppressions_as_errors() {
        let doc = r#"
            select 2;

            -- pgt-ignore lint
            select 1;
            "#;

        // no diagnostics
        let diagnostics: Vec<TestDiagnostic> = vec![];

        let suppressions =
            super::Suppressions::from(doc).with_unused_suppressions_as_errors(&diagnostics);

        assert_eq!(suppressions.diagnostics.len(), 1);

        assert_eq!(
            suppressions.diagnostics[0],
            SuppressionDiagnostic {
                span: TextRange::new(36.into(), 54.into()),
                message: MessageAndDescription::from("This suppression has no effect.".to_string())
            }
        );
    }
}
