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
    time::{Duration, Instant},
};

use tokio::runtime::{Handle, Runtime};

use super::database::{DatabaseQueries, Postgres};
use crate::{WORKER_STACK_SIZE_BYTES, WorkspaceError};

/// How often [`WorkspaceRuntime::run_until`] checks whether to stop.
const STOP_CHECK_PERIOD: Duration = Duration::from_millis(20);

/// Where the workspace runs its async work, what answers its database queries, and the clock of
/// its backoffs and idle timeouts.
///
/// The workspace's methods are sync and called from threads that may not belong to a runtime, so
/// the futures run on this runtime while the calling thread waits for them. See
/// https://greptime.com/blogs/2023-03-09-bridging-async-and-sync-rust for details.
pub(crate) struct WorkspaceRuntime {
    executor: Executor,
    database: Arc<dyn DatabaseQueries>,
    clock: Clock,
}

enum Executor {
    /// A runtime of the workspace's own, created on first use.
    Own(OwnRuntime),
    /// The runtime of whoever created the workspace, e.g. a test's runtime with a paused clock.
    Injected(Handle),
}

impl Default for WorkspaceRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceRuntime {
    /// A runtime of the workspace's own, created when the first future runs, the queries sent to
    /// Postgres, and the system clock.
    pub(crate) fn new() -> Self {
        Self {
            executor: Executor::Own(OwnRuntime::default()),
            database: Arc::new(Postgres),
            clock: Clock::system(),
        }
    }

    /// The runtime of `handle` and its clock, and the queries answered by `database`. Paused, the
    /// clock only moves when the runtime advances it.
    ///
    /// The futures run on the thread that calls the workspace, in the context of the runtime. So
    /// they may wait for the runtime's timers and IO only if that thread doesn't drive a
    /// current-thread runtime of `handle`: called from its `spawn_blocking` threads, as the
    /// language server does by default, they can. A fake `database` that answers at once works
    /// either way, also with the blocking work inline on the runtime's thread.
    pub(crate) fn with_handle(handle: Handle, database: Arc<dyn DatabaseQueries>) -> Self {
        Self {
            clock: Clock::of(&handle),
            executor: Executor::Injected(handle),
            database,
        }
    }

    /// The clock of the backoffs and idle timeouts.
    pub(crate) fn clock(&self) -> &Clock {
        &self.clock
    }

    /// What answers the database queries.
    pub(crate) fn database(&self) -> Arc<dyn DatabaseQueries> {
        Arc::clone(&self.database)
    }

    /// Runs `future` on the runtime and waits for it on this thread.
    pub(crate) fn run<F, R>(&self, future: F) -> Result<R, WorkspaceError>
    where
        F: Future<Output = R> + Send + 'static,
        R: Send + 'static,
    {
        match &self.executor {
            Executor::Own(own) => {
                let handle = own.get()?.handle();
                futures::executor::block_on(async {
                    handle.spawn(future).await.map_err(|e| e.into())
                })
            }
            Executor::Injected(handle) => {
                let _context = handle.enter();
                Ok(futures::executor::block_on(future))
            }
        }
    }

