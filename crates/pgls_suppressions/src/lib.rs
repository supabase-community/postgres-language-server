pub mod parser;
pub mod suppression;

use std::ops::Range;

use pgls_analyse::RuleFilter;
use pgls_diagnostics::{Category, Diagnostic, Location, MessageAndDescription};
use pgls_text_size::{TextRange, TextSize};

use crate::{
    parser::SuppressionsParser,
    suppression::{RangeSuppression, RuleSpecifier, Suppression, SuppressionDiagnostic},
};

/// A diagnostic that a check may still report, e.g. a check against the database of a statement
/// that changed since it last ran. A suppression it would match isn't reported as unused, since
/// whether it is used isn't known yet.
#[derive(Debug, Clone, Copy)]
pub struct PendingDiagnostic {
    pub span: TextRange,
    pub category: &'static Category,
}

impl Diagnostic for PendingDiagnostic {
    fn category(&self) -> Option<&'static Category> {
        Some(self.category)
    }

    fn location(&self) -> Location<'_> {
        Location::builder().span(&self.span).build()
    }
}

/// The suppressions of a file.
///
/// A diagnostic only reaches the suppressions near it: the line suppressions are sorted by the
/// line they target, and are found by binary search on the diagnostic's span. The category of a
/// diagnostic is only parsed when a suppression is near it.
#[derive(Debug, Default, Clone)]
pub struct Suppressions {
    file_suppressions: Vec<Suppression>,
    /// In file order, so also sorted by [`LineSuppression::target`].
    line_suppressions: Vec<LineSuppression>,
    range_suppressions: Vec<RangeSuppression>,
    pub diagnostics: Vec<SuppressionDiagnostic>,
    /// The ranges of the top-level statements, in file order.
    statements: Vec<TextRange>,
}

#[derive(Debug, Clone)]
pub(crate) struct LineSuppression {
    pub(crate) suppression: Suppression,
    /// The offsets of the line the suppression targets: the next line that isn't a line
    /// suppression, up to the start of the line after it. The last line of the file ends at
    /// `u32::MAX`, so that it holds the offsets past the end of the file, as in
    /// [`pgls_line_index::LineIndex::line_col`]. Empty at `u32::MAX` if the file has no such line.
    pub(crate) target: TextRange,
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
    /// Sets the ranges of the statements of the file. A line suppression above the first line of
    /// a statement then applies to the whole statement, also to diagnostics on its later lines.
    pub fn with_statements(mut self, statements: Vec<TextRange>) -> Self {
        self.statements = statements;
        self
    }

    fn is_empty(&self) -> bool {
        self.file_suppressions.is_empty()
            && self.line_suppressions.is_empty()
            && self.range_suppressions.is_empty()
    }

