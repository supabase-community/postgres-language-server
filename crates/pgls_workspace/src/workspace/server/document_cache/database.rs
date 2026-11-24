//! What the checks against the database found for each statement of a document.
//!
//! This is the interim step of the owner's decision OD5: the language server runs the database
//! checks when a file is opened or saved, and a pull while the user types reuses their results
//! for the statements whose text is unchanged. A run checks again only the statements whose
//! result can differ from the last run's, see [`DatabaseResults::still_valid`]. The statement
//! table (plan item P16) replaces this storage with a column of the statement.

use std::sync::{Arc, Weak};

use pgls_catalog::Snapshot;
use pgls_diagnostics::{DiagnosticExt, category, serde::Diagnostic as SDiagnostic};
use pgls_suppressions::PendingDiagnostic;
use pgls_text_size::TextRange;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{settings::PlPgSqlCheckSettings, workspace::StatementId};

/// What the checks against the database found for one statement.
#[derive(Default, Clone)]
pub(crate) struct StatementDatabaseResult {
    /// The diagnostics, with spans relative to the start of the statement.
    pub(crate) diagnostics: Vec<SDiagnostic>,
    /// Whether plpgsql_check checked the statement. It locates the syntax errors of a PL/pgSQL
    /// body, so the syntax error of libpg_query, which spans the whole body, isn't reported.
    pub(crate) plpgsql_checked: bool,
    /// What the statement was typechecked with, if it was.
    pub(crate) typecheck: Option<TypecheckInput>,
    /// Whether a check failed, e.g. because the connection broke. The next run checks the
    /// statement again.
    pub(crate) failed: bool,
}

impl StatementDatabaseResult {
    /// Whether the run neither found anything nor asked the database about the statement, so
    /// that checking it again costs nothing.
    fn is_empty(&self) -> bool {
        self.diagnostics.is_empty() && !self.plpgsql_checked && self.typecheck.is_none()
    }
}

/// What the typecheck of a statement depends on besides its text, the snapshot and the
/// settings: what the analysis of the statements before it found, see
/// [`pgls_analyser::StatementAnalysis`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TypecheckInput {
    /// The explicit search path of the session at the statement.
    pub(crate) search_path: Vec<String>,
    /// Whether a lint rule already reported the error of the database for the statement.
    pub(crate) duplicate_assignments_reported: bool,
}

/// What the results of a run depend on besides the statements: the snapshot of the database,
/// and the settings of the checks. The snapshot changes when the schema cache is invalidated or
/// the connection changes, and each connection has its own.
#[derive(Clone)]
pub(crate) struct DatabaseRun {
    /// Only for its identity: a `Weak` keeps the allocation, so that no other snapshot can have
    /// its address, but not the snapshot.
    snapshot: Weak<Snapshot>,
    typecheck: bool,
    plpgsql_check: PlPgSqlCheckSettings,
}

impl DatabaseRun {
    pub(crate) fn new(
        snapshot: &Arc<Snapshot>,
        typecheck: bool,
        plpgsql_check: PlPgSqlCheckSettings,
    ) -> Self {
        Self {
            snapshot: Arc::downgrade(snapshot),
            typecheck,
            plpgsql_check,
        }
    }
}

impl PartialEq for DatabaseRun {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.snapshot, &other.snapshot)
            && self.typecheck == other.typecheck
            && self.plpgsql_check == other.plpgsql_check
    }
}

/// What the last run of the database checks found for the top-level statements of a document,
/// by statement text: the results of the occurrences of the text, in document order. Statements
/// that the run neither found anything for nor asked the database about aren't kept, so it holds
/// at most one entry per statement of the document.
#[derive(Default)]
pub(crate) struct DatabaseResults {
    statements: FxHashMap<StatementId, Vec<StatementDatabaseResult>>,
    /// The statements that edits added since the last run, which it didn't check. At most the
    /// statements of the document: an edit drops the ones it removed.
    unchecked: FxHashSet<StatementId>,
    /// What the last run checked with. `None` before the first run.
    run: Option<DatabaseRun>,
}

/// What [`DatabaseResults::reuse`] finds for the statements of a document.
#[derive(Default)]
pub(crate) struct DatabaseDiagnostics {
    pub(crate) diagnostics: Vec<SDiagnostic>,
    /// The ranges of the statements that plpgsql_check checked.
    pub(crate) plpgsql_checked: Vec<TextRange>,
    /// The ranges of the statements that the last run didn't check, because an edit added them
    /// since. Their database diagnostics aren't known.
    pub(crate) unchecked: Vec<TextRange>,
}

