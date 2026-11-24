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
    /// Whether to check the statements against the database. Defaults to
    /// [`DatabaseChecks::Run`].
    #[serde(default)]
    pub database_checks: DatabaseChecks,
}

/// Whether a diagnostics pull checks the statements against the database: EXPLAIN for the
/// statements whose references all come unchanged from the database, and plpgsql_check for
/// PL/pgSQL functions. The checks take a round trip per statement, seconds on long files, so the
/// language server runs them when a file is opened or saved, and reuses their results while the
/// user types.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum DatabaseChecks {
    /// Check against the database, and keep the results of each statement for
    /// [`DatabaseChecks::Reuse`].
    #[default]
    Run,
    /// Don't query the database: report what the last [`DatabaseChecks::Run`] found for the
    /// statements whose text is unchanged since. An edited statement has no results until the
    /// next run.
    Reuse,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PullDiagnosticsResult {
    pub diagnostics: Vec<pgls_diagnostics::serde::Diagnostic>,
    /// How many diagnostics `max_diagnostics` left out.
    pub skipped_diagnostics: u32,
    /// The client's version of the document whose text the diagnostics are for. `None` if they
    /// don't depend on it: the path is ignored or no settings apply, or they are the database's.
    /// A pull that stopped because the document changed has no diagnostics and the version of
    /// the change, see [PullDiagnosticsResult::outdated].
    pub version: Option<i32>,
}

impl PullDiagnosticsResult {
    /// The result of a pull that stopped because the document changed, now at `version`: no
    /// diagnostics, and a version that tells the caller that they are outdated.
    pub fn outdated(version: Option<i32>) -> Self {
        Self {
            version,
            ..Self::default()
        }
    }

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
            version: None,
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