    /// Some diagnostics can be turned off via the configuration.
    /// This will mark suppressions that try to suppress these disabled diagnostics as errors.
    pub fn get_disabled_diagnostic_suppressions_as_errors(
        &self,
        disabled_rules: &[RuleFilter<'_>],
    ) -> Vec<SuppressionDiagnostic> {
        if disabled_rules.is_empty() {
            return Vec::new();
        }

        self.file_suppressions
            .iter()
            .chain(self.line_suppressions.iter().map(|l| &l.suppression))
            .chain(self.range_suppressions.iter().map(|r| &r.start_suppression))
            .filter(|s| s.rule_specifier.is_disabled(disabled_rules))
            .map(Suppression::to_disabled_diagnostic)
            .collect()
    }

    /// The line suppressions that no diagnostic matches, in file order.
    ///
    /// One pass over the diagnostics marks the line suppressions that match each: O(D log L) for
    /// D diagnostics and L line suppressions, plus the suppressions within each diagnostic's lines.
    ///
    /// A suppression that one of `pending` would match isn't unused yet.
    pub fn get_unused_suppressions_as_errors<D: Diagnostic>(
        &self,
        diagnostics: &[D],
        pending: &[PendingDiagnostic],
    ) -> Vec<SuppressionDiagnostic> {
        // Users may use many suppressions for a single diagnostic, like so:
        // ```
        // -- pgt-ignore banDropTable
        // -- pgt-ignore banDropColumn
        // <statement causing two diagnostics>
        // ```
        // All of them target the first line that isn't a suppression.
        if self.line_suppressions.is_empty() {
            return Vec::new();
        }

        let mut used = vec![false; self.line_suppressions.len()];
        for diagnostic in diagnostics {
            self.mark_line_suppressions_used(&mut used, diagnostic);
        }
        for diagnostic in pending {
            self.mark_line_suppressions_used(&mut used, diagnostic);
        }

        self.line_suppressions
            .iter()
            .zip(used)
            .filter(|(_, used)| !used)
            .map(|(line, _)| SuppressionDiagnostic {
                span: line.suppression.suppression_range,
                message: MessageAndDescription::from("This suppression has no effect.".to_string()),
            })
            .collect()
    }

    /// Marks in `used` the line suppressions that match `diagnostic`.
    fn mark_line_suppressions_used<D: Diagnostic>(&self, used: &mut [bool], diagnostic: &D) {
        let Some(span) = diagnostic.location().span else {
            return;
        };
        let mut specifier = None;
        for index in self.line_suppressions_for(span) {
            let (Some(used), Some(line)) = (used.get_mut(index), self.line_suppressions.get(index))
            else {
                continue;
            };
            if *used {
                continue;
            }
            let Some(specifier) = specifier.get_or_insert_with(|| RuleSpecifier::of(diagnostic))
            else {
                break;
            };
            *used = line.suppression.matches(specifier);
        }
    }

    /// O(log L) for L line suppressions, plus the suppressions near the diagnostic and the range
    /// suppressions. Without any suppression, it returns at once.
    pub fn is_suppressed<D: Diagnostic>(&self, diagnostic: &D) -> bool {
        if self.is_empty() {
            return false;
        }

        let span = diagnostic.location().span;
        let mut candidates = self
            .file_suppressions
            .iter()
            .chain(span.into_iter().flat_map(|span| {
                self.range_suppressions
                    .iter()
                    .filter(move |r| r.suppressed_range.contains_range(span))
                    .map(|r| &r.start_suppression)
            }))
            .chain(span.into_iter().flat_map(|span| {
                self.line_suppressions_for(span)
                    .filter_map(|index| self.line_suppressions.get(index))
                    .map(|line| &line.suppression)
            }))
            .peekable();
        if candidates.peek().is_none() {
            return false;
        }

        RuleSpecifier::of(diagnostic)
            .is_some_and(|specifier| candidates.any(|s| s.matches(&specifier)))
    }

    /// The indices of the line suppressions that target a line of `span`, or the first line of
    /// the statement that `span` starts in. These are at most two runs of the sorted
    /// suppressions; the second may repeat indices of the first.
    fn line_suppressions_for(&self, span: TextRange) -> impl Iterator<Item = usize> {
        let start = span.start();
        let end = span
            .end()
            .checked_sub(TextSize::new(1))
            .unwrap_or_else(|| span.end());
        let on_lines = self.line_suppressions_targeting(start, end);
        let on_statement = self
            .statement_start(start)
            .map(|statement_start| {
                self.line_suppressions_targeting(statement_start, statement_start)
            })
            .unwrap_or_default();
        on_lines.chain(on_statement)
    }

    /// The indices of the line suppressions whose target line holds an offset in
    /// `start..=end`. Empty if `end < start`.
    fn line_suppressions_targeting(&self, start: TextSize, end: TextSize) -> Range<usize> {
        let first = self
            .line_suppressions
            .partition_point(|line| line.target.end() <= start);
        let count = self
            .line_suppressions
            .get(first..)
            .unwrap_or_default()
            .partition_point(|line| line.target.start() <= end);
        first..first + count
    }

    /// The start of the statement that contains `offset`.
    fn statement_start(&self, offset: TextSize) -> Option<TextSize> {
        let index = self
            .statements
            .partition_point(|statement| statement.start() <= offset)
            .checked_sub(1)?;
        let statement = self.statements.get(index)?;
        statement
            .contains_inclusive(offset)
            .then_some(statement.start())
    }
}

#[cfg(test)]
mod tests {
    use pgls_diagnostics::{Diagnostic, MessageAndDescription};
    use pgls_text_size::{TextRange, TextSize};

    use crate::suppression::SuppressionDiagnostic;

