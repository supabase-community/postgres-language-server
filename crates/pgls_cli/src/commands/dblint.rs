use std::time::Instant;

use crate::cli_options::CliOptions;
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
    let configuration = session.prepare_with_config(cli_options, cli_configuration)?;
    session.setup_workspace(configuration, VcsIntegration::Disabled)?;
    let workspace = session.workspace();

    let max_diagnostics = if cli_options.reporter.is_default() {
        cli_options.max_diagnostics.into()
    } else {
        u32::MAX
    };

    let start = Instant::now();

    let params = PullDatabaseDiagnosticsParams {
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

    let exit_result = enforce_exit_codes(cli_options, &report);
    session.report("dblint", cli_options, &report)?;
    exit_result
}

fn enforce_exit_codes(cli_options: &CliOptions, payload: &Report) -> Result<(), CliDiagnostic> {
    let errors = payload.errors;
    let warnings = payload.warnings;
    let category = category!("check");

    if errors > 0 {
        return Err(CliDiagnostic::check_error(category));
    }

    if warnings > 0 && cli_options.error_on_warnings {
        return Err(CliDiagnostic::check_warnings(category));
    }

    Ok(())
}
