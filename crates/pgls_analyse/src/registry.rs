use std::{borrow, collections::BTreeSet};

use crate::{
    filter::{GroupKey, RuleKey},
    metadata::{GroupCategory, RuleGroup, RuleMeta},
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
        R: RuleMeta + 'static;
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
        R: RuleMeta + 'static,
    {
        self.insert_rule(<R::Group as RuleGroup>::NAME, R::METADATA.name);
    }
}