    #[derive(Clone, Debug, Diagnostic)]
    #[diagnostic(category = "lint", severity = Error)]
    pub(crate) struct TestDiagnostic {
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
    fn a_lone_carriage_return_does_not_end_a_line() {
        // Lines end at `\n` only, like in the editor positions and `str::lines`.
        let doc = "select '\r';\n-- pgls-ignore lint\nselect 1;";
        let start = u32::try_from(doc.find("select 1").unwrap()).unwrap();
        let diagnostic = TestDiagnostic {
            span: TextRange::new(start.into(), u32::try_from(doc.len()).unwrap().into()),
        };

        let suppressions = super::Suppressions::from(doc);

        assert!(suppressions.diagnostics.is_empty());
        assert!(suppressions.is_suppressed(&diagnostic));
        let line = &suppressions.line_suppressions[0];
        assert_eq!(
            &doc[line.suppression.suppression_range],
            "-- pgls-ignore lint"
        );
        assert_eq!(line.target, TextRange::new(start.into(), u32::MAX.into()));
    }

    #[test]
    fn correctly_suppresses_diagnostics_when_target_line_is_inside_multiline_span() {
        let doc = "alter table users\n-- pgt-ignore lint\ndrop column deprecated_field;";

        let suppressions = super::Suppressions::from(doc);

        let diagnostic = TestDiagnostic {
            span: TextRange::new(0.into(), u32::try_from(doc.len()).unwrap().into()),
        };

        assert!(suppressions.is_suppressed(&diagnostic));
        assert!(
            suppressions
                .get_unused_suppressions_as_errors(&[diagnostic], &[])
                .is_empty()
        );
    }

    #[test]
    fn line_suppression_above_a_statement_applies_to_all_of_its_lines() {
        let doc = "-- pgt-ignore lint\ncreate table users (\n    id bigint,\n    name text\n);\nselect 1;";
        let next = TextRange::new(72.into(), 81.into());
        let suppressions = super::Suppressions::from(doc)
            .with_statements(vec![TextRange::new(19.into(), 71.into()), next]);

        let on_a_later_line = TestDiagnostic {
            span: TextRange::new(59.into(), 63.into()),
        };
        assert_eq!(&doc[on_a_later_line.span], "name");
        assert!(suppressions.is_suppressed(&on_a_later_line));
        assert!(
            suppressions
                .get_unused_suppressions_as_errors(&[on_a_later_line], &[])
                .is_empty()
        );

        // The next statement isn't suppressed, and without the statements, only the first line is.
        assert!(!suppressions.is_suppressed(&TestDiagnostic { span: next }));
        assert!(
            !super::Suppressions::from(doc).is_suppressed(&TestDiagnostic {
                span: TextRange::new(59.into(), 63.into()),
            })
        );
    }

    #[test]
    fn many_line_suppressions_find_their_statements() {
        // A suppression above every other statement, the first of each pair with one that matches
        // nothing above it; one diagnostic per statement, on its last line.
        let mut doc = String::new();
        let mut statements = Vec::new();
        for i in 0..100 {
            if i % 2 == 0 {
                doc.push_str("-- pgls-ignore typecheck\n-- pgls-ignore lint\n");
            }
            let start = u32::try_from(doc.len()).unwrap();
            doc.push_str(&format!("select\n  {i};\n"));
            let end = u32::try_from(doc.len() - 1).unwrap();
            statements.push(TextRange::new(start.into(), end.into()));
        }
        let suppressions =
            super::Suppressions::from(doc.as_str()).with_statements(statements.clone());
        let diagnostics: Vec<_> = statements
            .iter()
            .map(|statement| TestDiagnostic {
                span: TextRange::new(statement.start() + TextSize::new(9), statement.end()),
            })
            .collect();

        for (i, diagnostic) in diagnostics.iter().enumerate() {
            assert_eq!(suppressions.is_suppressed(diagnostic), i % 2 == 0, "{i}");
        }
        let unused = suppressions.get_unused_suppressions_as_errors(&diagnostics, &[]);
        assert_eq!(unused.len(), 50);
        assert!(
            unused
                .iter()
                .all(|error| &doc[error.span] == "-- pgls-ignore typecheck")
        );
        assert!(unused.is_sorted_by_key(|error| error.span.start()));
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

            -- pgt-ignore banDropTable
            select 1;
            "#;

        let suppressions = super::Suppressions::from(doc);

        let disabled_diagnostics = suppressions.get_disabled_diagnostic_suppressions_as_errors(&[
            pgls_analyse::RuleFilter::Group("destructive"),
        ]);

        assert_eq!(disabled_diagnostics.len(), 1);

        assert_eq!(
            disabled_diagnostics[0],
            SuppressionDiagnostic {
                span: TextRange::new(36.into(), 62.into()),
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

        let unused_diagnostics = suppressions.get_unused_suppressions_as_errors(&diagnostics, &[]);

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
