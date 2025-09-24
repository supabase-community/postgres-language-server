use rustc_hash::FxHashMap;

use crate::{Rule, RuleKey};
use std::any::{Any, TypeId};
use std::fmt::Debug;

/// A convenient new type data structure to store the options that belong to a rule
#[derive(Debug)]
pub struct RuleOptions(TypeId, Box<dyn Any>);

impl RuleOptions {
    /// Creates a new [RuleOptions]
    pub fn new<O: 'static>(options: O) -> Self {
        Self(TypeId::of::<O>(), Box::new(options))
    }

    /// It returns the deserialized rule option
    pub fn value<O: 'static>(&self) -> &O {
        let RuleOptions(type_id, value) = &self;
        let current_id = TypeId::of::<O>();
        debug_assert_eq!(type_id, &current_id);
        // SAFETY: the code should fail when asserting the types.
        // If the code throws an error here, it means that the developer didn't test
        // the rule with the options
        value.downcast_ref::<O>().unwrap()
    }
}

/// A convenient new type data structure to insert and get rules
#[derive(Debug, Default)]
pub struct AnalyserRules(FxHashMap<RuleKey, RuleOptions>);

impl AnalyserRules {
    /// It tracks the options of a specific rule
    pub fn push_rule(&mut self, rule_key: RuleKey, options: RuleOptions) {
        self.0.insert(rule_key, options);
    }

    /// It retrieves the options of a stored rule, given its name
    pub fn get_rule_options<O: 'static>(&self, rule_key: &RuleKey) -> Option<&O> {
        self.0.get(rule_key).map(|o| o.value::<O>())
    }
}

/// A set of information useful to the analyser infrastructure
#[derive(Debug, Default)]
pub struct AnalyserOptions {
    /// A data structured derived from the [`postgres-language-server.jsonc`] file
    pub rules: AnalyserRules,
}

impl AnalyserOptions {
    pub fn rule_options<R>(&self) -> Option<R::Options>
    where
        R: Rule<Options: Clone> + 'static,
    {
        self.rules
            .get_rule_options::<R::Options>(&RuleKey::rule::<R>())
            .cloned()
    }
}