impl DatabaseDiagnostics {
    /// The diagnostics that the database checks may report for the statements they haven't
    /// checked yet, so that a suppression of one isn't reported as unused before they do.
    pub(crate) fn pending(&self) -> Vec<PendingDiagnostic> {
        let categories = [category!("typecheck"), category!("plpgsql_check")];
        self.unchecked
            .iter()
            .flat_map(|span| {
                categories.map(|category| PendingDiagnostic {
                    span: *span,
                    category,
                })
            })
            .collect()
    }
}

impl DatabaseResults {
    /// The results of the top-level statements that a run with `run` checked, `(statement,
    /// range, result)` in document order. Each one has its own diagnostics, also when the
    /// document repeats a statement's text.
    pub(crate) fn new(
        run: DatabaseRun,
        checked: impl IntoIterator<Item = (StatementId, TextRange, StatementDatabaseResult)>,
    ) -> Self {
        let checked: Vec<_> = checked.into_iter().collect();
        // The texts with a result keep the results of all of their occurrences, so that
        // `reuse` finds an occurrence's result by its position.
        let with_results: FxHashSet<StatementId> = checked
            .iter()
            .filter(|(_, _, result)| !result.is_empty())
            .map(|(id, _, _)| id.clone())
            .collect();
        let mut statements: FxHashMap<StatementId, Vec<StatementDatabaseResult>> =
            FxHashMap::default();
        for (id, range, result) in checked {
            if !with_results.contains(&id) {
                continue;
            }
            debug_assert!(
                result.diagnostics.iter().all(|diagnostic| diagnostic
                    .span()
                    .is_none_or(|span| span.end() <= range.len())),
                "the spans are relative to the statement"
            );
            statements.entry(id).or_default().push(result);
        }
        Self {
            statements,
            unchecked: FxHashSet::default(),
            run: Some(run),
        }
    }

