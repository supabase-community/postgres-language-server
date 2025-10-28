use biome_deserialize_macros::{Merge, Partial};
use bpaf::Bpaf;
use serde::{Deserialize, Serialize};

/// The configuration for type checking.
#[derive(Clone, Debug, Deserialize, Eq, Partial, PartialEq, Serialize)]
#[partial(derive(Bpaf, Clone, Eq, PartialEq, Merge))]
#[partial(cfg_attr(feature = "schema", derive(schemars::JsonSchema)))]
#[partial(serde(rename_all = "camelCase", default, deny_unknown_fields))]
pub struct PlPgSqlCheckConfiguration {
    /// if `false`, it disables the feature and pglpgsql_check won't be executed. `true` by default
    #[partial(bpaf(hide))]
    pub enabled: bool,
}

impl Default for PlPgSqlCheckConfiguration {
    fn default() -> Self {
        Self { enabled: true }
    }
}
