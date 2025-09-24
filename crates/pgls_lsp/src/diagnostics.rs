use anyhow::Error;
use pgls_workspace::WorkspaceError;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum LspError {
    WorkspaceError(WorkspaceError),
    Anyhow(anyhow::Error),
    Error(pgls_diagnostics::Error),
}

impl From<WorkspaceError> for LspError {
    fn from(value: WorkspaceError) -> Self {
        Self::WorkspaceError(value)
    }
}

impl From<pgls_diagnostics::Error> for LspError {
    fn from(value: pgls_diagnostics::Error) -> Self {
        Self::Error(value)
    }
}

impl From<anyhow::Error> for LspError {
    fn from(value: Error) -> Self {
        Self::Anyhow(value)
    }
}

impl Display for LspError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LspError::WorkspaceError(err) => {
                write!(f, "{err}")
            }
            LspError::Anyhow(err) => {
                write!(f, "{err}")
            }
            LspError::Error(err) => err.description(f),
        }
    }
}