    /// The results of the last run that a run with `run` would find again for `statements`,
    /// `(text, typecheck input)` of the statements that a run checks in document order, as in
    /// [`Self::new`]. A statement has its last result if the last run checked with the same
    /// snapshot and settings, the statement's text occurs as often as then, its occurrence was
    /// typechecked with the same input, and no check of it failed. Such a statement needn't be
    /// checked again: its checks ask the database the same questions about the same schema.
    pub(crate) fn still_valid<'a>(
        &self,
        run: &DatabaseRun,
        statements: impl IntoIterator<Item = (&'a StatementId, Option<&'a TypecheckInput>)>,
    ) -> Vec<Option<StatementDatabaseResult>> {
        let statements = statements.into_iter();
        if self.run.as_ref() != Some(run) {
            return statements.map(|_| None).collect();
        }

        // The last results of each statement's text, and how often each text occurs now.
        let mut found: Vec<Option<(&StatementId, &[StatementDatabaseResult], usize)>> = Vec::new();
        let mut occurrences: FxHashMap<&StatementId, usize> = FxHashMap::default();
        let mut typecheck = Vec::new();
        for (text, input) in statements {
            typecheck.push(input);
            found.push(self.statements.get_key_value(text).map(|(text, results)| {
                let occurrence = occurrences.entry(text).or_default();
                let entry = (text, results.as_slice(), *occurrence);
                *occurrence += 1;
                entry
            }));
        }

        found
            .into_iter()
            .zip(typecheck)
            .map(|(found, input)| {
                let (text, results, occurrence) = found?;
                if occurrences.get(text) != Some(&results.len()) {
                    return None;
                }
                results
                    .get(occurrence)
                    .filter(|result| !result.failed && result.typecheck.as_ref() == input)
                    .cloned()
            })
            .collect()
    }

    /// The results for the top-level statements `statements`, `(text, range)` in document order,
    /// with the spans moved to where each statement is now. They must be the statements that a
    /// run checks, as in [`Self::new`], so that the occurrences of a text match. A text whose
    /// number of occurrences changed has none, since its occurrences can't be told apart.
    pub(crate) fn reuse<'a>(
        &self,
        statements: impl IntoIterator<Item = (&'a str, TextRange)>,
    ) -> DatabaseDiagnostics {
        let mut reused = DatabaseDiagnostics::default();
        if self.statements.is_empty() && self.unchecked.is_empty() {
            return reused;
        }

        // The statements with results, and how often each text occurs in the document.
        let mut found: Vec<(&StatementId, &[StatementDatabaseResult], usize, TextRange)> =
            Vec::new();
        let mut occurrences: FxHashMap<&StatementId, usize> = FxHashMap::default();
        for (text, range) in statements {
            let statement = StatementId::new(text);
            if self.unchecked.contains(&statement) {
                reused.unchecked.push(range);
            }
            if let Some((id, results)) = self.statements.get_key_value(&statement) {
                let occurrence = occurrences.entry(id).or_default();
                found.push((id, results.as_slice(), *occurrence, range));
                *occurrence += 1;
            }
        }

        for (id, results, occurrence, range) in found {
            if occurrences.get(id) != Some(&results.len()) {
                continue;
            }
            let Some(result) = results.get(occurrence) else {
                continue;
            };
            if result.plpgsql_checked {
                reused.plpgsql_checked.push(range);
            }
            reused
                .diagnostics
                .extend(result.diagnostics.iter().map(|diagnostic| {
                    let span = diagnostic.span().map(|span| span + range.start());
                    debug_assert!(span.is_none_or(|span| range.contains_range(span)));
                    SDiagnostic::new(diagnostic.clone().with_file_span(span))
                }));
        }
        reused
    }

    /// An edit added the statement with the text `text`, which the last run didn't check.
    pub(crate) fn edited(&mut self, text: StatementId) {
        self.unchecked.insert(text);
    }

    /// Drops the results of the statements with the text `text`, which the document no longer
    /// has.
    pub(crate) fn remove(&mut self, text: &StatementId) {
        self.statements.remove(text);
        self.unchecked.remove(text);
    }

    pub(crate) fn shrink(&mut self) {
        self.statements.shrink_to_fit();
        self.unchecked.shrink_to_fit();
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use pgls_diagnostics::{Diagnostic, DiagnosticExt, Error, serde::Diagnostic as SDiagnostic};
    use pgls_text_size::TextRange;

    use super::{DatabaseResults, DatabaseRun, StatementDatabaseResult, TypecheckInput};
    use crate::{settings::PlPgSqlCheckSettings, workspace::StatementId};

    #[derive(Debug, Diagnostic)]
    #[diagnostic(category = "plpgsql_check", message = "not found")]
    struct NotFound;

    impl StatementDatabaseResult {
        /// A statement that plpgsql_check checked, with a diagnostic at `span` of the statement.
        fn checked_with_diagnostic(span: TextRange) -> Self {
            Self {
                diagnostics: vec![SDiagnostic::new(Error::from(NotFound).with_file_span(span))],
                plpgsql_checked: true,
                ..Self::default()
            }
        }
    }

    impl DatabaseRun {
        /// A run with the default settings on a snapshot of its own.
        fn with_new_snapshot() -> Self {
            Self::new(&Arc::default(), true, PlPgSqlCheckSettings::default())
        }
    }

    /// The results move with their statement, and a statement whose text changed has none.
    #[test]
    fn reuses_the_results_of_unchanged_statements() {
        let results = DatabaseResults::new(
            DatabaseRun::with_new_snapshot(),
            [
                (
                    StatementId::new("select 1;"),
                    TextRange::new(0.into(), 9.into()),
                    StatementDatabaseResult::default(),
                ),
                (
                    StatementId::new("select a;"),
                    TextRange::new(10.into(), 19.into()),
                    StatementDatabaseResult::checked_with_diagnostic(TextRange::new(
                        7.into(),
                        8.into(),
                    )),
                ),
                (
                    StatementId::new("select b;"),
                    TextRange::new(20.into(), 29.into()),
                    StatementDatabaseResult::checked_with_diagnostic(TextRange::new(
                        7.into(),
                        8.into(),
                    )),
                ),
            ],
        );

        // A statement was added before them, and `select b;` changed.
        let reused = results.reuse([
            ("select 0;", TextRange::new(0.into(), 9.into())),
            ("select 1;", TextRange::new(10.into(), 19.into())),
            ("select a;", TextRange::new(20.into(), 29.into())),
            ("select c;", TextRange::new(30.into(), 39.into())),
        ]);

        assert_eq!(
            reused
                .diagnostics
                .iter()
                .map(SDiagnostic::span)
                .collect::<Vec<_>>(),
            [Some(TextRange::new(27.into(), 28.into()))]
        );
        assert_eq!(
            reused.plpgsql_checked,
            [TextRange::new(20.into(), 29.into())]
        );
    }

    /// Each occurrence of a repeated statement keeps its own results, as long as the number of
    /// occurrences is unchanged.
    #[test]
    fn reuses_the_results_of_each_occurrence() {
        let results = DatabaseResults::new(
            DatabaseRun::with_new_snapshot(),
            [
                (
                    StatementId::new("select a;"),
                    TextRange::new(0.into(), 9.into()),
                    StatementDatabaseResult::default(),
                ),
                (
                    StatementId::new("select a;"),
                    TextRange::new(10.into(), 19.into()),
                    StatementDatabaseResult::checked_with_diagnostic(TextRange::new(
                        7.into(),
                        8.into(),
                    )),
                ),
            ],
        );

        let moved = results.reuse([
            ("select a;", TextRange::new(5.into(), 14.into())),
            ("select a;", TextRange::new(20.into(), 29.into())),
        ]);
        assert_eq!(
            moved
                .diagnostics
                .iter()
                .map(SDiagnostic::span)
                .collect::<Vec<_>>(),
            [Some(TextRange::new(27.into(), 28.into()))]
        );

        let removed = results.reuse([("select a;", TextRange::new(0.into(), 9.into()))]);
        assert_eq!(
            removed
                .diagnostics
                .iter()
                .map(SDiagnostic::span)
                .collect::<Vec<_>>(),
            []
        );
        assert!(removed.plpgsql_checked.is_empty());
    }

    /// A statement that an edit added since the last run is unchecked until it's removed or the
    /// next run.
    #[test]
    fn reports_the_statements_edited_since_the_last_run() {
        let mut results = DatabaseResults::new(DatabaseRun::with_new_snapshot(), []);
        results.edited(StatementId::new("select 2;"));
        let statements = [
            ("select 1;", TextRange::new(0.into(), 9.into())),
            ("select 2;", TextRange::new(10.into(), 19.into())),
        ];

        let reused = results.reuse(statements);
        assert_eq!(reused.unchecked, [TextRange::new(10.into(), 19.into())]);
        assert_eq!(reused.pending().len(), 2);

        results.remove(&StatementId::new("select 2;"));
        assert!(results.reuse(statements).unchecked.is_empty());
    }

    /// A run checks again the statements whose result can differ from the last run's.
    #[test]
    fn keeps_the_results_that_are_still_valid() {
        let typecheck = |schema: &str| TypecheckInput {
            search_path: vec![schema.to_string()],
            duplicate_assignments_reported: false,
        };
        let typechecked = |schema: &str| StatementDatabaseResult {
            typecheck: Some(typecheck(schema)),
            ..StatementDatabaseResult::default()
        };
        let function = StatementId::new("create function f() ...;");
        let select = StatementId::new("select a;");
        let repeated = StatementId::new("select b;");
        let failed = StatementId::new("create function g() ...;");
        let unchecked = StatementId::new("select 1;");
        let run = DatabaseRun::with_new_snapshot();
        let range = TextRange::new(0.into(), 9.into());
        let results = DatabaseResults::new(
            run.clone(),
            [
                (
                    function.clone(),
                    range,
                    StatementDatabaseResult::checked_with_diagnostic(range),
                ),
                (select.clone(), range, typechecked("public")),
                (repeated.clone(), range, typechecked("public")),
                (
                    failed.clone(),
                    range,
                    StatementDatabaseResult {
                        plpgsql_checked: true,
                        failed: true,
                        ..StatementDatabaseResult::default()
                    },
                ),
                (unchecked.clone(), range, StatementDatabaseResult::default()),
            ],
        );
        let public = typecheck("public");
        let other = typecheck("other");
        let kept = |run: &DatabaseRun, statements: &[(&StatementId, Option<&TypecheckInput>)]| {
            results
                .still_valid(run, statements.iter().copied())
                .iter()
                .map(Option::is_some)
                .collect::<Vec<_>>()
        };

        assert_eq!(
            kept(
                &run,
                &[
                    (&function, None),
                    (&select, Some(&public)),
                    (&repeated, Some(&public)),
                    (&failed, None),
                    (&unchecked, None),
                ]
            ),
            [true, true, true, false, false]
        );
        // The search path of `select a;` changed, and `select b;` occurs twice now.
        assert_eq!(
            kept(
                &run,
                &[
                    (&function, None),
                    (&select, Some(&other)),
                    (&repeated, Some(&public)),
                    (&repeated, Some(&public)),
                ]
            ),
            [true, false, false, false]
        );
        // Another snapshot or other settings.
        assert_eq!(
            kept(&DatabaseRun::with_new_snapshot(), &[(&function, None)]),
            [false]
        );
        let mut settings = run.clone();
        settings.plpgsql_check.all_warnings = !settings.plpgsql_check.all_warnings;
        assert_eq!(kept(&settings, &[(&function, None)]), [false]);
        let mut settings = run.clone();
        settings.typecheck = false;
        assert_eq!(kept(&settings, &[(&function, None)]), [false]);
    }
}
