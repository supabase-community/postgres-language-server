use tokio::task::JoinHandle;

/// Where the language server runs the work that blocks the thread that runs it: the workspace's
/// methods and the search for configuration files.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Blocking {
    /// On tokio's blocking thread pool, so that the runtime keeps handling messages meanwhile.
    #[default]
    ThreadPool,
    /// As a task of the runtime. On a current-thread runtime, that runs the work on the runtime's
    /// thread, one after the other, in an order that only depends on the client's messages and
    /// the runtime's clock, so that a simulated client can replay a session.
    ///
    /// The work then must not wait for the runtime, e.g. for a database query of a workspace that
    /// runs its queries on the same runtime.
    Inline,
}

impl Blocking {
    /// Runs `work` and returns its result, or the panic it raised, like
    /// [`tokio::task::spawn_blocking`].
    pub(crate) fn spawn<F, R>(self, work: F) -> JoinHandle<R>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        match self {
            Self::ThreadPool => tokio::task::spawn_blocking(work),
            Self::Inline => tokio::task::spawn(async move { work() }),
        }
    }
}
