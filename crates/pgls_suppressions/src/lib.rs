use std::collections::HashMap;
pub mod parser;
pub mod suppression;

use pgls_analyse::RuleFilter;
use pgls_diagnostics::{Diagnostic, MessageAndDescription};

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
    pub fn get_disabled_diagnostic_suppressions_as_errors(
        &self,
        disabled_rules: &[RuleFilter<'_>],
    ) -> Vec<SuppressionDiagnostic> {
        let mut diagnostics = vec![];

        {
            let disabled = self
                .file_suppressions
                .iter()
                .filter(|s| s.rule_specifier.is_disabled(disabled_rules));

            for suppr in disabled {
                diagnostics.push(suppr.to_disabled_diagnostic());
            }
        }

        {
            let disabled = self
                .line_suppressions
                .iter()
                .filter(|(_, s)| s.rule_specifier.is_disabled(disabled_rules));

            for (_, suppr) in disabled {
                diagnostics.push(suppr.to_disabled_diagnostic());
            }
        }

        {
            let disabled = self.range_suppressions.iter().filter(|s| {
                s.start_suppression
                    .rule_specifier
                    .is_disabled(disabled_rules)
            });

            for range_suppr in disabled {
                diagnostics.push(range_suppr.start_suppression.to_disabled_diagnostic());
            }
        }

        diagnostics
    }

    pub fn get_unused_suppressions_as_errors<D: Diagnostic>(
        &self,
        diagnostics: &[D],
    ) -> Vec<SuppressionDiagnostic> {
        let mut results = vec![];

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

        // Users may use many suppressions for a single diagnostic, like so:
        // ```
        // -- pgt-ignore lint/safety/banDropTable
        // -- pgt-ignore lint/safety/banDropColumn
        // <statement causing two diagnostics>
        // ```
        // So to find a matching diagnostic for any suppression, we're moving
        // down lines until we find a line where there's no suppression.
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
                results.push(SuppressionDiagnostic {
                    span: suppr.suppression_range,
                    message: MessageAndDescription::from(
                        "This suppression has no effect.".to_string(),
                    ),
                })
            }
        }

        results
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
                // until there are no more suppressions
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
    use pgls_diagnostics::{Diagnostic, MessageAndDescription};
    use pgls_text_size::TextRange;

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
            -- pgt-ignore syntax
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

        let suppressions = super::Suppressions::from(doc);

        let disabled_diagnostics = suppressions.get_disabled_diagnostic_suppressions_as_errors(&[
            pgls_analyse::RuleFilter::Group("safety"),
        ]);

        assert_eq!(disabled_diagnostics.len(), 1);

        assert_eq!(
            disabled_diagnostics[0],
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

        let suppressions = super::Suppressions::from(doc);

        let unused_diagnostics = suppressions.get_unused_suppressions_as_errors(&diagnostics);

        assert_eq!(unused_diagnostics.len(), 1);

        assert_eq!(
            unused_diagnostics[0],
            SuppressionDiagnostic {
                span: TextRange::new(36.into(), 54.into()),
                message: MessageAndDescription::from("This suppression has no effect.".to_string())
            }
        );
    }
}
