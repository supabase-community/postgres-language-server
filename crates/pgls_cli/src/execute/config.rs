#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    pub mode: ExecutionMode,
    pub max_diagnostics: u32,
    pub allow_writes: bool,
}

impl ExecutionConfig {
    pub fn new(mode: ExecutionMode, max_diagnostics: u32) -> Self {
        let allow_writes = mode.allows_writes();
        Self {
            mode,
            max_diagnostics,
            allow_writes,
        }
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
    Check { vcs: VcsTargeting },
    Format { write: bool, vcs: VcsTargeting },
}

impl ExecutionMode {
    pub fn allows_writes(&self) -> bool {
        match self {
            ExecutionMode::Check { .. } => false,
            ExecutionMode::Format { write, .. } => *write,
        }
    }

    pub fn vcs(&self) -> &VcsTargeting {
        match self {
            ExecutionMode::Check { vcs } => vcs,
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

impl From<(bool, bool)> for VcsTargeting {
    fn from(value: (bool, bool)) -> Self {
        Self {
            staged: value.0,
            changed: value.1,
        }
    }
}
