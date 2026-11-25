use pgls_analyse::{Applicability, RuleMeta};
use pgls_console::fmt::Display;
use pgls_console::{MarkupBuf, markup};
use pgls_diagnostics::advice::CodeSuggestionAdvice;
use pgls_diagnostics::{
    Advices, Category, Diagnostic, DiagnosticTags, Location, LogCategory, MessageAndDescription,
    Visit,
};
use pgls_text_edit::RangeEdit;
use pgls_text_size::{TextRange, TextSize};
use std::fmt::Debug;
use std::ops::Range;

use crate::AnalysableStatement;
use crate::linter_context::LinterRuleContext;

/// Trait implemented by all AST-based linter rules
pub trait LinterRule: RuleMeta + Sized {
    type Options: Default + Clone + Debug;

    /// Whether the rule reads [`LinterRuleContext::lookups`]. The lookups are collected by the
    /// name resolution of each statement, only when an enabled rule reads them.
    const READS_LOOKUPS: bool = false;

    /// Whether the rule reports an error that Postgres also raises when it prepares the
    /// statement, without a position. The database check doesn't report Postgres' error again
    /// for a statement the rule reported, see [`crate::StatementAnalysis`].
    const REPORTS_DATABASE_ERROR: bool = false;

    /// Execute the rule on the given AST context
    fn run(rule_context: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic>;
}

/// Diagnostic object returned by a single linter rule
#[derive(Clone, Debug, Diagnostic, PartialEq)]
pub struct LinterDiagnostic {
    #[category]
    pub(crate) category: &'static Category,
    #[location(span)]
    pub(crate) span: Option<TextRange>,
    #[message]
    #[description]
    pub(crate) message: MessageAndDescription,
    #[tags]
    pub(crate) tags: DiagnosticTags,
    #[advice]
    pub(crate) rule_advice: RuleAdvice,
    pub(crate) fix: Option<RuleFix>,
}

/// A change to the source that fixes the issue reported by a diagnostic.
///
/// The ranges of the edits are relative to the statement while the rule runs, and relative to
/// the file in the diagnostics returned by the analyser. A rule adds them in order with
/// [`Self::insert`], [`Self::replace`] and [`Self::delete`], and [`Self::apply`] checks that they
/// fit the text, in release builds too. The analyser doesn't check that the fix
/// keeps the SQL valid, which costs a parse; check it with [`Self::keeps_sql_valid`] before the
/// fix is offered or applied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleFix {
    /// What the fix does, e.g. "Add CONCURRENTLY".
    pub message: String,
    /// Set by the analyser from the fix kind of the rule, unless the rule marked the fix as
    /// unsafe.
    pub applicability: Applicability,
    /// Non-overlapping edits, ordered by their position.
    edits: Vec<RangeEdit>,
    /// The range of the statement in the file that the fix was computed on, set when the
    /// analyser moves the fix into the file.
    statement: TextRange,
}

