use std::{
    any::Any,
    future::Future,
    io,
    panic::{self, AssertUnwindSafe},
    sync::{
        Arc, Mutex, OnceLock, PoisonError,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, RecvTimeoutError},
    },
    time::Duration,
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

/// How often [`run_async_until`] checks whether to stop.
const STOP_CHECK_PERIOD: Duration = Duration::from_millis(20);

/// Like [`run_async`], for a future that winds down once `stop` is set, like one that stops
/// starting new work. While it runs, `should_stop` is checked on this thread, before it starts
/// and every [`STOP_CHECK_PERIOD`], so that it can read what is local to the thread, like the
/// request's cancellation. Once it returns `true`, `stop` is set.
pub fn run_async_until<F, R>(
    future: F,
    stop: &Arc<AtomicBool>,
    should_stop: &dyn Fn() -> bool,
) -> Result<R, WorkspaceError>
where
    F: Future<Output = R> + Send + 'static,
    R: Send + 'static,
{
    if should_stop() {
        stop.store(true, Ordering::Relaxed);
    }
    let runtime = get_or_build(&RUNTIME, &RUNTIME_INIT, build_runtime)?;
    let (done, finished) = mpsc::channel::<()>();
    let task = runtime.spawn(async move {
        let result = future.await;
        // The receiver only waits for this, and a panic drops the sender as well.
        let _ = done.send(());
        result
    });
    while let Err(RecvTimeoutError::Timeout) = finished.recv_timeout(STOP_CHECK_PERIOD) {
        if !stop.load(Ordering::Relaxed) && should_stop() {
            stop.store(true, Ordering::Relaxed);
        }
    }
    futures::executor::block_on(task).map_err(|e| e.into())
}

#[cfg(test)]
mod tests {
    use std::{
        hint::black_box,
        sync::{
            Arc, Mutex, OnceLock,
            atomic::{AtomicBool, AtomicUsize, Ordering},
        },
        time::Duration,
    };

    use super::{build_runtime, get_or_build, run_async_until};
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

    #[test]
    fn the_future_stops_once_it_should() {
        let stop = Arc::new(AtomicBool::new(false));
        let checks = AtomicUsize::new(0);
        // Counts until it is stopped, which the third check does.
        let counted = run_async_until(
            {
                let stop = Arc::clone(&stop);
                async move {
                    let mut count = 0;
                    while !stop.load(Ordering::Relaxed) {
                        count += 1;
                        std::thread::sleep(Duration::from_millis(5));
                        tokio::task::yield_now().await;
                    }
                    count
                }
            },
            &stop,
            &|| checks.fetch_add(1, Ordering::Relaxed) == 2,
        )
        .unwrap();
        assert!(counted > 0);
        assert_eq!(checks.load(Ordering::Relaxed), 3);
    }

    #[test]
    fn a_future_that_should_stop_right_away_is_stopped_before_it_starts() {
        let stop = Arc::new(AtomicBool::new(false));
        let started = run_async_until(
            {
                let stop = Arc::clone(&stop);
                async move { !stop.load(Ordering::Relaxed) }
            },
            &stop,
            &|| true,
        )
        .unwrap();
        assert!(!started);
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
