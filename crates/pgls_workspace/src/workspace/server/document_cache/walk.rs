//! What the walk over a document leaves for the next requests: states to resume it from, the
//! references of the statements it resolved, and the database snapshot as one statement sees
//! it.
//!
//! All of it depends on the text before its top-level statement, and the resolved references
//! also on the statement itself. An edit keeps what comes before it, see [`ContentChange`].

use std::{collections::BTreeMap, sync::Arc};

use pgls_analyser::{AnalysableStatement, MigrationSection, WalkState};
use pgls_catalog::{Snapshot, resolve::Reference};
use pgls_text_size::TextSize;

/// The states a document keeps to resume walks from. Each holds a copy of the catalog's maps of
/// the objects that the file changed before it (the objects themselves are shared), so they are
/// few. A request records two: at its statement, and at the statement of the last edit, which
/// covers typing in a statement with requests after it.
const WALK_CHECKPOINTS_MAX: usize = 4;

/// A statement with the references that the walk resolved in it.
pub(crate) struct ResolvedStatement {
    pub(crate) stmt: AnalysableStatement,
    pub(crate) references: Vec<Reference>,
}

/// How an edit changed the document, as far as the walk is concerned.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ContentChange {
    /// The length of the text that is the same before and after the edit.
    pub(crate) unchanged: TextSize,
    /// The start of the first top-level statement, before the edit, that isn't the same after
    /// it, or that doesn't end before the change. All statements before it are unchanged.
    pub(crate) first_changed_statement: TextSize,
}

#[derive(Default)]
pub(crate) struct WalkCache {
    /// Least recently used first.
    checkpoints: Vec<WalkState>,
    /// The statements of each top-level statement that the walk resolved, by its start: the
    /// top-level statement and those of its function body. Empty for a statement that doesn't
    /// parse, or that has identifier parameters.
    resolved: BTreeMap<TextSize, Arc<[ResolvedStatement]>>,
    /// The database snapshot as the statements of one top-level statement see it.
    snapshot: Option<SnapshotBefore>,
}

struct SnapshotBefore {
    /// The start of the top-level statement.
    at: TextSize,
    /// Whether it is the snapshot of the statements of its function body, which also see the
    /// function itself.
    in_body: bool,
    snapshot: Arc<Snapshot>,
}

impl WalkCache {
    /// The last checkpoint at or before `at` that fits `sections`.
    pub(crate) fn checkpoint_before(
        &mut self,
        at: TextSize,
        sections: &[MigrationSection],
    ) -> Option<WalkState> {
        let (index, _) = self
            .checkpoints
            .iter()
            .enumerate()
            .filter(|(_, checkpoint)| checkpoint.at() <= at && checkpoint.fits(sections))
            .max_by_key(|(_, checkpoint)| checkpoint.at())?;
        // Most recently used last.
        let checkpoint = self.checkpoints.remove(index);
        self.checkpoints.push(checkpoint.clone());
        Some(checkpoint)
    }

    /// Keeps `state`, replacing a checkpoint at the same statement, and drops the least
    /// recently used one if there are too many.
    pub(crate) fn record_checkpoint(&mut self, state: WalkState) {
        self.checkpoints
            .retain(|checkpoint| checkpoint.at() != state.at());
        if self.checkpoints.len() == WALK_CHECKPOINTS_MAX {
            self.checkpoints.remove(0);
        }
        self.checkpoints.push(state);
    }

    /// The resolved statements of the top-level statement that starts at `at`.
    pub(crate) fn resolved(&self, at: TextSize) -> Option<Arc<[ResolvedStatement]>> {
        self.resolved.get(&at).cloned()
    }

    pub(crate) fn record_resolved(&mut self, at: TextSize, statements: Arc<[ResolvedStatement]>) {
        self.resolved.insert(at, statements);
    }

    /// The snapshot as the statements of the top-level statement at `at` see it, or the
    /// statements of its function body if `in_body`.
    pub(crate) fn snapshot(&self, at: TextSize, in_body: bool) -> Option<Arc<Snapshot>> {
        self.snapshot
            .as_ref()
            .filter(|cached| cached.at == at && cached.in_body == in_body)
            .map(|cached| Arc::clone(&cached.snapshot))
    }

    pub(crate) fn record_snapshot(&mut self, at: TextSize, in_body: bool, snapshot: Arc<Snapshot>) {
        self.snapshot = Some(SnapshotBefore {
            at,
            in_body,
            snapshot,
        });
    }

    /// Drops what `change` invalidates. A checkpoint, and the snapshot of a top-level statement,
    /// stay valid while the text before their statement and the statements before it are the
    /// same. The resolved statements, and the snapshot of a function body, also need their
    /// statement to be the same.
    pub(crate) fn content_changed(&mut self, change: ContentChange) {
        let before_statement =
            |at: TextSize| at <= change.unchanged && at <= change.first_changed_statement;
        self.checkpoints
            .retain(|checkpoint| before_statement(checkpoint.at()));
        self.resolved.split_off(&change.first_changed_statement);
        if self.snapshot.as_ref().is_some_and(|cached| {
            if cached.in_body {
                cached.at >= change.first_changed_statement
            } else {
                !before_statement(cached.at)
            }
        }) {
            self.snapshot = None;
        }
    }

    /// The number of resolved statements and checkpoints.
    pub(crate) fn len(&self) -> (usize, usize) {
        (
            self.resolved
                .values()
                .map(|statements| statements.len())
                .sum(),
            self.checkpoints.len(),
        )
    }
}