    /// Like [`Self::run`], for a future that winds down once `stop` is set, like one that stops
    /// starting new work. While it runs, `should_stop` is checked on this thread, before it
    /// starts and every [`STOP_CHECK_PERIOD`], so that it can read what is local to the thread,
    /// like the request's cancellation. Once it returns `true`, `stop` is set.
    pub(crate) fn run_until<F, R>(
        &self,
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
        let handle = match &self.executor {
            Executor::Own(own) => own.get()?.handle(),
            Executor::Injected(_) => {
                let result = self.run(future);
                if should_stop() {
                    stop.store(true, Ordering::Relaxed);
                }
                return result;
            }
        };
        let (done, finished) = mpsc::channel::<()>();
        let task = handle.spawn(async move {
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
}

/// The runtime the workspace creates itself. Its workers type check statements, which walks
/// their parse trees and parses them with tree-sitter, so they get the stack size of the language
/// server's threads. Creating it fails if the OS can't start its threads: the request that tried
/// reports that, and the next request tries again.
#[derive(Default)]
struct OwnRuntime {
    runtime: OnceLock<Runtime>,
    /// Held while the runtime is created, so that only one request creates it.
    init: Mutex<()>,
}

impl OwnRuntime {
    fn get(&self) -> Result<&Runtime, WorkspaceError> {
        self.get_or_build(Self::build)
    }

    fn build() -> io::Result<Runtime> {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_stack_size(WORKER_STACK_SIZE_BYTES)
            .build()
    }

    /// Returns the runtime, and creates it with `build` if no earlier call did.
    ///
    /// Tokio's `Builder::build` panics instead of returning an error when the OS can't spawn a
    /// worker thread. The panic is reported as an error, and the runtime stays unset, so the next
    /// call tries again instead of failing for the rest of the session.
    fn get_or_build(
        &self,
        build: impl FnOnce() -> io::Result<Runtime>,
    ) -> Result<&Runtime, WorkspaceError> {
        if let Some(runtime) = self.runtime.get() {
            return Ok(runtime);
        }
        let _init = self.init.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(runtime) = self.runtime.get() {
            return Ok(runtime);
        }
        let runtime = match panic::catch_unwind(AssertUnwindSafe(build)) {
            Ok(Ok(runtime)) => runtime,
            Ok(Err(err)) => return Err(Self::error(&err.to_string())),
            Err(payload) => return Err(Self::error(Self::panic_message(payload.as_ref()))),
        };
        Ok(self.runtime.get_or_init(|| runtime))
    }

    fn error(reason: &str) -> WorkspaceError {
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
}

impl Drop for OwnRuntime {
    /// Drops the runtime's tasks without waiting for them. The workspace may be dropped on a
    /// thread of an async runtime, where waiting would panic.
    fn drop(&mut self) {
        if let Some(runtime) = self.runtime.take() {
            runtime.shutdown_background();
        }
    }
}

/// The clock of the workspace's backoffs and idle timeouts: the system's, or the one of an
/// injected runtime, which a test can pause and advance.
pub(crate) struct Clock {
    /// The runtime whose clock this is, if it was injected.
    runtime: Option<Handle>,
    /// When the clock was created, see [`Clock::elapsed`].
    epoch: Instant,
}

impl Clock {
    fn system() -> Self {
        Self {
            runtime: None,
            epoch: Instant::now(),
        }
    }

    fn of(handle: &Handle) -> Self {
        let _context = handle.enter();
        Self {
            runtime: Some(handle.clone()),
            epoch: tokio::time::Instant::now().into_std(),
        }
    }

    pub(crate) fn now(&self) -> Instant {
        match &self.runtime {
            Some(runtime) => {
                let _context = runtime.enter();
                tokio::time::Instant::now().into_std()
            }
            None => Instant::now(),
        }
    }

    /// How long ago the clock was created.
    pub(crate) fn elapsed(&self) -> Duration {
        self.now().saturating_duration_since(self.epoch)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        hint::black_box,
        sync::{
            Arc,
            atomic::{AtomicBool, AtomicUsize, Ordering},
        },
        time::Duration,
    };

    use super::{OwnRuntime, WorkspaceRuntime};
    use crate::WorkspaceError;
    use crate::workspace::server::database::Postgres;

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
        let used = WorkspaceRuntime::new()
            .run(async {
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
        let counted = WorkspaceRuntime::new()
            .run_until(
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
        let started = WorkspaceRuntime::new()
            .run_until(
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
        let own = OwnRuntime::default();

        // The OS can't map a 1 PiB stack, so spawning the first worker thread fails.
        let failed = own.get_or_build(|| {
            tokio::runtime::Builder::new_multi_thread()
                .thread_stack_size(1 << 50)
                .build()
        });
        assert!(matches!(failed, Err(WorkspaceError::RuntimeError(_))));
        assert!(own.runtime.get().is_none());

        let runtime = own.get().unwrap();
        assert_eq!(runtime.block_on(async { 1 }), 1);
    }

    /// The workspace, and with it its runtime, may be dropped on a thread of the language
    /// server's runtime.
    #[tokio::test]
    async fn the_own_runtime_can_be_dropped_in_an_async_context() {
        let runtime = WorkspaceRuntime::new();
        assert_eq!(runtime.run(async { 1 }).unwrap(), 1);
        drop(runtime);
    }

    /// An injected runtime runs the futures, and its clock is the one of the backoffs and idle
    /// timeouts, so a test can pause it and move it forward.
    #[test]
    fn an_injected_runtime_runs_the_futures_on_its_clock() {
        let test_runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .start_paused(true)
            .build()
            .unwrap();
        let runtime =
            WorkspaceRuntime::with_handle(test_runtime.handle().clone(), Arc::new(Postgres));
        assert_eq!(runtime.clock().elapsed(), Duration::ZERO);

        test_runtime.block_on(async {
            tokio::time::advance(Duration::from_secs(90)).await;
            // The workspace is called from a blocking thread, while the runtime's own thread
            // waits for it.
            let ran = tokio::task::spawn_blocking(move || {
                let ran = runtime.run(async { tokio::time::sleep(Duration::ZERO).await });
                (ran, runtime.clock().elapsed())
            })
            .await
            .unwrap();
            assert!(ran.0.is_ok());
            assert_eq!(ran.1, Duration::from_secs(90));
        });
    }
}
