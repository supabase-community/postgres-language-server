//! The statements that are being executed, so that a request can cancel them.

use std::sync::{Mutex, MutexGuard, PoisonError};

use futures::channel::oneshot;

use super::ExecuteStatementParams;

/// The statements that are being executed. An execution registers with [`Self::start`] for as
/// long as it runs, and [`Self::cancel`] cancels it.
#[derive(Default)]
pub(crate) struct RunningStatements {
    registry: Mutex<Registry>,
}

#[derive(Default)]
struct Registry {
    next_id: u64,
    running: Vec<RunningStatement>,
}

struct RunningStatement {
    /// Tells this execution apart from other executions of the same statement.
    id: u64,
    params: ExecuteStatementParams,
    cancel: oneshot::Sender<()>,
}

impl RunningStatements {
    /// Registers an execution of the statement of `params` until the returned guard is dropped.
    /// The receiver fires when [`Self::cancel`] cancels the execution.
    pub(crate) fn start(
        &self,
        params: &ExecuteStatementParams,
    ) -> (RunningStatementGuard<'_>, oneshot::Receiver<()>) {
        let (cancel, cancelled) = oneshot::channel();
        let mut registry = self.lock();
        let id = registry.next_id;
        registry.next_id += 1;
        registry.running.push(RunningStatement {
            id,
            params: params.clone(),
            cancel,
        });
        (
            RunningStatementGuard {
                statements: self,
                id,
            },
            cancelled,
        )
    }

    /// Cancels the executions of the statement of `params`, if any are running.
    pub(crate) fn cancel(&self, params: &ExecuteStatementParams) {
        let cancelled: Vec<RunningStatement> = self
            .lock()
            .running
            .extract_if(.., |running| {
                running.params.path == params.path
                    && running.params.statement_id == params.statement_id
            })
            .collect();
        for running in cancelled {
            // The execution may have finished in the meantime and dropped the receiver.
            let _ = running.cancel.send(());
        }
    }

    /// Whether no statement is being executed.
    #[cfg(test)]
    pub(crate) fn is_empty(&self) -> bool {
        self.lock().running.is_empty()
    }

    fn lock(&self) -> MutexGuard<'_, Registry> {
        // No code panics while it holds the lock, and the guard must not panic in `drop`.
        self.registry.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Removes an execution from the [`RunningStatements`] when dropped, also if the execution
/// panicked.
pub(crate) struct RunningStatementGuard<'a> {
    statements: &'a RunningStatements,
    id: u64,
}

impl Drop for RunningStatementGuard<'_> {
    fn drop(&mut self) {
        self.statements
            .lock()
            .running
            .retain(|running| running.id != self.id);
    }
}

#[cfg(test)]
mod tests {
    use pgls_fs::PgLSPath;

    use super::RunningStatements;
    use crate::features::code_actions::ExecuteStatementParams;
    use crate::workspace::{ProjectKey, StatementId};

    fn params(statement: &str) -> ExecuteStatementParams {
        ExecuteStatementParams {
            project_key: ProjectKey::default(),
            statement_id: StatementId::new(statement),
            path: PgLSPath::new("test.sql"),
        }
    }

    #[test]
    fn cancels_the_executions_of_the_statement() {
        let statements = RunningStatements::default();
        let (_first, mut first) = statements.start(&params("select 1"));
        let (_second, mut second) = statements.start(&params("select 1"));
        let (_other, mut other) = statements.start(&params("select 2"));

        statements.cancel(&params("select 1"));

        assert_eq!(first.try_recv(), Ok(Some(())));
        assert_eq!(second.try_recv(), Ok(Some(())));
        assert_eq!(other.try_recv(), Ok(None));
    }

    #[test]
    fn the_guard_removes_the_execution() {
        let statements = RunningStatements::default();
        let (first, _) = statements.start(&params("select 1"));
        let (second, _) = statements.start(&params("select 1"));

        drop(first);
        assert!(!statements.is_empty());
        drop(second);
        assert!(statements.is_empty());

        // Cancelling a statement that doesn't run does nothing.
        statements.cancel(&params("select 1"));
    }
}
