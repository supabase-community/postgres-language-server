use crate::cli_options::CliOptions;
use crate::commands::{FileTargets, enforce_exit_codes, print_diagnostics, read_stdin_payload};
use crate::diagnostics::DatabaseUnavailable;
use crate::execute::{run_files, run_stdin};
use crate::reporter::Report;
use crate::{CliDiagnostic, CliSession, VcsIntegration};
use crate::{ExecutionConfig, ExecutionMode};
use pgls_configuration::{PartialConfiguration, RuleSelector};
use pgls_diagnostics::{Error, category};
use pgls_workspace::features::code_actions::FixFileMode;
use pgls_workspace::features::diagnostics::CheckDatabaseConnectionParams;

pub struct CheckArgs {
    pub configuration: Option<PartialConfiguration>,
    pub targets: FileTargets,
    pub fix: bool,
    pub unsafe_: bool,
    pub only: Vec<RuleSelector>,
    pub skip: Vec<RuleSelector>,
}

pub fn check(
    mut session: CliSession,
    cli_options: &CliOptions,
    args: CheckArgs,
) -> Result<(), CliDiagnostic> {
    args.targets.validate()?;

    let configuration = session.configure_workspace(
        cli_options,
        args.configuration.clone(),
        VcsIntegration::Enabled,
    )?;

    let paths = args.targets.resolve_paths(session.fs(), &configuration)?;
    let configuration_errors = session.apply_nested_configurations(
        cli_options,
        &args.targets.nested_configuration_paths(&paths),
    )?;

    let max_diagnostics = if cli_options.reporter.is_default() {
        cli_options.max_diagnostics.into()
    } else {
        u32::MAX
    };

    let fix = match (args.fix, args.unsafe_) {
        (_, true) => Some(FixFileMode::SafeAndUnsafeFixes),
        (true, false) => Some(FixFileMode::SafeFixes),
        (false, false) => None,
    };
    let mode = ExecutionMode::Check {
        vcs: args.targets.vcs(),
        fix,
        only: args.only.clone(),
        skip: args.skip.clone(),
    };
    let execution = ExecutionConfig::new(mode, max_diagnostics);

    if let Some(stdin_path) = args.targets.stdin_file_path.as_deref() {
        print_diagnostics(session.console(), configuration_errors);
        let payload = read_stdin_payload(stdin_path, session.console(), "check")?;
        run_stdin(&mut session, &execution, payload)
    } else {
        let database_errors = check_database_connection(&session)?;
        let mut report: Report = run_files(&mut session, &execution, paths)?;
        for error in database_errors.into_iter().rev() {
            report.prepend_diagnostic(error);
        }
        for error in configuration_errors.into_iter().rev() {
            report.prepend_diagnostic(error);
        }

        let exit_result = enforce_exit_codes(cli_options, &report, category!("check"));
        session.report("check", cli_options, &report)?;
        exit_result
    }
}

/// A configured database that cannot be reached is an error: the checks that need it would
/// otherwise be skipped silently.
fn check_database_connection(session: &CliSession) -> Result<Vec<Error>, CliDiagnostic> {
    let result = session
        .workspace()
        .check_database_connection(CheckDatabaseConnectionParams {
            project_key: session.project_key(),
        })?;
    Ok(result
        .errors
        .into_iter()
        .map(|message| Error::from(DatabaseUnavailable::new(message)))
        .collect())
}
