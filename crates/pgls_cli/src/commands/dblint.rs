use std::time::Instant;

use crate::cli_options::CliOptions;
use crate::commands::enforce_exit_codes;
use crate::reporter::Report;
use crate::{CliDiagnostic, CliSession, VcsIntegration};
use pgls_analyse::RuleCategoriesBuilder;
use pgls_configuration::PartialConfiguration;
use pgls_diagnostics::Error;
use pgls_diagnostics::category;
use pgls_workspace::features::diagnostics::{PullDatabaseDiagnosticsParams, PullDiagnosticsResult};

pub fn dblint(
    mut session: CliSession,
    cli_options: &CliOptions,
    cli_configuration: Option<PartialConfiguration>,
) -> Result<(), CliDiagnostic> {
    session.configure_workspace(cli_options, cli_configuration, VcsIntegration::Disabled)?;
    let workspace = session.workspace();

    let max_diagnostics = if cli_options.reporter.is_default() {
        cli_options.max_diagnostics.into()
    } else {
        u32::MAX
    };

    let start = Instant::now();

    let params = PullDatabaseDiagnosticsParams {
        project_key: session.project_key(),
        categories: RuleCategoriesBuilder::default().all().build(),
        max_diagnostics,
        only: Vec::new(), // Uses configuration settings
        skip: Vec::new(), // Uses configuration settings
    };

    let PullDiagnosticsResult {
        diagnostics,
        skipped_diagnostics,
    } = workspace.pull_db_diagnostics(params)?;

    let report = Report::new(
        diagnostics.into_iter().map(Error::from).collect(),
        start.elapsed(),
        skipped_diagnostics,
        None,
    );

    let exit_result = enforce_exit_codes(cli_options, &report, category!("check"));
    session.report("dblint", cli_options, &report)?;
    exit_result
}
