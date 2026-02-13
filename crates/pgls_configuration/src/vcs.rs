use bpaf::Bpaf;
use pgls_configuration_macros::{Merge, Partial};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

const GIT_IGNORE_FILE_NAME: &str = ".gitignore";

/// Set of properties to integrate with a VCS software.
#[derive(Clone, Debug, Deserialize, Eq, Partial, PartialEq, Serialize)]
#[partial(derive(Bpaf, Clone, Eq, Merge, PartialEq))]
#[partial(cfg_attr(feature = "schema", derive(schemars::JsonSchema)))]
#[partial(serde(deny_unknown_fields, rename_all = "camelCase"))]
pub struct VcsConfiguration {
    /// Whether we should integrate itself with the VCS client
    #[partial(bpaf(long("vcs-enabled"), argument("true|false")))]
    pub enabled: bool,

    /// The kind of client.
    #[partial(bpaf(long("vcs-client-kind"), argument("git"), optional))]
    pub client_kind: VcsClientKind,

    /// Whether we should use the VCS ignore file. When [true], we will ignore the files
    /// specified in the ignore file.
    #[partial(bpaf(long("vcs-use-ignore-file"), argument("true|false")))]
    pub use_ignore_file: bool,

    /// The folder where we should check for VCS files. By default, we will use the same
    /// folder where `postgres-language-server.jsonc` was found.
    ///
    /// If we can't find the configuration, it will attempt to use the current working directory.
    /// If no current working directory can't be found, we won't use the VCS integration, and a diagnostic
    /// will be emitted
    #[partial(bpaf(long("vcs-root"), argument("PATH"), optional))]
    pub root: String,

    /// The main branch of the project
    #[partial(bpaf(long("vcs-default-branch"), argument("BRANCH"), optional))]
    pub default_branch: String,
}

impl Default for VcsConfiguration {
    fn default() -> Self {
        Self {
            client_kind: VcsClientKind::Git,
            enabled: false,
            use_ignore_file: true,
            root: Default::default(),
            default_branch: Default::default(),
        }
    }
}

impl PartialVcsConfiguration {
    pub const fn is_enabled(&self) -> bool {
        matches!(self.enabled, Some(true))
    }
    pub const fn is_disabled(&self) -> bool {
        !self.is_enabled()
    }
    pub const fn ignore_file_disabled(&self) -> bool {
        matches!(self.use_ignore_file, Some(false))
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Merge, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum VcsClientKind {
    #[default]
    /// Integration with the git client as VCS
    Git,
}

impl VcsClientKind {
    pub const fn ignore_file(&self) -> &'static str {
        match self {
            VcsClientKind::Git => GIT_IGNORE_FILE_NAME,
        }
    }
}

impl FromStr for VcsClientKind {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "git" => Ok(Self::Git),
            _ => Err("Value not supported for VcsClientKind"),
        }
    }
}
