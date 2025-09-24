use std::{future::Future, sync::LazyLock};

use tokio::runtime::Runtime;

use crate::WorkspaceError;

// Global Tokio Runtime
static RUNTIME: LazyLock<Runtime> =
    LazyLock::new(|| Runtime::new().expect("Failed to create Tokio runtime"));

/// Use this function to run async functions in the workspace, which is a sync trait called from an
/// async context.
///
/// Checkout https://greptime.com/blogs/2023-03-09-bridging-async-and-sync-rust for details.
pub fn run_async<F, R>(future: F) -> Result<R, WorkspaceError>
where
    F: Future<Output = R> + Send + 'static,
    R: Send + 'static,
{
    futures::executor::block_on(async { RUNTIME.spawn(future).await.map_err(|e| e.into()) })
}
