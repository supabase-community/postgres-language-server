use crate::cli_options::CliOptions;
use crate::commands::get_files_to_process_with_cli_options;
use crate::execute::{StdinPayload, run_files, run_stdin};
use crate::reporter::Report;
use crate::{CliDiagnostic, CliSession, VcsIntegration};
use crate::{ExecutionConfig, ExecutionMode, VcsTargeting};
use pgls_configuration::PartialConfiguration;
use pgls_console::Console;
use pgls_diagnostics::category;
use pgls_fs::FileSystem;
use pgls_workspace::DynRef;
use std::ffi::OsString;

pub struct CheckArgs {
    pub configuration: Option<PartialConfiguration>,
    pub paths: Vec<OsString>,
    pub stdin_file_path: Option<String>,
    pub staged: bool,
    pub changed: bool,
    pub since: Option<String>,
}

pub fn check(
    mut session: CliSession,
    cli_options: &CliOptions,
    args: CheckArgs,
) -> Result<(), CliDiagnostic> {
    validate_args(&args)?;

    let configuration = session.prepare_with_config(cli_options, args.configuration.clone())?;
    session.setup_workspace(configuration.clone(), VcsIntegration::Enabled)?;

    let paths = resolve_paths(session.fs(), &configuration, &args)?;

    let vcs = VcsTargeting {
        staged: args.staged,
        changed: args.changed,
    };

    let max_diagnostics = if cli_options.reporter.is_default() {
        cli_options.max_diagnostics.into()
    } else {
        u32::MAX
    };

    let mode = ExecutionMode::Check { vcs };
    let execution = ExecutionConfig::new(mode, max_diagnostics);

    if let Some(stdin_path) = args.stdin_file_path.as_deref() {
        let payload = read_stdin_payload(stdin_path, session.console())?;
        run_stdin(&mut session, &execution, payload)
    } else {
        let report: Report = run_files(&mut session, &execution, paths)?;

        let exit_result = enforce_exit_codes(cli_options, &report);
        session.report("check", cli_options, &report)?;
        exit_result
    }
}

fn resolve_paths(
    fs: &DynRef<'_, dyn FileSystem>,
    configuration: &PartialConfiguration,
    args: &CheckArgs,
) -> Result<Vec<OsString>, CliDiagnostic> {
    let mut paths = get_files_to_process_with_cli_options(
        args.since.as_deref(),
        args.changed,
        args.staged,
        fs,
        configuration,
    )?
    .unwrap_or_else(|| args.paths.clone());

    if paths.is_empty() && args.stdin_file_path.is_none() {
        if let Some(current_dir) = fs.working_directory() {
            paths.push(current_dir.into_os_string());
        }
    }

    Ok(paths)
}

fn read_stdin_payload(
    path: &str,
    console: &mut dyn Console,
) -> Result<StdinPayload, CliDiagnostic> {
    let input_code = console.read();
    if let Some(input_code) = input_code {
        Ok(StdinPayload {
            path: path.into(),
            content: input_code,
        })
    } else {
        Err(CliDiagnostic::missing_argument("stdin", "check"))
    }
}

fn enforce_exit_codes(cli_options: &CliOptions, payload: &Report) -> Result<(), CliDiagnostic> {
    let traversal = payload.traversal.as_ref();
    let processed = traversal.map_or(0, |t| t.changed + t.unchanged);
    let skipped = traversal.map_or(0, |t| t.skipped);

    if processed.saturating_sub(skipped) == 0 && !cli_options.no_errors_on_unmatched {
        return Err(CliDiagnostic::no_files_processed());
    }

    let warnings = payload.warnings;
    let errors = payload.errors;
    let category = category!("check");

    if errors > 0 {
        return Err(CliDiagnostic::check_error(category));
    }

    if warnings > 0 && cli_options.error_on_warnings {
        return Err(CliDiagnostic::check_warnings(category));
    }

    Ok(())
}

fn validate_args(args: &CheckArgs) -> Result<(), CliDiagnostic> {
    if args.since.is_some() {
        if !args.changed {
            return Err(CliDiagnostic::incompatible_arguments("since", "changed"));
        }
        if args.staged {
            return Err(CliDiagnostic::incompatible_arguments("since", "staged"));
        }
    }

    if args.changed && args.staged {
        return Err(CliDiagnostic::incompatible_arguments("changed", "staged"));
    }

    Ok(())
}
