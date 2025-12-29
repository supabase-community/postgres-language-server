//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
mod options;
pub use options::SplinterRuleOptions;
mod rules;
use biome_deserialize::StringSet;
use biome_deserialize_macros::{Merge, Partial};
use bpaf::Bpaf;
pub use rules::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Deserialize, Eq, Partial, PartialEq, Serialize)]
#[partial(derive(Bpaf, Clone, Eq, Merge, PartialEq))]
#[partial(cfg_attr(feature = "schema", derive(schemars::JsonSchema)))]
#[partial(serde(rename_all = "camelCase", default, deny_unknown_fields))]
pub struct SplinterConfiguration {
    #[doc = r" if `false`, it disables the feature and the linter won't be executed. `true` by default"]
    #[partial(bpaf(hide))]
    pub enabled: bool,
    #[doc = r" A list of glob patterns for database objects to ignore across all rules."]
    #[doc = r" Patterns use Unix-style globs where `*` matches any sequence of characters."]
    #[doc = r#" Format: `schema.object_name`, e.g., "public.my_table", "audit.*""#]
    #[partial(bpaf(hide))]
    pub ignore: StringSet,
    #[doc = r" List of rules"]
    #[partial(bpaf(pure(Default::default()), optional, hide))]
    pub rules: Rules,
}
impl SplinterConfiguration {
    pub const fn is_disabled(&self) -> bool {
        !self.enabled
    }
    #[doc = r" Build a matcher from the global ignore patterns."]
    #[doc = r" Returns None if no patterns are configured."]
    pub fn get_global_ignore_matcher(&self) -> Option<pgls_matcher::Matcher> {
        if self.ignore.is_empty() {
            return None;
        }
        let mut m = pgls_matcher::Matcher::new(pgls_matcher::MatchOptions::default());
        for p in self.ignore.iter() {
            let _ = m.add_pattern(p);
        }
        Some(m)
    }
}
impl Default for SplinterConfiguration {
    fn default() -> Self {
        Self {
            enabled: true,
            ignore: Default::default(),
            rules: Default::default(),
        }
    }
}
impl PartialSplinterConfiguration {
    pub const fn is_disabled(&self) -> bool {
        matches!(self.enabled, Some(false))
    }
    pub fn get_rules(&self) -> Rules {
        self.rules.clone().unwrap_or_default()
    }
}
