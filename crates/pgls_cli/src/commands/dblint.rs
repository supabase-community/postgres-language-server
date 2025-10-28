use crate::cli_options::CliOptions;
use crate::reporter::Report;
use crate::{CliDiagnostic, CliSession, VcsIntegration};
use pgls_configuration::PartialConfiguration;

pub fn dblint(
    mut session: CliSession,
    cli_options: &CliOptions,
    cli_configuration: Option<PartialConfiguration>,
) -> Result<(), CliDiagnostic> {
    let configuration = session.prepare_with_config(cli_options, cli_configuration)?;
    session.setup_workspace(configuration, VcsIntegration::Disabled)?;

    // TODO: Implement actual dblint logic here
    let report = Report::new(vec![], std::time::Duration::new(0, 0), 0, None);

    session.report("dblint", cli_options, &report)
}
