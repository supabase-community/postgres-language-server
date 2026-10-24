//! Cancellation of long requests, like the language server's requests that a client cancels with
//! `$/cancelRequest`.
//!
//! The flag is set for the thread that runs the request, so this only works with the workspace
//! server in the same process. Requests through a workspace transport (the daemon) run to the
//! end.

use std::{
    cell::RefCell,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

thread_local! {
    static CANCELLED: RefCell<Option<Arc<AtomicBool>>> = const { RefCell::new(None) };
}

/// Runs `f` on this thread so that the work it does stops early once `cancelled` is set.
pub fn with_cancellation<T>(cancelled: Arc<AtomicBool>, f: impl FnOnce() -> T) -> T {
    struct Restore(Option<Arc<AtomicBool>>);

    impl Drop for Restore {
        fn drop(&mut self) {
            CANCELLED.with(|current| current.replace(self.0.take()));
        }
    }

    let _restore = Restore(CANCELLED.with(|current| current.replace(Some(cancelled))));
    f()
}

/// Whether the request this thread is running was cancelled.
pub(crate) fn is_cancelled() -> bool {
    CANCELLED.with(|current| {
        current
            .borrow()
            .as_ref()
            .is_some_and(|cancelled| cancelled.load(Ordering::Relaxed))
    })
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };

    use super::{is_cancelled, with_cancellation};

    #[test]
    fn the_flag_applies_while_the_request_runs() {
        assert!(!is_cancelled());
        let outer = Arc::new(AtomicBool::new(false));
        with_cancellation(Arc::clone(&outer), || {
            assert!(!is_cancelled());
            outer.store(true, Ordering::Relaxed);
            assert!(is_cancelled());
            with_cancellation(
                Arc::new(AtomicBool::new(false)),
                || assert!(!is_cancelled()),
            );
            assert!(is_cancelled());
        });
        assert!(!is_cancelled());
    }
}
