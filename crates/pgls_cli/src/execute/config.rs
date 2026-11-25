use crate::commands::Targets;
use pgls_configuration::RuleSelector;
use pgls_diagnostics::{Diagnostic, category};
use pgls_workspace::features::code_actions::FixFileMode;
#[derive(Debug, Clone)]
pub(crate) struct ExecutionConfig {
    pub(crate) mode: ExecutionMode,
    max_diagnostics: u32,
    /// Whether the files to process come from git, with `--staged` or `--changed`.
    vcs_targeted: bool,
    /// Skip files with syntax errors instead of reporting them (`--skip-errors`).
    pub(crate) skip_errors: bool,
}

impl ExecutionConfig {
    pub(crate) fn new(mode: ExecutionMode, max_diagnostics: u32, targets: &Targets) -> Self {
        Self {
            mode,
            max_diagnostics,
            vcs_targeted: targets.is_vcs(),
            skip_errors: false,
        }
    }

    pub(crate) fn with_skip_errors(mut self, skip_errors: bool) -> Self {
        self.skip_errors = skip_errors;
        self
    }

    /// Whether `--skip-errors` skips a file with `diagnostics`: they contain a syntax error.
    pub(crate) fn skips<'a, D: Diagnostic + 'a>(
        &self,
        diagnostics: impl IntoIterator<Item = &'a D>,
    ) -> bool {
        self.skip_errors
            && diagnostics
                .into_iter()
                .any(|diagnostic| diagnostic.category() == Some(category!("syntax")))
    }

    pub(crate) fn max_diagnostics(&self) -> u32 {
        self.max_diagnostics
    }

    pub(crate) fn allows_writes(&self) -> bool {
        match &self.mode {
            ExecutionMode::Check { fix, .. } => fix.is_some(),
            ExecutionMode::Format { write } => *write,
        }
    }

    /// Whether the files to process come from git, with `--staged` or `--changed`.
    pub(crate) fn is_vcs_targeted(&self) -> bool {
        self.vcs_targeted
    }
}

#[derive(Debug, Clone)]
pub(crate) enum ExecutionMode {
    Check {
        /// Which fixes to apply and write, if any.
        fix: Option<FixFileMode>,
        /// The rules to run, if not those of the configuration.
        only: Vec<RuleSelector>,
        /// The rules to skip.
        skip: Vec<RuleSelector>,
    },
    Format {
        write: bool,
    },
}
