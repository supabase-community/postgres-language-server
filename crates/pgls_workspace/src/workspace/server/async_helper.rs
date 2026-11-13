#![warn(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use std::{
    any::Any,
    future::Future,
    io,
    panic::{self, AssertUnwindSafe},
    sync::{Mutex, OnceLock, PoisonError},
};

use tokio::runtime::Runtime;

use crate::{WORKER_STACK_SIZE_BYTES, WorkspaceError};

// Global Tokio Runtime. Its workers type check statements, which walks their parse trees and
// parses them with tree-sitter, so they get the stack size of the language server's threads.
// Creating it fails if the OS can't start its threads: the request that tried reports that, and
// the next request tries again.
static RUNTIME: OnceLock<Runtime> = OnceLock::new();
/// Held while the runtime is created, so that only one request creates it.
static RUNTIME_INIT: Mutex<()> = Mutex::new(());

fn build_runtime() -> io::Result<Runtime> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_stack_size(WORKER_STACK_SIZE_BYTES)
        .build()
}

/// Returns the runtime in `cell`, and creates it with `build` if no earlier call did.
///
/// Tokio's `Builder::build` panics instead of returning an error when the OS can't spawn a worker
/// thread. The panic is reported as an error, and `cell` stays empty, so the next call tries again
/// instead of failing for the rest of the session.
fn get_or_build<'a>(
    cell: &'a OnceLock<Runtime>,
    init: &Mutex<()>,
    build: impl FnOnce() -> io::Result<Runtime>,
) -> Result<&'a Runtime, WorkspaceError> {
    if let Some(runtime) = cell.get() {
        return Ok(runtime);
    }
    let _init = init.lock().unwrap_or_else(PoisonError::into_inner);
    if let Some(runtime) = cell.get() {
        return Ok(runtime);
    }
    let runtime = match panic::catch_unwind(AssertUnwindSafe(build)) {
        Ok(Ok(runtime)) => runtime,
        Ok(Err(err)) => return Err(runtime_error(&err.to_string())),
        Err(payload) => return Err(runtime_error(panic_message(payload.as_ref()))),
    };
    Ok(cell.get_or_init(|| runtime))
}

fn runtime_error(reason: &str) -> WorkspaceError {
    WorkspaceError::runtime(&format!("Failed to create the Tokio runtime: {reason}"))
}

fn panic_message(payload: &(dyn Any + Send)) -> &str {
    if let Some(msg) = payload.downcast_ref::<&'static str>() {
        msg
    } else if let Some(msg) = payload.downcast_ref::<String>() {
        msg
    } else {
        "it panicked"
    }
}

/// Use this function to run async functions in the workspace, which is a sync trait called from an
/// async context.
///
/// Checkout https://greptime.com/blogs/2023-03-09-bridging-async-and-sync-rust for details.
pub fn run_async<F, R>(future: F) -> Result<R, WorkspaceError>
where
    F: Future<Output = R> + Send + 'static,
    R: Send + 'static,
{
    let runtime = get_or_build(&RUNTIME, &RUNTIME_INIT, build_runtime)?;
    futures::executor::block_on(async { runtime.spawn(future).await.map_err(|e| e.into()) })
}

#[cfg(test)]
mod tests {
    use std::{
        hint::black_box,
        sync::{Mutex, OnceLock},
    };

    use super::{build_runtime, get_or_build};
    use crate::WorkspaceError;

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

    /// Tokio panics when it can't spawn a worker thread. That's reported as an error, and the
    /// next call creates the runtime instead of failing until the server restarts.
    #[test]
    fn a_runtime_that_fails_to_start_is_created_by_the_next_call() {
        let cell = OnceLock::new();
        let init = Mutex::new(());

        // The OS can't map a 1 PiB stack, so spawning the first worker thread fails.
        let failed = get_or_build(&cell, &init, || {
            tokio::runtime::Builder::new_multi_thread()
                .thread_stack_size(1 << 50)
                .build()
        });
        assert!(matches!(failed, Err(WorkspaceError::RuntimeError(_))));
        assert!(cell.get().is_none());

        let runtime = get_or_build(&cell, &init, build_runtime).unwrap();
        assert_eq!(runtime.block_on(async { 1 }), 1);
    }
}
