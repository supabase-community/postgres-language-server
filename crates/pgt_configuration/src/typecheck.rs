use biome_deserialize::StringSet;
use biome_deserialize_macros::{Merge, Partial};
use bpaf::Bpaf;
use serde::{Deserialize, Serialize};

/// The configuration for type checking.
#[derive(Clone, Debug, Deserialize, Eq, Partial, PartialEq, Serialize)]
#[partial(derive(Bpaf, Clone, Eq, PartialEq, Merge))]
#[partial(cfg_attr(feature = "schema", derive(schemars::JsonSchema)))]
#[partial(serde(rename_all = "camelCase", default, deny_unknown_fields))]
pub struct TypecheckConfiguration {
    /// if `false`, it disables the feature and the typechecker won't be executed. `true` by default
    #[partial(bpaf(hide))]
    pub enabled: bool,
    /// Default search path schemas for type checking.
    /// Can be a list of schema names or glob patterns like ["public", "app_*"].
    /// If not specified, defaults to ["public"].
    #[partial(bpaf(long("search_path")))]
    pub search_path: StringSet,
}

impl Default for TypecheckConfiguration {
    fn default() -> Self {
        Self {
            enabled: true,
            search_path: ["public".to_string()].into_iter().collect(),
        }
    }
}
