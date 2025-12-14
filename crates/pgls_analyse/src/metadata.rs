use pgls_diagnostics::Severity;
use std::cmp::Ordering;

use crate::{categories::RuleCategory, registry::RegistryVisitor};

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
