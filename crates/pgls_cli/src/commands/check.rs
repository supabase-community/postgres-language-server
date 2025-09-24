use crate::cli_options::CliOptions;
use crate::{CliDiagnostic, Execution, TraversalMode};
use biome_deserialize::Merge;
use pgls_configuration::PartialConfiguration;
use pgls_console::Console;
use pgls_fs::FileSystem;
use pgls_workspace::{DynRef, Workspace, WorkspaceError, configuration::LoadedConfiguration};
use std::ffi::OsString;

use super::{CommandRunner, get_files_to_process_with_cli_options};

pub(crate) struct CheckCommandPayload {
    pub(crate) configuration: Option<PartialConfiguration>,
    pub(crate) paths: Vec<OsString>,
    pub(crate) stdin_file_path: Option<String>,
    pub(crate) staged: bool,
    pub(crate) changed: bool,
    pub(crate) since: Option<String>,
}

impl CommandRunner for CheckCommandPayload {
    const COMMAND_NAME: &'static str = "check";

    fn merge_configuration(
        &mut self,
        loaded_configuration: LoadedConfiguration,
        _fs: &DynRef<'_, dyn FileSystem>,
        _console: &mut dyn Console,
    ) -> Result<PartialConfiguration, WorkspaceError> {
        let LoadedConfiguration {
            configuration: mut fs_configuration,
            ..
        } = loaded_configuration;

        if let Some(configuration) = self.configuration.clone() {
            // overwrite fs config with cli args
            fs_configuration.merge_with(configuration);
        }

        Ok(fs_configuration)
    }

    fn get_files_to_process(
        &self,
        fs: &DynRef<'_, dyn FileSystem>,
        configuration: &PartialConfiguration,
    ) -> Result<Vec<OsString>, CliDiagnostic> {
        let paths = get_files_to_process_with_cli_options(
            self.since.as_deref(),
            self.changed,
            self.staged,
            fs,
            configuration,
        )?
        .unwrap_or(self.paths.clone());

        Ok(paths)
    }

    fn get_stdin_file_path(&self) -> Option<&str> {
        self.stdin_file_path.as_deref()
    }

    fn get_execution(
        &self,
        cli_options: &CliOptions,
        console: &mut dyn Console,
        _workspace: &dyn Workspace,
    ) -> Result<Execution, CliDiagnostic> {
        Ok(Execution::new(TraversalMode::Check {
            stdin: self.get_stdin(console)?,
            vcs_targeted: (self.staged, self.changed).into(),
        })
        .set_report(cli_options))
    }
}
