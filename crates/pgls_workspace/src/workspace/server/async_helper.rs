#![warn(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use std::{future::Future, sync::LazyLock};

use tokio::runtime::Runtime;

use crate::WorkspaceError;

// Global Tokio Runtime. Creating it fails if the OS can't start its threads, which every request
// that needs it reports.
static RUNTIME: LazyLock<std::io::Result<Runtime>> = LazyLock::new(Runtime::new);

/// Use this function to run async functions in the workspace, which is a sync trait called from an
/// async context.
///
/// Checkout https://greptime.com/blogs/2023-03-09-bridging-async-and-sync-rust for details.
pub fn run_async<F, R>(future: F) -> Result<R, WorkspaceError>
where
    F: Future<Output = R> + Send + 'static,
    R: Send + 'static,
{
    let runtime = RUNTIME.as_ref().map_err(|err| {
        WorkspaceError::runtime(&format!("Failed to create the Tokio runtime: {err}"))
    })?;
    futures::executor::block_on(async { runtime.spawn(future).await.map_err(|e| e.into()) })
}
