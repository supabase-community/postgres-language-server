use std::{borrow, collections::BTreeSet};

use crate::{
    AnalyserOptions,
    analysed_file_context::AnalysedFileContext,
    context::RuleContext,
    filter::{AnalysisFilter, GroupKey, RuleKey},
    rule::{GroupCategory, Rule, RuleDiagnostic, RuleGroup},
};

pub trait RegistryVisitor {
    /// Record the category `C` to this visitor
    fn record_category<C: GroupCategory>(&mut self) {
        C::record_groups(self);
    }

    /// Record the group `G` to this visitor
    fn record_group<G: RuleGroup>(&mut self) {
        G::record_rules(self);
    }

    /// Record the rule `R` to this visitor
    fn record_rule<R>(&mut self)
    where
        R: Rule + 'static;
}

/// Key struct for a rule in the metadata map, sorted alphabetically
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MetadataKey {
    inner: (&'static str, &'static str),
}

impl MetadataKey {
    fn into_group_key(self) -> GroupKey {
        let (group, _) = self.inner;
        GroupKey::new(group)
    }

    fn into_rule_key(self) -> RuleKey {
        let (group, rule) = self.inner;
        RuleKey::new(group, rule)
    }
}

impl<'a> borrow::Borrow<(&'a str, &'a str)> for MetadataKey {
    fn borrow(&self) -> &(&'a str, &'a str) {
        &self.inner
    }
}

impl borrow::Borrow<str> for MetadataKey {
    fn borrow(&self) -> &str {
        self.inner.0
    }
}

/// Stores metadata information for all the rules in the registry, sorted
/// alphabetically
#[derive(Debug, Default)]
pub struct MetadataRegistry {
    inner: BTreeSet<MetadataKey>,
}

impl MetadataRegistry {
    /// Return a unique identifier for a rule group if it's known by this registry
    pub fn find_group(&self, group: &str) -> Option<GroupKey> {
        let key = self.inner.get(group)?;
        Some(key.into_group_key())
    }

    /// Return a unique identifier for a rule if it's known by this registry
    pub fn find_rule(&self, group: &str, rule: &str) -> Option<RuleKey> {
        let key = self.inner.get(&(group, rule))?;
        Some(key.into_rule_key())
    }

    pub(crate) fn insert_rule(&mut self, group: &'static str, rule: &'static str) {
        self.inner.insert(MetadataKey {
            inner: (group, rule),
        });
    }
}

impl RegistryVisitor for MetadataRegistry {
    fn record_rule<R>(&mut self)
    where
        R: Rule + 'static,
    {
        self.insert_rule(<R::Group as RuleGroup>::NAME, R::METADATA.name);
    }
}

pub struct RuleRegistryBuilder<'a> {
    filter: &'a AnalysisFilter<'a>,
    // Rule Registry
    registry: RuleRegistry,
}

impl RegistryVisitor for RuleRegistryBuilder<'_> {
    fn record_category<C: GroupCategory>(&mut self) {
        if self.filter.match_category::<C>() {
            C::record_groups(self);
        }
    }

    fn record_group<G: RuleGroup>(&mut self) {
        if self.filter.match_group::<G>() {
            G::record_rules(self);
        }
    }

    /// Add the rule `R` to the list of rules stored in this registry instance
    fn record_rule<R>(&mut self)
    where
        R: Rule<Options: Default> + 'static,
    {
        if !self.filter.match_rule::<R>() {
            return;
        }

        let rule = RegistryRule::new::<R>();

        self.registry.rules.push(rule);
    }
}

/// The rule registry holds type-erased instances of all active analysis rules
pub struct RuleRegistry {
    pub rules: Vec<RegistryRule>,
}

impl IntoIterator for RuleRegistry {
    type Item = RegistryRule;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.rules.into_iter()
    }
}

/// Internal representation of a single rule in the registry
#[derive(Copy, Clone)]
pub struct RegistryRule {
    pub run: RuleExecutor,
}

impl RuleRegistry {
    pub fn builder<'a>(filter: &'a AnalysisFilter<'a>) -> RuleRegistryBuilder<'a> {
        RuleRegistryBuilder {
            filter,
            registry: RuleRegistry {
                rules: Default::default(),
            },
        }
    }
}

pub struct RegistryRuleParams<'a> {
    pub root: &'a pgls_query::NodeEnum,
    pub options: &'a AnalyserOptions,
    pub analysed_file_context: &'a AnalysedFileContext<'a>,
    pub schema_cache: Option<&'a pgls_schema_cache::SchemaCache>,
}

/// Executor for rule as a generic function pointer
type RuleExecutor = fn(&RegistryRuleParams) -> Vec<RuleDiagnostic>;

impl RegistryRule {
    fn new<R>() -> Self
    where
        R: Rule<Options: Default> + 'static,
    {
        /// Generic implementation of RuleExecutor for any rule type R
        fn run<R>(params: &RegistryRuleParams) -> Vec<RuleDiagnostic>
        where
            R: Rule<Options: Default> + 'static,
        {
            let options = params.options.rule_options::<R>().unwrap_or_default();

            let ctx = RuleContext::new(
                params.root,
                &options,
                params.schema_cache,
                params.analysed_file_context,
            );

            R::run(&ctx)
        }

        Self { run: run::<R> }
    }
}

impl RuleRegistryBuilder<'_> {
    pub fn build(self) -> RuleRegistry {
        self.registry
    }
}