impl RuleFix {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            applicability: Applicability::Always,
            edits: Vec::new(),
            statement: TextRange::default(),
        }
    }

    /// Marks the fix as unsafe, even if the rule's fixes are safe in general.
    pub fn unsafe_fix(mut self) -> Self {
        self.applicability = Applicability::MaybeIncorrect;
        self
    }

    /// Inserts `text` at `offset`.
    pub fn insert(self, offset: TextSize, text: impl Into<String>) -> Self {
        self.replace(TextRange::empty(offset), text)
    }

    /// Replaces the text in `range` with `text`.
    pub fn replace(mut self, range: TextRange, text: impl Into<String>) -> Self {
        self.edits.push(RangeEdit::new(range, text));
        self
    }

    /// Removes the text in `range`.
    pub fn delete(self, range: TextRange) -> Self {
        self.replace(range, "")
    }

    /// The edits of the fix, ordered by their position.
    pub fn edits(&self) -> &[RangeEdit] {
        &self.edits
    }

    /// Applies the fix to `text`. The edits must be relative to `text`. `None` if they are not
    /// sorted, overlap, or are not on char boundaries of `text`.
    pub fn apply(&self, text: &str) -> Option<String> {
        self.apply_at(text, TextSize::from(0))
    }

    /// Applies the fix to `text`, which starts at `start` in the text the edits are relative to.
    fn apply_at(&self, text: &str, start: TextSize) -> Option<String> {
        let applied = RangeEdit::apply_all(text, start, &self.edits);
        debug_assert!(
            applied.is_some(),
            "the edits of fix {:?} are not sorted, overlap, or are not on char boundaries of a text of length {} at {start:?}: {:?}",
            self.message,
            text.len(),
            self.edits
        );
        applied
    }

    /// Whether the SQL still parses with the fix applied. `text` is the text of the file that
    /// the analyser analysed, and `statements` are the ranges of its top-level statements, in
    /// file order. A fix within its statement, or at its end, is checked on the statement. A fix
    /// that also edits after the statement, e.g. to add a `COMMIT` after the last statement, is
    /// checked on the text from the start of the statement to the end of the last statement it
    /// edits. If that text doesn't parse without the fix, the fix can't be checked and is kept.
    ///
    /// Like the statements themselves, the SQL is parsed with its placeholders (psql variables
    /// such as `:name`, `$name`) converted to parameters, see
    /// [pgls_lexer::convert_to_positional_params].
    ///
    /// It parses once or twice, so check the fixes that are used, not all of them.
    pub fn keeps_sql_valid(&self, text: &str, statements: &[TextRange]) -> bool {
        let Some(edits_end) = self.edits.iter().map(|edit| edit.range.end()).max() else {
            return true;
        };
        let start = self.statement.start();
        if edits_end <= self.statement.end() {
            return text
                .get(Range::<usize>::from(self.statement))
                .and_then(|sql| self.apply_at(sql, start))
                .is_some_and(|sql| Self::parses(&sql));
        }
        let first = statements.partition_point(|statement| statement.start() < start);
        let end = statements
            .get(first..)
            .unwrap_or_default()
            .iter()
            .take_while(|statement| statement.start() < edits_end)
            .map(|statement| statement.end())
            .fold(edits_end, TextSize::max);
        let Some(text) = text.get(usize::from(start)..usize::from(end)) else {
            return true;
        };
        if !Self::parses(text) {
            return true;
        }
        self.apply_at(text, start)
            .is_some_and(|sql| Self::parses(&sql))
    }

    /// Whether the SQL parses with its placeholders converted to parameters. The conversion
    /// keeps the length of the text, so the fix edits apply to the source text unchanged.
    fn parses(sql: &str) -> bool {
        pgls_query::parse(&pgls_lexer::convert_to_positional_params(sql)).is_ok()
    }

    /// The range covered by all edits.
    pub fn range(&self) -> Option<TextRange> {
        let first = self.edits.first()?;
        let last = self.edits.last()?;
        Some(first.range.cover(last.range))
    }

    /// Moves the edits of a fix computed on the statement at `statement` into the file.
    pub(crate) fn move_into_file(&mut self, statement: TextRange) {
        for edit in &mut self.edits {
            edit.range += statement.start();
        }
        self.statement = statement;
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
/// It contains possible advices to show when printing a diagnostic that belong to the rule
pub struct RuleAdvice {
    pub(crate) details: Vec<Detail>,
    pub(crate) notes: Vec<(LogCategory, MarkupBuf)>,
    pub(crate) suggestion_list: Option<SuggestionList>,
    pub(crate) code_suggestion_list: Vec<CodeSuggestionAdvice<MarkupBuf>>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct SuggestionList {
    pub(crate) message: MarkupBuf,
    pub(crate) list: Vec<MarkupBuf>,
}

impl Advices for RuleAdvice {
    fn record(&self, visitor: &mut dyn Visit) -> std::io::Result<()> {
        for detail in &self.details {
            visitor.record_log(
                detail.log_category,
                &markup! { {detail.message} }.to_owned(),
            )?;
            visitor.record_frame(Location::builder().span(&detail.range).build())?;
        }
        // we then print notes
        for (log_category, note) in &self.notes {
            visitor.record_log(*log_category, &markup! { {note} }.to_owned())?;
        }

        if let Some(suggestion_list) = &self.suggestion_list {
            visitor.record_log(
                LogCategory::Info,
                &markup! { {suggestion_list.message} }.to_owned(),
            )?;
            let list: Vec<_> = suggestion_list
                .list
                .iter()
                .map(|suggestion| suggestion as &dyn Display)
                .collect();
            visitor.record_list(&list)?;
        }

        // finally, we print possible code suggestions on how to fix the issue
        for suggestion in &self.code_suggestion_list {
            suggestion.record(visitor)?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Detail {
    pub log_category: LogCategory,
    pub message: MarkupBuf,
    pub range: Option<TextRange>,
}

impl LinterDiagnostic {
    /// Creates a new [`LinterDiagnostic`] with a severity and title that will be
    /// used in a builder-like way to modify labels.
    pub fn new(category: &'static Category, span: Option<TextRange>, title: impl Display) -> Self {
        let message = markup!({ title }).to_owned();
        Self {
            category,
            span,
            message: MessageAndDescription::from(message),
            tags: DiagnosticTags::empty(),
            rule_advice: RuleAdvice::default(),
            fix: None,
        }
    }

    /// Attaches a fix to this [`LinterDiagnostic`]. The rule must declare a `fix_kind`.
    pub fn fix(mut self, fix: RuleFix) -> Self {
        self.fix = Some(fix);
        self
    }

    /// Attaches a fix if there is one.
    pub fn maybe_fix(mut self, fix: Option<RuleFix>) -> Self {
        self.fix = fix;
        self
    }

    /// The fix for the reported issue, if the rule can fix it.
    pub fn get_fix(&self) -> Option<&RuleFix> {
        self.fix.as_ref()
    }

    /// Set an explicit plain-text summary for this diagnostic.
    pub fn description(mut self, summary: impl Into<String>) -> Self {
        self.message.set_description(summary.into());
        self
    }

    /// Marks this diagnostic as deprecated code, which will
    /// be displayed in the language server.
    ///
    /// This does not have any influence on the diagnostic rendering.
    pub fn deprecated(mut self) -> Self {
        self.tags |= DiagnosticTags::DEPRECATED_CODE;
        self
    }

    /// Moves the spans of the diagnostic, which are relative to the statement while the rule
    /// runs, into the file. A diagnostic without a span points at the whole statement.
    pub(crate) fn move_into_file(&mut self, statement: &AnalysableStatement) {
        self.span = Some(match self.span {
            Some(span) => statement.document_range(span),
            None => statement.range,
        });
        for detail in &mut self.rule_advice.details {
            if let Some(range) = &mut detail.range {
                *range = statement.document_range(*range);
            }
        }
    }

    /// Sets the span of this diagnostic.
    pub fn span(mut self, span: TextRange) -> Self {
        self.span = Some(span);
        self
    }

    /// Marks this diagnostic as unnecessary code, which will
    /// be displayed in the language server.
    ///
    /// This does not have any influence on the diagnostic rendering.
    pub fn unnecessary(mut self) -> Self {
        self.tags |= DiagnosticTags::UNNECESSARY_CODE;
        self
    }

    /// Attaches a label to this [`LinterDiagnostic`].
    ///
    /// Like the span of the diagnostic, the given span is relative to the statement.
    pub fn label(mut self, span: Option<TextRange>, msg: impl Display) -> Self {
        self.rule_advice.details.push(Detail {
            log_category: LogCategory::Info,
            message: markup!({ msg }).to_owned(),
            range: span,
        });
        self
    }

    /// Attaches a detailed message to this [`LinterDiagnostic`].
    pub fn detail(self, span: Option<TextRange>, msg: impl Display) -> Self {
        self.label(span, msg)
    }

    /// Adds a footer to this [`LinterDiagnostic`], which will be displayed under the actual error.
    fn footer(mut self, log_category: LogCategory, msg: impl Display) -> Self {
        self.rule_advice
            .notes
            .push((log_category, markup!({ msg }).to_owned()));
        self
    }

    /// Adds a footer to this [`LinterDiagnostic`], with the `Info` log category.
    pub fn note(self, msg: impl Display) -> Self {
        self.footer(LogCategory::Info, msg)
    }

    /// It creates a new footer note which contains a message and a list of possible suggestions.
    /// Useful when there's need to suggest a list of things inside a diagnostic.
    pub fn footer_list(mut self, message: impl Display, list: &[impl Display]) -> Self {
        if !list.is_empty() {
            self.rule_advice.suggestion_list = Some(SuggestionList {
                message: markup! { {message} }.to_owned(),
                list: list
                    .iter()
                    .map(|msg| markup! { {msg} }.to_owned())
                    .collect(),
            });
        }

        self
    }

    /// Adds a footer to this [`LinterDiagnostic`], with the `Warn` severity.
    pub fn warning(self, msg: impl Display) -> Self {
        self.footer(LogCategory::Warn, msg)
    }

    pub fn advices(&self) -> &RuleAdvice {
        &self.rule_advice
    }

    /// Will return the rule's category name as defined via `define_categories! { .. }`.
    pub fn get_category_name(&self) -> &'static str {
        self.category.name()
    }
}
