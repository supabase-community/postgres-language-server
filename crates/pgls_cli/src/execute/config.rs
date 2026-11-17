use pgls_configuration::RuleSelector;
use pgls_diagnostics::{Diagnostic, category};
use pgls_workspace::features::code_actions::FixFileMode;
#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    pub mode: ExecutionMode,
    pub max_diagnostics: u32,
    pub allow_writes: bool,
    /// Skip files with syntax errors instead of reporting them (`--skip-errors`).
    pub skip_errors: bool,
}

impl ExecutionConfig {
    pub fn new(mode: ExecutionMode, max_diagnostics: u32) -> Self {
        let allow_writes = mode.allows_writes();
        Self {
            mode,
            max_diagnostics,
            allow_writes,
            skip_errors: false,
        }
    }

    pub fn with_skip_errors(mut self, skip_errors: bool) -> Self {
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

    pub fn max_diagnostics(&self) -> u32 {
        self.max_diagnostics
    }

    pub fn allows_writes(&self) -> bool {
        self.allow_writes
    }
}

#[derive(Debug, Clone)]
pub enum ExecutionMode {
    Check {
        vcs: VcsTargeting,
        /// Which fixes to apply and write, if any.
        fix: Option<FixFileMode>,
        /// The rules to run, if not those of the configuration.
        only: Vec<RuleSelector>,
        /// The rules to skip.
        skip: Vec<RuleSelector>,
    },
    Format {
        write: bool,
        vcs: VcsTargeting,
    },
}

impl ExecutionMode {
    pub fn allows_writes(&self) -> bool {
        match self {
            ExecutionMode::Check { fix, .. } => fix.is_some(),
            ExecutionMode::Format { write, .. } => *write,
        }
    }

    pub fn vcs(&self) -> &VcsTargeting {
        match self {
            ExecutionMode::Check { vcs, .. } => vcs,
            ExecutionMode::Format { vcs, .. } => vcs,
        }
    }

    pub fn command_name(&self) -> &str {
        match self {
            ExecutionMode::Check { .. } => "check",
            ExecutionMode::Format { .. } => "format",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VcsTargeting {
    pub staged: bool,
    pub changed: bool,
}

impl VcsTargeting {
    /// Whether the files to process come from git, with `--staged` or `--changed`.
    pub fn is_targeted(&self) -> bool {
        self.staged || self.changed
    }
}

impl From<(bool, bool)> for VcsTargeting {
    fn from(value: (bool, bool)) -> Self {
        Self {
            staged: value.0,
            changed: value.1,
        }
    }
}
