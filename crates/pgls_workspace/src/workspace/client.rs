use crate::workspace::ServerInfo;
use crate::{TransportError, Workspace, WorkspaceError};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::json;
use std::{
    panic::RefUnwindSafe,
    sync::atomic::{AtomicU64, Ordering},
};

pub struct WorkspaceClient<T> {
    transport: T,
    request_id: AtomicU64,
    server_info: Option<ServerInfo>,
}

pub trait WorkspaceTransport {
    fn request<P, R>(&self, request: TransportRequest<P>) -> Result<R, TransportError>
    where
        P: Serialize,
        R: DeserializeOwned;
}

#[derive(Debug)]
pub struct TransportRequest<P> {
    pub id: u64,
    pub method: &'static str,
    pub params: P,
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Deserialize, Serialize)]
pub(super) struct InitializeResult {
    /// Information about the server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_info: Option<ServerInfo>,
}

impl<T> WorkspaceClient<T>
where
    T: WorkspaceTransport + RefUnwindSafe + Send + Sync,
{
    pub fn new(transport: T) -> Result<Self, WorkspaceError> {
        let mut client = Self {
            transport,
            request_id: AtomicU64::new(0),
            server_info: None,
        };

        // TODO: The current implementation of the JSON-RPC protocol in
        // tower_lsp_server doesn't allow any request to be sent before a call to
        // initialize, this is something we could be able to lift by using our
        // own RPC protocol implementation
        let value: InitializeResult = client.request(
            "initialize",
            json!({
                "capabilities": {},
                "clientInfo": {
                    "name": env!("CARGO_PKG_NAME"),
                    "version": pgls_env::VERSION
                },
            }),
        )?;

        client.server_info = value.server_info;

        Ok(client)
    }

    fn request<P, R>(&self, method: &'static str, params: P) -> Result<R, WorkspaceError>
    where
        P: Serialize,
        R: DeserializeOwned,
    {
        let id = self.request_id.fetch_add(1, Ordering::Relaxed);
        let request = TransportRequest { id, method, params };

        let response = self.transport.request(request)?;

        Ok(response)
    }

    pub fn shutdown(self) -> Result<(), WorkspaceError> {
        self.request("pgls/shutdown", ())
    }
}

macro_rules! workspace_client {
    ($($(#[$attr:meta])* fn $name:ident($params:ty) -> $result:ty;)*) => {
        impl<T> Workspace for WorkspaceClient<T>
        where
            T: WorkspaceTransport + RefUnwindSafe + Send + Sync,
        {
            $(
                fn $name(&self, params: $params) -> Result<$result, WorkspaceError> {
                    self.request(concat!("pgls/", stringify!($name)), params)
                }
            )*

            fn server_info(&self) -> Option<&ServerInfo> {
                self.server_info.as_ref()
            }
        }
    };
}

crate::for_each_workspace_method!(workspace_client);
