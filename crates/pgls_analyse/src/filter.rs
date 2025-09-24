use std::fmt::{Debug, Display, Formatter};

use crate::{
    categories::RuleCategories,
    rule::{GroupCategory, Rule, RuleGroup},
};

/// Allow filtering a single rule or group of rules by their names
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuleFilter<'a> {
    Group(&'a str),
    Rule(&'a str, &'a str),
}

/// Allows filtering the list of rules that will be executed in a run of the analyser,
/// and at what source code range signals (diagnostics or actions) may be raised
#[derive(Debug, Default, Clone, Copy)]
pub struct AnalysisFilter<'a> {
    /// Only allow rules with these categories to emit signals
    pub categories: RuleCategories,
    /// Only allow rules matching these names to emit signals
    /// If `enabled_rules` is set to `None`, then all rules are enabled.
    pub enabled_rules: Option<&'a [RuleFilter<'a>]>,
    /// Do not allow rules matching these names to emit signals
    pub disabled_rules: &'a [RuleFilter<'a>],
}

impl<'analysis> AnalysisFilter<'analysis> {
    /// It creates a new filter with the set of [enabled rules](RuleFilter) passed as argument
    pub fn from_enabled_rules(enabled_rules: &'analysis [RuleFilter<'analysis>]) -> Self {
        Self {
            enabled_rules: Some(enabled_rules),
            ..AnalysisFilter::default()
        }
    }

    /// Return `true` if the category `C` matches this filter
    pub fn match_category<C: GroupCategory>(&self) -> bool {
        self.categories.contains(C::CATEGORY)
    }

    /// Return `true` if the group `G` matches this filter
    pub fn match_group<G: RuleGroup>(&self) -> bool {
        self.match_category::<G::Category>()
            && self.enabled_rules.is_none_or(|enabled_rules| {
                enabled_rules.iter().any(|filter| filter.match_group::<G>())
            })
            && !self
                .disabled_rules
                .iter()
                .any(|filter| matches!(filter, RuleFilter::Group(_)) && filter.match_group::<G>())
    }

    /// Return `true` if the rule `R` matches this filter
    pub fn match_rule<R: Rule>(&self) -> bool {
        self.match_category::<<R::Group as RuleGroup>::Category>()
            && self.enabled_rules.is_none_or(|enabled_rules| {
                enabled_rules.iter().any(|filter| filter.match_rule::<R>())
            })
            && !self
                .disabled_rules
                .iter()
                .any(|filter| filter.match_rule::<R>())
    }
}

impl<'a> RuleFilter<'a> {
    // Returns the group name of this filter.
    pub fn group(self) -> &'a str {
        match self {
            RuleFilter::Group(group) => group,
            RuleFilter::Rule(group, _) => group,
        }
    }
    /// Return `true` if the group `G` matches this filter
    pub fn match_group<G: RuleGroup>(self) -> bool {
        match self {
            RuleFilter::Group(group) => group == G::NAME,
            RuleFilter::Rule(group, _) => group == G::NAME,
        }
    }

    /// Return `true` if the rule `R` matches this filter
    pub fn match_rule<R>(self) -> bool
    where
        R: Rule,
    {
        match self {
            RuleFilter::Group(group) => group == <R::Group as RuleGroup>::NAME,
            RuleFilter::Rule(group, rule) => {
                group == <R::Group as RuleGroup>::NAME && rule == R::METADATA.name
            }
        }
    }
}

impl Debug for RuleFilter<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for RuleFilter<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RuleFilter::Group(group) => {
                write!(f, "{group}")
            }
            RuleFilter::Rule(group, rule) => {
                write!(f, "{group}/{rule}")
            }
        }
    }
}

impl pgls_console::fmt::Display for RuleFilter<'_> {
    fn fmt(&self, fmt: &mut pgls_console::fmt::Formatter) -> std::io::Result<()> {
        match self {
            RuleFilter::Group(group) => {
                write!(fmt, "{group}")
            }
            RuleFilter::Rule(group, rule) => {
                write!(fmt, "{group}/{rule}")
            }
        }
    }
}

/// Opaque identifier for a group of rule
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct GroupKey {
    group: &'static str,
}

impl GroupKey {
    pub(crate) fn new(group: &'static str) -> Self {
        Self { group }
    }

    pub fn group<G: RuleGroup>() -> Self {
        Self::new(G::NAME)
    }
}

impl From<GroupKey> for RuleFilter<'static> {
    fn from(key: GroupKey) -> Self {
        RuleFilter::Group(key.group)
    }
}

/// Opaque identifier for a single rule
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RuleKey {
    group: &'static str,
    rule: &'static str,
}

impl RuleKey {
    pub fn new(group: &'static str, rule: &'static str) -> Self {
        Self { group, rule }
    }

    pub fn rule<R: Rule>() -> Self {
        Self::new(<R::Group as RuleGroup>::NAME, R::METADATA.name)
    }

    pub fn group(&self) -> &'static str {
        self.group
    }

    pub fn rule_name(&self) -> &'static str {
        self.rule
    }
}

impl From<RuleKey> for RuleFilter<'static> {
    fn from(key: RuleKey) -> Self {
        RuleFilter::Rule(key.group, key.rule)
    }
}

impl PartialEq<RuleKey> for RuleFilter<'static> {
    fn eq(&self, other: &RuleKey) -> bool {
        match *self {
            RuleFilter::Group(group) => group == other.group,
            RuleFilter::Rule(group, rule) => group == other.group && rule == other.rule,
        }
    }
}
