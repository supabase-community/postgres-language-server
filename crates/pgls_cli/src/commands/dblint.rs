use std::time::Instant;

use crate::cli_options::CliOptions;
use crate::reporter::Report;
use crate::{CliDiagnostic, CliSession, VcsIntegration};
use pgls_configuration::PartialConfiguration;
use pgls_diagnostics::Error;
use pgls_workspace::features::diagnostics::{PullDatabaseDiagnosticsParams, PullDiagnosticsResult};

pub fn dblint(
    mut session: CliSession,
    cli_options: &CliOptions,
    cli_configuration: Option<PartialConfiguration>,
) -> Result<(), CliDiagnostic> {
    let configuration = session.prepare_with_config(cli_options, cli_configuration)?;
    session.setup_workspace(configuration, VcsIntegration::Disabled)?;
    let workspace = session.workspace();

    let max_diagnostics = if cli_options.reporter.is_default() {
        cli_options.max_diagnostics.into()
    } else {
        u32::MAX
    };

    let start = Instant::now();

    let PullDiagnosticsResult {
        diagnostics,
        skipped_diagnostics,
    } = workspace.pull_db_diagnostics(PullDatabaseDiagnosticsParams { max_diagnostics })?;

    let report = Report::new(
        diagnostics.into_iter().map(Error::from).collect(),
        start.elapsed(),
        skipped_diagnostics,
        None,
    );

    session.report("dblint", cli_options, &report)
}
