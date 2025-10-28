use std::num::NonZeroU64;

use biome_deserialize::StringSet;
use biome_deserialize_macros::{Merge, Partial};
use bpaf::Bpaf;
use serde::{Deserialize, Serialize};

/// Limit the size of files to 1.0 MiB by default
pub const DEFAULT_FILE_SIZE_LIMIT: NonZeroU64 =
    // SAFETY: This constant is initialized with a non-zero value
    NonZeroU64::new(1024 * 1024).unwrap();

/// The configuration of the filesystem
#[derive(Clone, Debug, Deserialize, Eq, Partial, PartialEq, Serialize)]
#[partial(derive(Bpaf, Clone, Eq, PartialEq, Merge))]
#[partial(cfg_attr(feature = "schema", derive(schemars::JsonSchema)))]
#[partial(serde(rename_all = "camelCase", default, deny_unknown_fields))]
pub struct FilesConfiguration {
    /// The maximum allowed size for source code files in bytes. Files above
    /// this limit will be ignored for performance reasons. Defaults to 1 MiB
    #[partial(bpaf(long("files-max-size"), argument("NUMBER")))]
    pub max_size: NonZeroU64,

    /// A list of Unix shell style patterns. Will ignore files/folders that will
    /// match these patterns.
    #[partial(bpaf(hide))]
    pub ignore: StringSet,

    /// A list of Unix shell style patterns. Will handle only those files/folders that will
    /// match these patterns.
    #[partial(bpaf(hide))]
    pub include: StringSet,
}

impl Default for FilesConfiguration {
    fn default() -> Self {
        Self {
            max_size: DEFAULT_FILE_SIZE_LIMIT,
            ignore: Default::default(),
            include: Default::default(),
        }
    }
}
