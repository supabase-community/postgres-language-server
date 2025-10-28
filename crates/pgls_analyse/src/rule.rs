use pgls_console::fmt::Display;
use pgls_console::{MarkupBuf, markup};
use pgls_diagnostics::advice::CodeSuggestionAdvice;
use pgls_diagnostics::{
    Advices, Category, Diagnostic, DiagnosticTags, Location, LogCategory, MessageAndDescription,
    Severity, Visit,
};
use pgls_text_size::TextRange;
use std::cmp::Ordering;
use std::fmt::Debug;

use crate::{categories::RuleCategory, context::RuleContext, registry::RegistryVisitor};

#[derive(Clone, Debug)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize),
    serde(rename_all = "camelCase")
)]
/// Static metadata containing information about a rule
pub struct RuleMetadata {
    /// It marks if a rule is deprecated, and if so a reason has to be provided.
    pub deprecated: Option<&'static str>,
    /// The version when the rule was implemented
    pub version: &'static str,
    /// The name of this rule, displayed in the diagnostics it emits
    pub name: &'static str,
    /// The content of the documentation comments for this rule
    pub docs: &'static str,
    /// Whether a rule is recommended or not
    pub recommended: bool,
    /// The source URL of the rule
    pub sources: &'static [RuleSource],
    /// The default severity of the rule
    pub severity: Severity,
}

impl RuleMetadata {
    pub const fn new(
        version: &'static str,
        name: &'static str,
        docs: &'static str,
        severity: Severity,
    ) -> Self {
        Self {
            deprecated: None,
            version,
            name,
            docs,
            sources: &[],
            recommended: false,
            severity,
        }
    }

    pub const fn recommended(mut self, recommended: bool) -> Self {
        self.recommended = recommended;
        self
    }

    pub const fn deprecated(mut self, deprecated: &'static str) -> Self {
        self.deprecated = Some(deprecated);
        self
    }

    pub const fn sources(mut self, sources: &'static [RuleSource]) -> Self {
        self.sources = sources;
        self
    }
}

pub trait RuleMeta {
    type Group: RuleGroup;
    const METADATA: RuleMetadata;
}

/// A rule group is a collection of rules under a given name, serving as a
/// "namespace" for lint rules and allowing the entire set of rules to be
/// disabled at once
pub trait RuleGroup {
    type Category: GroupCategory;
    /// The name of this group, displayed in the diagnostics emitted by its rules
    const NAME: &'static str;
    /// Register all the rules belonging to this group into `registry`
    fn record_rules<V: RegistryVisitor + ?Sized>(registry: &mut V);
}

/// A group category is a collection of rule groups under a given category ID,
/// serving as a broad classification on the kind of diagnostic or code action
/// these rule emit, and allowing whole categories of rules to be disabled at
/// once depending on the kind of analysis being performed
pub trait GroupCategory {
    /// The category ID used for all groups and rule belonging to this category
    const CATEGORY: RuleCategory;
    /// Register all the groups belonging to this category into `registry`
    fn record_groups<V: RegistryVisitor + ?Sized>(registry: &mut V);
}

/// Trait implemented by all analysis rules: declares interest to a certain AstNode type,
/// and a callback function to be executed on all nodes matching the query to possibly
/// raise an analysis event
pub trait Rule: RuleMeta + Sized {
    type Options: Default + Clone + Debug;

    /// `schema_cache` will only be available if the user has a working database connection.
    fn run(rule_context: &RuleContext<Self>) -> Vec<RuleDiagnostic>;
}

/// Diagnostic object returned by a single analysis rule
#[derive(Debug, Diagnostic, PartialEq)]
pub struct RuleDiagnostic {
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

impl RuleDiagnostic {
    /// Creates a new [`RuleDiagnostic`] with a severity and title that will be
    /// used in a builder-like way to modify labels.
    pub fn new(category: &'static Category, span: Option<TextRange>, title: impl Display) -> Self {
        let message = markup!({ title }).to_owned();
        Self {
            category,
            span,
            message: MessageAndDescription::from(message),
            tags: DiagnosticTags::empty(),
            rule_advice: RuleAdvice::default(),
        }
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

    /// Attaches a label to this [`RuleDiagnostic`].
    ///
    /// The given span has to be in the file that was provided while creating this [`RuleDiagnostic`].
    pub fn label(mut self, span: Option<TextRange>, msg: impl Display) -> Self {
        self.rule_advice.details.push(Detail {
            log_category: LogCategory::Info,
            message: markup!({ msg }).to_owned(),
            range: span,
        });
        self
    }

    /// Attaches a detailed message to this [`RuleDiagnostic`].
    pub fn detail(self, span: Option<TextRange>, msg: impl Display) -> Self {
        self.label(span, msg)
    }

    /// Adds a footer to this [`RuleDiagnostic`], which will be displayed under the actual error.
    fn footer(mut self, log_category: LogCategory, msg: impl Display) -> Self {
        self.rule_advice
            .notes
            .push((log_category, markup!({ msg }).to_owned()));
        self
    }

    /// Adds a footer to this [`RuleDiagnostic`], with the `Info` log category.
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

    /// Adds a footer to this [`RuleDiagnostic`], with the `Warn` severity.
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

#[derive(Debug, Clone, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum RuleSource {
    /// Rules from [Squawk](https://squawkhq.com)
    Squawk(&'static str),
    /// Rules from [Eugene](https://github.com/kaaveland/eugene)
    Eugene(&'static str),
}

impl PartialEq for RuleSource {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl std::fmt::Display for RuleSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Squawk(_) => write!(f, "Squawk"),
            Self::Eugene(_) => write!(f, "Eugene"),
        }
    }
}

impl PartialOrd for RuleSource {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RuleSource {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_rule = self.as_rule_name();
        let other_rule = other.as_rule_name();
        self_rule.cmp(other_rule)
    }
}

impl RuleSource {
    pub fn as_rule_name(&self) -> &'static str {
        match self {
            Self::Squawk(rule_name) => rule_name,
            Self::Eugene(rule_name) => rule_name,
        }
    }

    pub fn to_namespaced_rule_name(&self) -> String {
        match self {
            Self::Squawk(rule_name) => format!("squawk/{rule_name}"),
            Self::Eugene(rule_name) => format!("eugene/{rule_name}"),
        }
    }

    pub fn to_rule_url(&self) -> String {
        match self {
            Self::Squawk(rule_name) => format!("https://squawkhq.com/docs/{rule_name}"),
            Self::Eugene(rule_name) => {
                format!("https://kaveland.no/eugene/hints/{rule_name}/index.html")
            }
        }
    }

    pub fn as_url_and_rule_name(&self) -> (String, &'static str) {
        (self.to_rule_url(), self.as_rule_name())
    }
}
