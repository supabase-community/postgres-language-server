//! What the server is still doing after it answered, so that a test can wait until it's idle.
//!
//! As in gopls' integration tests (`gopls/internal/test/integration/expectation.go`), a test
//! client counts the notifications it sends and waits until the server handled as many and
//! finished the work they started, with a timeout on the real clock. rust-analyzer's and ty's
//! tests wait for the server's own idle report the same way.
//!
//! Only the `testing` feature counts. Without it, the types are empty and counting compiles to
//! nothing.

#[cfg(feature = "testing")]
use std::sync::Arc;
#[cfg(feature = "testing")]
use std::time::Duration;

#[cfg(feature = "testing")]
use tokio::sync::watch;

/// Counts the work of a connection that outlives the message that started it: the work on the
/// blocking thread pool, the debounced diagnostics and configuration reloads, and the requests
/// to the client that the server doesn't wait for. It also counts the notifications the server
/// handled and the diagnostics it published.
#[derive(Clone, Default)]
pub struct Activity {
    #[cfg(feature = "testing")]
    progress: Arc<watch::Sender<Progress>>,
}

/// What [Activity] counted so far.
#[cfg(feature = "testing")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Progress {
    /// The jobs in flight.
    pub jobs: usize,
    /// The notifications the server handled, once the jobs they started were counted.
    pub notifications: u64,
    /// The `textDocument/publishDiagnostics` notifications the server sent.
    pub diagnostics_published: u64,
}

impl Activity {
    /// Starts a job, which lasts until the returned guard is dropped. Move it into the task that
    /// does the work, so that the job ends when the task finishes or is aborted.
    pub(crate) fn job(&self) -> Job {
        #[cfg(feature = "testing")]
        self.progress
            .send_modify(|progress| progress.jobs = progress.jobs.saturating_add(1));
        Job {
            #[cfg(feature = "testing")]
            activity: self.clone(),
        }
    }

    /// Counts a notification as handled when the returned guard is dropped. Take it first thing
    /// in the handler, so that it counts after the jobs the handler starts.
    pub(crate) fn notification(&self) -> HandledNotification {
        HandledNotification {
            #[cfg(feature = "testing")]
            activity: self.clone(),
        }
    }

    /// Counts diagnostics that the server published.
    pub(crate) fn diagnostics_published(&self) {
        #[cfg(feature = "testing")]
        self.progress.send_modify(|progress| {
            progress.diagnostics_published = progress.diagnostics_published.saturating_add(1);
        });
    }

    /// What was counted so far.
    #[cfg(feature = "testing")]
    pub fn progress(&self) -> Progress {
        *self.progress.borrow()
    }

    /// Waits until the server handled `notifications` notifications and no job is in flight, at
    /// most `timeout` on the real clock, and returns what was counted then.
    #[cfg(feature = "testing")]
    pub async fn wait_until_idle(
        &self,
        notifications: u64,
        timeout: Duration,
    ) -> anyhow::Result<Progress> {
        let mut receiver = self.progress.subscribe();
        let idle = async {
            receiver
                .wait_for(|progress| progress.jobs == 0 && progress.notifications >= notifications)
                .await
                .map(|progress| *progress)
        };
        match tokio::time::timeout(timeout, idle).await {
            Ok(Ok(progress)) => Ok(progress),
            // `self` holds the sender, so the channel stays open.
            Ok(Err(closed)) => Err(closed.into()),
            Err(_) => anyhow::bail!(
                "the server isn't idle after {timeout:?}, waiting for {notifications} \
                 notifications: {:?}",
                self.progress()
            ),
        }
    }
}

/// A job of [Activity], which ends when this is dropped.
pub(crate) struct Job {
    #[cfg(feature = "testing")]
    activity: Activity,
}

#[cfg(feature = "testing")]
impl Drop for Job {
    fn drop(&mut self) {
        self.activity
            .progress
            .send_modify(|progress| progress.jobs = progress.jobs.saturating_sub(1));
    }
}

/// A notification that [Activity] counts as handled when this is dropped.
pub(crate) struct HandledNotification {
    #[cfg(feature = "testing")]
    activity: Activity,
}

#[cfg(feature = "testing")]
impl Drop for HandledNotification {
    fn drop(&mut self) {
        self.activity.progress.send_modify(|progress| {
            progress.notifications = progress.notifications.saturating_add(1);
        });
    }
}

#[cfg(all(test, feature = "testing"))]
mod tests {
    use std::time::Duration;

    use super::{Activity, Progress};

    #[tokio::test]
    async fn idle_once_the_notifications_are_handled_and_the_jobs_ended() {
        let activity = Activity::default();
        let handled = activity.notification();
        let job = activity.job();
        drop(handled);
        assert_eq!(
            activity.progress(),
            Progress {
                jobs: 1,
                notifications: 1,
                diagnostics_published: 0
            }
        );

        let waiting = tokio::spawn({
            let activity = activity.clone();
            async move { activity.wait_until_idle(1, Duration::from_secs(30)).await }
        });
        activity.diagnostics_published();
        drop(job);
        let progress = waiting.await.unwrap().unwrap();
        assert_eq!(progress.jobs, 0);
        assert_eq!(progress.diagnostics_published, 1);
    }

    #[tokio::test]
    async fn a_job_that_doesnt_end_times_out() {
        let activity = Activity::default();
        let _job = activity.job();
        let waited = activity.wait_until_idle(0, Duration::from_millis(10)).await;
        assert!(waited.is_err());
    }
}
