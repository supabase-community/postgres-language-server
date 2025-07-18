use pgt_workspace_macros::ignored_path;

use crate::{features::code_actions::CodeActionsParams, workspace::server::WorkspaceServer};

impl WorkspaceServer {
    #[ignored_path(path=&params.path)]
    pub fn something(&self, params: CodeActionsParams) -> Result<(), String> {
        Ok(())
    }
}
