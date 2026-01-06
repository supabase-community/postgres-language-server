//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
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
    #[doc = r" List of rules"]
    #[partial(bpaf(pure(Default::default()), optional, hide))]
    pub rules: Rules,
    #[doc = r" A list of Unix shell style patterns. The linter will ignore files/folders that will match these patterns."]
    #[partial(bpaf(hide))]
    pub ignore: StringSet,
    #[doc = r" A list of Unix shell style patterns. The linter will include files/folders that will match these patterns."]
    #[partial(bpaf(hide))]
    pub include: StringSet,
}
impl SplinterConfiguration {
    pub const fn is_disabled(&self) -> bool {
        !self.enabled
    }
}
impl Default for SplinterConfiguration {
    fn default() -> Self {
        Self {
            enabled: true,
            rules: Default::default(),
            ignore: Default::default(),
            include: Default::default(),
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
