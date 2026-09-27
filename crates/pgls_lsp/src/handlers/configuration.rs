use crate::session::Session;
use pgls_configuration::PartialConfiguration;
use serde::Deserialize;
use tower_lsp::jsonrpc::{Error, Result as LspResult};
use tower_lsp::lsp_types::DidChangeConfigurationParams;

/// Deliberately not `Debug`: the payload carries whatever the client overrides, including database
/// credentials, and `PartialDatabaseConfiguration` renders its password verbatim.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SetConfigurationOverridesParams {
    /// Configuration applied on top of the configuration file and the
    /// environment, or `null` to drop the current overrides. Replaces the
    /// previous overrides; it does not merge with them.
    #[serde(default)]
    pub(crate) overrides: Option<PartialConfiguration>,
}

/// This is a request rather than a notification so the client receives an
/// acknowledgement and can distinguish applied from failed configuration.
pub(crate) async fn set_configuration_overrides(
    session: &Session,
    params: SetConfigurationOverridesParams,
) -> LspResult<()> {
    let status = session.set_client_overrides(params.overrides).await;
    if status.is_error() {
        return Err(Error {
            code: tower_lsp::jsonrpc::ErrorCode::InternalError,
            message: "Configuration overrides could not be applied; check the server log".into(),
            data: None,
        });
    }
    Ok(())
}

/// The configuration notification writes a sticky layer so the snapshot
/// survives later reloads, unlike the former one-shot merge.
pub(crate) async fn did_change_configuration(
    session: &Session,
    params: DidChangeConfigurationParams,
) {
    match serde_json::from_value::<PartialConfiguration>(params.settings) {
        Ok(configuration) => {
            session.set_client_overrides(Some(configuration)).await;
        }
        Err(error) => {
            tracing::info!("Ignoring invalid workspace/didChangeConfiguration settings: {error}");
            session.reload_workspace_settings().await;
            session.update_all_diagnostics().await;
        }
    }
}
