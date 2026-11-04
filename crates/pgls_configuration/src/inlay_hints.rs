use bpaf::Bpaf;
use pgls_configuration_macros::{Merge, Partial};
use serde::{Deserialize, Serialize};

/// The configuration for inlay hints.
#[derive(Clone, Debug, Deserialize, Eq, Partial, PartialEq, Serialize)]
#[partial(derive(Bpaf, Clone, Eq, PartialEq, Merge))]
#[partial(cfg_attr(feature = "schema", derive(schemars::JsonSchema)))]
#[partial(serde(rename_all = "camelCase", default, deny_unknown_fields))]
pub struct InlayHintsConfiguration {
    /// if `false`, the language server sends no inlay hints. `true` by default
    #[partial(bpaf(hide))]
    pub enabled: bool,
}

impl Default for InlayHintsConfiguration {
    fn default() -> Self {
        Self { enabled: true }
    }
}
