//! What the walk over a document leaves for the next requests: states to resume it from, the
//! references of the statements it resolved, and the database snapshot as one statement sees
//! it.
//!
//! All of it depends on the text before its top-level statement, and the resolved references
//! also on the statement itself. An edit keeps what comes before it, see [`ContentChange`]. It
//! also depends on the migration sections before it, and on whether the queries it is in run in
//! an implicit transaction block, which text after it can change, see [`MigrationContext`].

use std::{collections::BTreeMap, sync::Arc};

use pgls_analyser::{AnalysableStatement, Direction, MigrationSection, WalkState};
use pgls_catalog::{Snapshot, resolve::Reference};
use pgls_text_size::{TextRange, TextSize};

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

/// The migration sections of a document, and the queries that run in an implicit transaction
/// block, as boundaries in file order. A walk to a statement depends on the boundaries before
/// it. Text after the statement can change them: a `StatementEnd` after it puts it in a query,
/// a statement added to its query makes the query run in an implicit block, and goose's
/// `NO TRANSACTION` applies to the whole file.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct MigrationContext {
    /// Ordered by offset.
    boundaries: Vec<Boundary>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Boundary {
    Section {
        start: TextSize,
        down: bool,
        transaction: bool,
    },
    ImplicitBlockStart(TextSize),
    ImplicitBlockEnd(TextSize),
}

impl Boundary {
    fn offset(&self) -> TextSize {
        match *self {
            Boundary::Section { start, .. } => start,
            Boundary::ImplicitBlockStart(offset) | Boundary::ImplicitBlockEnd(offset) => offset,
        }
    }
}

impl MigrationContext {
    /// The context of `sections`. `statement_ranges` are the ranges of the top-level statements
    /// of the document.
    ///
    /// Cost: O(log statements) per query of the sections.
    pub(crate) fn new(sections: &[MigrationSection], statement_ranges: &[TextRange]) -> Self {
        let mut boundaries: Vec<Boundary> = sections
            .iter()
            .flat_map(|section| {
                std::iter::once(Boundary::Section {
                    start: section.range.start(),
                    down: section.direction == Direction::Down,
                    transaction: section.transaction,
                })
                .chain(section.implicit_blocks(statement_ranges).flat_map(
                    |block| {
                        [
                            Boundary::ImplicitBlockStart(block.start()),
                            Boundary::ImplicitBlockEnd(block.end()),
                        ]
                    },
                ))
            })
            .collect();
        boundaries.sort_unstable_by_key(|boundary| (boundary.offset(), *boundary));
        Self { boundaries }
    }

    /// The first offset from which the two contexts can differ: they have the same boundaries
    /// before it. `None` if they are the same.
    fn first_difference(&self, other: &Self) -> Option<TextSize> {
        let same = self
            .boundaries
            .iter()
            .zip(&other.boundaries)
            .take_while(|(this, other)| this == other)
            .count();
        self.boundaries
            .get(same)
            .into_iter()
            .chain(other.boundaries.get(same))
            .map(Boundary::offset)
            .min()
    }
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
    /// The context of the document that the entries were computed in.
    context: MigrationContext,
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

    /// Takes out the entries that `context` invalidates, and returns them: those of the
    /// statements from the first boundary where it differs from the context of the entries.
    pub(crate) fn set_context(&mut self, context: &MigrationContext) -> WalkCache {
        let Some(changed) = self.context.first_difference(context) else {
            return WalkCache::default();
        };
        self.context = context.clone();
        let (checkpoints, invalid) = std::mem::take(&mut self.checkpoints)
            .into_iter()
            .partition(|checkpoint| checkpoint.at() < changed);
        self.checkpoints = checkpoints;
        WalkCache {
            checkpoints: invalid,
            resolved: self.resolved.split_off(&changed),
            snapshot: self.snapshot.take_if(|cached| cached.at >= changed),
            context: MigrationContext::default(),
        }
    }

    /// Takes out what `change` invalidates, and returns it. A checkpoint, and the snapshot of a
    /// top-level statement, stay valid while the text before their statement and the statements
    /// before it are the same. The resolved statements, and the snapshot of a function body,
    /// also need their statement to be the same.
    pub(crate) fn content_changed(&mut self, change: ContentChange) -> WalkCache {
        let before_statement =
            |at: TextSize| at <= change.unchanged && at <= change.first_changed_statement;
        let (checkpoints, invalid) = std::mem::take(&mut self.checkpoints)
            .into_iter()
            .partition(|checkpoint| before_statement(checkpoint.at()));
        self.checkpoints = checkpoints;
        let resolved = self.resolved.split_off(&change.first_changed_statement);
        let snapshot = self.snapshot.take_if(|cached| {
            if cached.in_body {
                cached.at >= change.first_changed_statement
            } else {
                !before_statement(cached.at)
            }
        });
        WalkCache {
            checkpoints: invalid,
            resolved,
            snapshot,
            context: MigrationContext::default(),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.checkpoints.is_empty() && self.resolved.is_empty() && self.snapshot.is_none()
    }

    /// The number of resolved statements and checkpoints.
    #[cfg(test)]
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
