use biome_deserialize_macros::{Merge, Partial};
use bpaf::Bpaf;
use serde::{Deserialize, Serialize};

/// The configuration of the filesystem
#[derive(Clone, Debug, Deserialize, Eq, Partial, PartialEq, Serialize, Default)]
#[partial(derive(Bpaf, Clone, Eq, PartialEq, Merge))]
#[partial(serde(rename_all = "camelCase", default, deny_unknown_fields))]
#[partial(cfg_attr(feature = "schema", derive(schemars::JsonSchema)))]
pub struct MigrationsConfiguration {
    /// The directory where the migration files are stored
    #[partial(bpaf(long("migrations-dir")))]
    pub migrations_dir: String,

    /// Ignore any migrations before this timestamp
    #[partial(bpaf(long("after")))]
    pub after: u64,
}
