//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
mod rules;
use bpaf::Bpaf;
use pgls_configuration_macros::{Merge, Partial};
pub use rules::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Deserialize, Eq, Partial, PartialEq, Serialize)]
#[partial(derive(Bpaf, Clone, Eq, Merge, PartialEq))]
#[partial(cfg_attr(feature = "schema", derive(schemars::JsonSchema)))]
#[partial(serde(rename_all = "camelCase", default, deny_unknown_fields))]
pub struct PglinterConfiguration {
    #[doc = r" if `false`, it disables the feature and the linter won't be executed. `true` by default"]
    #[partial(bpaf(hide))]
    pub enabled: bool,
    #[doc = r" List of rules"]
    #[partial(bpaf(pure(Default::default()), optional, hide))]
    pub rules: Rules,
}
impl PglinterConfiguration {
    pub const fn is_disabled(&self) -> bool {
        !self.enabled
    }
}
impl Default for PglinterConfiguration {
    fn default() -> Self {
        Self {
            enabled: true,
            rules: Default::default(),
        }
    }
}
impl PartialPglinterConfiguration {
    pub const fn is_disabled(&self) -> bool {
        matches!(self.enabled, Some(false))
    }
    pub fn get_rules(&self) -> Rules {
        self.rules.clone().unwrap_or_default()
    }
}
