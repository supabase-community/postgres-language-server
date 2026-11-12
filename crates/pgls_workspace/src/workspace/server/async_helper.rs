use std::{future::Future, sync::LazyLock};

use tokio::runtime::Runtime;

use crate::{WORKER_STACK_SIZE_BYTES, WorkspaceError};

// Global Tokio Runtime. Its workers type check statements, which walks their parse trees and
// parses them with tree-sitter, so they get the stack size of the language server's threads.
// Creating it fails if the OS can't start its threads, which every request that needs it
// reports.
static RUNTIME: LazyLock<std::io::Result<Runtime>> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_stack_size(WORKER_STACK_SIZE_BYTES)
        .build()
});

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

#[cfg(test)]
mod tests {
    use std::hint::black_box;

    /// Recurses until the frames below `base` take at least `target` bytes of stack, and returns
    /// how many they take.
    fn use_stack(base: usize, target: usize) -> usize {
        let frame = black_box([0u8; 1024]);
        let used = base.abs_diff(frame.as_ptr() as usize);
        if used >= target {
            return used;
        }
        black_box(use_stack(base, target)) + usize::from(frame[0])
    }

    /// The futures get the stack of the language server's threads, not tokio's 2 MiB default,
    /// which this recursion would overflow, aborting the process.
    #[test]
    fn futures_run_with_the_worker_stack_size() {
        const TARGET: usize = 4 * 1024 * 1024;
        let used = super::run_async(async {
            let base = black_box(0u8);
            use_stack(&base as *const u8 as usize, TARGET)
        })
        .unwrap();
        assert!(used >= TARGET);
    }
}
