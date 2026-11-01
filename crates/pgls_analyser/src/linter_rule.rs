use pgls_analyse::{Applicability, RuleMeta};
use pgls_console::fmt::Display;
use pgls_console::{MarkupBuf, markup};
use pgls_diagnostics::advice::CodeSuggestionAdvice;
use pgls_diagnostics::{
    Advices, Category, Diagnostic, DiagnosticTags, Location, LogCategory, MessageAndDescription,
    Visit,
};
use pgls_text_size::{TextRange, TextSize};
use std::fmt::Debug;

use crate::linter_context::LinterRuleContext;

/// Trait implemented by all AST-based linter rules
pub trait LinterRule: RuleMeta + Sized {
    type Options: Default + Clone + Debug;

    /// Execute the rule on the given AST context
    /// `snapshot` will only be available if the user has a working database connection.
    fn run(rule_context: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic>;
}

/// Diagnostic object returned by a single linter rule
#[derive(Debug, Diagnostic, PartialEq)]
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
/// the file in the diagnostics returned by the analyser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleFix {
    /// What the fix does, e.g. "Add CONCURRENTLY".
    pub message: String,
    /// Set by the analyser from the fix kind of the rule, unless the rule marked the fix as
    /// unsafe.
    pub applicability: Applicability,
    /// Non-overlapping edits, ordered by their position.
    pub edits: Vec<FixEdit>,
}

/// Replaces the text in `range` with `replacement`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixEdit {
    pub range: TextRange,
    pub replacement: String,
}

impl RuleFix {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            applicability: Applicability::Always,
            edits: Vec::new(),
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
        self.edits.push(FixEdit {
            range,
            replacement: text.into(),
        });
        self
    }

    /// Removes the text in `range`.
    pub fn delete(self, range: TextRange) -> Self {
        self.replace(range, "")
    }

    /// Applies the fix to `text`. The edits must be relative to `text`.
    pub fn apply(&self, text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        let mut last = 0;
        for edit in &self.edits {
            let start = usize::from(edit.range.start());
            result.push_str(&text[last..start]);
            result.push_str(&edit.replacement);
            last = usize::from(edit.range.end());
        }
        result.push_str(&text[last..]);
        result
    }

    /// The range covered by all edits.
    pub fn range(&self) -> Option<TextRange> {
        let first = self.edits.first()?;
        let last = self.edits.last()?;
        Some(first.range.cover(last.range))
    }

    pub(crate) fn shift(&mut self, offset: TextSize) {
        for edit in &mut self.edits {
            edit.range += offset;
        }
    }
}

#[derive(Debug, Default, PartialEq)]
/// It contains possible advices to show when printing a diagnostic that belong to the rule
pub struct RuleAdvice {
    pub(crate) details: Vec<Detail>,
    pub(crate) notes: Vec<(LogCategory, MarkupBuf)>,
    pub(crate) suggestion_list: Option<SuggestionList>,
    pub(crate) code_suggestion_list: Vec<CodeSuggestionAdvice<MarkupBuf>>,
}

#[derive(Debug, Default, PartialEq)]
pub struct SuggestionList {
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

#[derive(Debug, PartialEq)]
pub struct Detail {
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
    pub(crate) fn move_into_file(&mut self, statement: TextRange) {
        self.span = Some(match self.span {
            Some(span) => span + statement.start(),
            None => statement,
        });
        for detail in &mut self.rule_advice.details {
            if let Some(range) = &mut detail.range {
                *range += statement.start();
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
