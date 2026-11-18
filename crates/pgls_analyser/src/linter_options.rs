use pgls_analyse::RuleKey;
use rustc_hash::FxHashMap;
use std::any::{Any, TypeId};
use std::fmt::Debug;

use crate::linter_rule::LinterRule;

/// A convenient new type data structure to store the options that belong to a rule
#[derive(Debug)]
pub struct RuleOptions(TypeId, Box<dyn Any>);

impl RuleOptions {
    /// Creates a new [RuleOptions]
    pub fn new<O: 'static>(options: O) -> Self {
        Self(TypeId::of::<O>(), Box::new(options))
    }

    /// It returns the deserialized rule option, or `None` if the options are of another type.
    pub fn value<O: 'static>(&self) -> Option<&O> {
        let RuleOptions(type_id, value) = &self;
        let current_id = TypeId::of::<O>();
        // A rule tested with its options never asks for another type.
        debug_assert_eq!(type_id, &current_id);
        value.downcast_ref::<O>()
    }
}

/// A convenient new type data structure to insert and get rules
#[derive(Debug, Default)]
pub struct LinterRules(FxHashMap<RuleKey, RuleOptions>);

impl LinterRules {
    /// It tracks the options of a specific rule
    pub fn push_rule(&mut self, rule_key: RuleKey, options: RuleOptions) {
        self.0.insert(rule_key, options);
    }

    /// It retrieves the options of a stored rule, given its name
    pub fn get_rule_options<O: 'static>(&self, rule_key: &RuleKey) -> Option<&O> {
        self.0.get(rule_key).and_then(RuleOptions::value::<O>)
    }
}

/// A set of information useful to the linter infrastructure
#[derive(Debug, Default)]
pub struct LinterOptions {
    /// A data structured derived from the [`postgres-language-server.jsonc`] file
    pub rules: LinterRules,
}

impl LinterOptions {
    pub fn rule_options<R>(&self) -> Option<R::Options>
    where
        R: LinterRule<Options: Clone> + 'static,
    {
        self.rules
            .get_rule_options::<R::Options>(&RuleKey::rule::<R>())
            .cloned()
    }
}
