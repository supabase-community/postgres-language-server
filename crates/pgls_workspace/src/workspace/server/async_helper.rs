use std::{
    future::Future,
    sync::{
        Arc, LazyLock,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, RecvTimeoutError},
    },
    time::Duration,
};

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
    let (done, finished) = mpsc::channel::<()>();
    let task = RUNTIME.spawn(async move {
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
        sync::{
            Arc,
            atomic::{AtomicBool, AtomicUsize, Ordering},
        },
        time::Duration,
    };

    use super::run_async_until;

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
}
