use pgls_analyse::RuleCategories;
use pgls_configuration::RuleSelector;
use pgls_diagnostics::Diagnostic;
use pgls_fs::PgLSPath;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PullFileDiagnosticsParams {
    pub project_key: crate::workspace::ProjectKey,
    pub path: PgLSPath,
    pub categories: RuleCategories,
    /// At most this many diagnostics are returned, the most severe first.
    pub max_diagnostics: u32,
    pub only: Vec<RuleSelector>,
    pub skip: Vec<RuleSelector>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PullDiagnosticsResult {
    pub diagnostics: Vec<pgls_diagnostics::serde::Diagnostic>,
    /// How many diagnostics `max_diagnostics` left out.
    pub skipped_diagnostics: u32,
}

impl PullDiagnosticsResult {
    /// At most `max_diagnostics` of `diagnostics`: if there are more, the most severe ones, each
    /// severity in the order of `diagnostics`.
    pub fn limited(
        mut diagnostics: Vec<pgls_diagnostics::serde::Diagnostic>,
        max_diagnostics: u32,
    ) -> Self {
        let max = usize::try_from(max_diagnostics).unwrap_or(usize::MAX);
        let skipped = diagnostics.len().saturating_sub(max);
        if skipped > 0 {
            // The sort is stable.
            diagnostics.sort_by_key(|diagnostic| std::cmp::Reverse(diagnostic.severity()));
            diagnostics.truncate(max);
        }
        Self {
            diagnostics,
            skipped_diagnostics: u32::try_from(skipped).unwrap_or(u32::MAX),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CheckDatabaseConnectionParams {
    /// The project whose databases are checked: the one of its root settings and of every
    /// nested configuration file.
    pub project_key: crate::workspace::ProjectKey,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CheckDatabaseConnectionResult {
    /// Why the configured databases cannot be used, one message per database that can't.
    pub errors: Vec<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PullDatabaseDiagnosticsParams {
    pub project_key: crate::workspace::ProjectKey,
    pub categories: RuleCategories,
    /// At most this many diagnostics are returned, the most severe first.
    pub max_diagnostics: u32,
    pub only: Vec<RuleSelector>,
    pub skip: Vec<RuleSelector>,
}
