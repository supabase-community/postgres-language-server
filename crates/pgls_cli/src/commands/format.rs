use crate::cli_options::CliOptions;
use crate::commands::get_files_to_process_with_cli_options;
use crate::execute::run_files;
use crate::reporter::Report;
use crate::{CliDiagnostic, CliSession, VcsIntegration};
use crate::{ExecutionConfig, ExecutionMode, VcsTargeting};
use pgls_configuration::PartialConfiguration;
use pgls_diagnostics::category;
use pgls_fs::FileSystem;
use pgls_workspace::DynRef;
use std::ffi::OsString;

pub struct FormatArgs {
    pub configuration: Option<PartialConfiguration>,
    pub paths: Vec<OsString>,
    pub write: bool,
    pub staged: bool,
    pub changed: bool,
    pub since: Option<String>,
}

pub fn format(
    mut session: CliSession,
    cli_options: &CliOptions,
    args: FormatArgs,
) -> Result<(), CliDiagnostic> {
    validate_args(&args)?;

    let configuration = session.prepare_with_config(cli_options, args.configuration.clone())?;
    session.setup_workspace(configuration.clone(), VcsIntegration::Enabled)?;

    let paths = resolve_paths(session.fs(), &configuration, &args)?;

    let vcs = VcsTargeting {
        staged: args.staged,
        changed: args.changed,
    };

    let mode = ExecutionMode::Format {
        write: args.write,
        vcs,
    };
    let execution = ExecutionConfig::new(mode, u32::MAX);

    let report: Report = run_files(&mut session, &execution, paths)?;

    let exit_result = enforce_exit_codes(cli_options, &report);
    session.report("format", cli_options, &report)?;
    exit_result
}

fn resolve_paths(
    fs: &DynRef<'_, dyn FileSystem>,
    configuration: &PartialConfiguration,
    args: &FormatArgs,
) -> Result<Vec<OsString>, CliDiagnostic> {
    let mut paths = get_files_to_process_with_cli_options(
        args.since.as_deref(),
        args.changed,
        args.staged,
        fs,
        configuration,
    )?
    .unwrap_or_else(|| args.paths.clone());

    if paths.is_empty() {
        if let Some(current_dir) = fs.working_directory() {
            paths.push(current_dir.into_os_string());
        }
    }

    Ok(paths)
}

fn enforce_exit_codes(cli_options: &CliOptions, payload: &Report) -> Result<(), CliDiagnostic> {
    let traversal = payload.traversal.as_ref();
    let processed = traversal.map_or(0, |t| t.changed + t.unchanged);
    let skipped = traversal.map_or(0, |t| t.skipped);

    if processed.saturating_sub(skipped) == 0 && !cli_options.no_errors_on_unmatched {
        return Err(CliDiagnostic::no_files_processed());
    }

    let errors = payload.errors;
    let category = category!("format");

    if errors > 0 {
        return Err(CliDiagnostic::check_error(category));
    }

    Ok(())
}

fn validate_args(args: &FormatArgs) -> Result<(), CliDiagnostic> {
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
