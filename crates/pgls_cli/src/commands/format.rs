use crate::cli_options::CliOptions;
use crate::commands::{FileTargets, enforce_exit_codes, print_diagnostics, read_stdin_payload};
use crate::execute::{run_files, run_stdin};
use crate::reporter::Report;
use crate::{CliDiagnostic, CliSession, VcsIntegration};
use crate::{ExecutionConfig, ExecutionMode};
use pgls_configuration::PartialConfiguration;
use pgls_diagnostics::category;

pub struct FormatArgs {
    pub configuration: Option<PartialConfiguration>,
    pub targets: FileTargets,
    pub write: bool,
}

pub fn format(
    mut session: CliSession,
    cli_options: &CliOptions,
    args: FormatArgs,
) -> Result<(), CliDiagnostic> {
    args.targets.validate()?;
    if args.write && args.targets.stdin_file_path.is_some() {
        return Err(CliDiagnostic::incompatible_arguments(
            "stdin-file-path",
            "write",
        ));
    }

    let configuration = session.prepare_with_config(cli_options, args.configuration.clone())?;
    session.setup_workspace(configuration.clone(), VcsIntegration::Enabled)?;

    let mode = ExecutionMode::Format {
        write: args.write,
        vcs: args.targets.vcs(),
    };
    let execution = ExecutionConfig::new(mode, u32::MAX);

    let paths = args.targets.resolve_paths(session.fs(), &configuration)?;
    let configuration_errors = session.apply_nested_configurations(
        cli_options,
        args.configuration.clone(),
        &args.targets.nested_configuration_paths(&paths),
    )?;

    if let Some(stdin_path) = args.targets.stdin_file_path.as_deref() {
        print_diagnostics(session.console(), configuration_errors);
        let payload = read_stdin_payload(stdin_path, session.console(), "format")?;
        return run_stdin(&mut session, &execution, payload);
    }

    let mut report: Report = run_files(&mut session, &execution, paths)?;
    for error in configuration_errors.into_iter().rev() {
        report.prepend_diagnostic(error);
    }

    let exit_result = enforce_exit_codes(cli_options, &report, category!("format"));
    session.report("format", cli_options, &report)?;
    exit_result
}
