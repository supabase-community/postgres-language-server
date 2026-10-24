//! The walk of a document for hover, inlay hints, and semantic tokens: the references of its
//! statements, resolved against the catalog as the statements before them leave it.
//!
//! Walking a long file from the start for every request is slow, so the walk is cached per
//! document. Every few statements it keeps a checkpoint of the catalog and the session, and it
//! keeps the references of every statement it resolved. Both are keyed by a hash of the
//! statements before them, so that an edit only invalidates what comes after it. A request for a
//! range of the file then resolves at most the statements since the last checkpoint before it,
//! and a repeated request only looks the references up.

use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::{Arc, Mutex},
};

use pgls_analyser::{AnalysableStatement, Direction, MigrationSection, StatementOffsets};
use pgls_catalog::{
    Catalog, Session, Snapshot, TransactionMode,
    resolve::{Reference, ResolveParams, resolve},
};
use pgls_fs::PgLSPath;
use pgls_text_size::TextRange;
use rustc_hash::{FxHashMap, FxHashSet};

use super::{
    StatementId, WorkspaceServer,
    document::{AnalyserDiagnosticsMapper, Document, IdMapper, StatementMapper},
    document_sections,
    sql_function::get_sql_fn_bodies,
};

/// A statement of the walk with its references.
pub(super) struct WalkedStatement {
    pub id: StatementId,
    /// The range of the statement in the document.
    pub range: TextRange,
    /// Where the text of the statement is in the document.
    pub offsets: StatementOffsets,
    pub references: Arc<[Reference]>,
}

impl WalkedStatement {
    /// The statement text, which the references point into.
    pub fn sql(&self) -> &str {
        self.id.content()
    }
}

/// The cached walk of one document.
#[derive(Default)]
pub(super) struct WalkCache {
    /// The snapshot the walk starts from. Holding it also keeps another snapshot from being
    /// allocated where it is, so comparing the pointers is enough.
    snapshot: Option<Arc<Snapshot>>,
    /// A hash of the settings the walk depends on.
    key: u64,
    /// The state before some of the top-level statements, ordered by statement.
    checkpoints: Vec<Checkpoint>,
    /// The statements walked so far, by [`statement_key`].
    statements: FxHashMap<u64, CachedStatement>,
}

#[derive(Clone)]
struct CachedStatement {
    offsets: StatementOffsets,
    /// `None` for statements that are skipped: those that don't parse, and those with
    /// identifier parameters.
    references: Option<Arc<[Reference]>>,
    /// The statements of the function body of a top-level statement, with their ranges relative
    /// to it.
    children: Arc<[(StatementId, TextRange)]>,
}

/// The state of the walk before a top-level statement.
#[derive(Clone)]
struct Checkpoint {
    /// The index of the statement in [`Document::statement_ranges`].
    root: usize,
    /// The hash of the statements before it, [`WalkHashes::before`].
    hash: u64,
    state: WalkState,
}

#[derive(Clone)]
struct WalkState {
    catalog: Catalog,
    session: Session,
    /// The index of the current migration section.
    section: Option<usize>,
    /// The catalog after the up sections so far, while walking a down section.
    up_catalog: Option<Catalog>,
}

/// The hashes of the top-level statements: of everything before each one, and of everything
/// before it together with the migration section it is in, which decide what it resolves to.
struct WalkHashes {
    before: Vec<u64>,
    with_section: Vec<u64>,
}

impl WalkHashes {
    fn new(doc: &Document, sections: &[MigrationSection], key: u64, roots: usize) -> Self {
        let content = doc.get_document_content();
        let mut before = Vec::with_capacity(roots + 1);
        let mut with_section = Vec::with_capacity(roots);
        let mut hash = key;
        for range in doc.statement_ranges().iter().take(roots) {
            before.push(hash);
            let mut hasher = DefaultHasher::new();
            hash.hash(&mut hasher);
            section_identity(sections, section_of(sections, range.start())).hash(&mut hasher);
            let with = hasher.finish();
            with_section.push(with);
            let mut hasher = DefaultHasher::new();
            with.hash(&mut hasher);
            content[*range].hash(&mut hasher);
            hash = hasher.finish();
        }
        before.push(hash);
        Self {
            before,
            with_section,
        }
    }
}

/// Saves a checkpoint every this many top-level statements, but at most about 32 per document:
/// each holds a copy of the catalog.
fn checkpoint_interval(roots: usize) -> usize {
    (roots / 32).max(64)
}

/// The index of the migration section that contains `offset`.
fn section_of(sections: &[MigrationSection], offset: pgls_text_size::TextSize) -> Option<usize> {
    sections
        .iter()
        .position(|section| section.range.contains(offset))
}

/// What a statement's resolution depends on about its section: not where it is, which changes
/// with every edit before it.
fn section_identity(
    sections: &[MigrationSection],
    section: Option<usize>,
) -> Option<(
    usize,
    std::mem::Discriminant<Direction>,
    std::mem::Discriminant<TransactionMode>,
)> {
    let section = section?;
    let found = sections.get(section)?;
    Some((
        section,
        std::mem::discriminant(&found.direction),
        std::mem::discriminant(&found.transaction_mode()),
    ))
}

/// The key of the references of a statement: the hash of its top-level statement with its
/// section, and the statement itself, whose id includes the function it is the body of.
fn statement_key(root_hash: u64, id: &StatementId) -> u64 {
    let mut hasher = DefaultHasher::new();
    root_hash.hash(&mut hasher);
    id.hash(&mut hasher);
    hasher.finish()
}

impl WorkspaceServer {
    /// The statements of `doc` whose range intersects `range`, with their references, in the
    /// order of the walk: each top-level statement before the statements of its function body.
    ///
    /// The walk is like the analyser's: the catalog and the session follow the statements, so
    /// that each statement sees the objects created earlier in the file. Each migration section
    /// gets a session of its own, and the changes of a down section don't carry into the next
    /// section.
    pub(super) fn walk_references(
        &self,
        path: &PgLSPath,
        doc: &Document,
        snapshot: &Arc<Snapshot>,
        range: TextRange,
    ) -> Vec<WalkedStatement> {
        let (search_path, sections) = self
            .workspaces()
            .settings()
            .map(|settings| {
                (
                    settings.typecheck.search_path.clone(),
                    document_sections(settings, doc),
                )
            })
            .unwrap_or_default();
        let search_path = snapshot.expand_search_path(&search_path);
        let key = {
            let mut hasher = DefaultHasher::new();
            search_path.hash(&mut hasher);
            hasher.finish()
        };

        let cache = Arc::clone(
            self.editor_walks
                .lock()
                .unwrap()
                .entry(path.clone())
                .or_default(),
        );
        let mut cache = cache.lock().unwrap();
        if cache.key != key
            || !cache
                .snapshot
                .as_ref()
                .is_some_and(|cached| Arc::ptr_eq(cached, snapshot))
        {
            *cache = WalkCache {
                snapshot: Some(Arc::clone(snapshot)),
                key,
                ..WalkCache::default()
            };
        }

        let ranges = doc.statement_ranges();
        // The top-level statements that may intersect the range, and those before them.
        let first = ranges.partition_point(|root| root.end() < range.start());
        let end = ranges.partition_point(|root| !is_after(*root, range));
        let hashes = WalkHashes::new(doc, &sections, key, end);

        if let Some(statements) = cached_statements(&cache, doc, &hashes, first, end, range) {
            return statements;
        }

        // Start from the last valid checkpoint before the range.
        let checkpoint = cache
            .checkpoints
            .iter()
            .rev()
            .find(|checkpoint| {
                checkpoint.root <= first
                    && hashes.before.get(checkpoint.root) == Some(&checkpoint.hash)
            })
            .cloned();
        let (start, mut state) = match checkpoint {
            Some(checkpoint) => (checkpoint.root, checkpoint.state),
            None => (
                0,
                WalkState {
                    // Editor features show what names most likely refer to, so a `DO` block
                    // or a statement that doesn't parse doesn't turn them off for the rest of
                    // the file.
                    catalog: Catalog::new(Some(self.catalog_base(snapshot))).best_effort(),
                    session: new_session(&search_path, &sections, None),
                    section: None,
                    up_catalog: None,
                },
            ),
        };

        let interval = checkpoint_interval(ranges.len());
        let mut statements = Vec::new();
        let mut root = start.wrapping_sub(1);
        for (id, stmt_range) in doc.iter_from(IdMapper, start) {
            if is_after(stmt_range, range) {
                break;
            }
            if id.is_root() {
                root = root.wrapping_add(1);
                if root > start && root % interval == 0 {
                    save_checkpoint(&mut cache, root, hashes.before[root], &state);
                }
            }
            let in_range = stmt_range.intersect(range).is_some();
            // The statements of function bodies before the range change nothing.
            if id.is_child() && !in_range {
                continue;
            }

            let current = section_of(&sections, stmt_range.start());
            if current != state.section {
                if let Some(up_catalog) = state.up_catalog.take() {
                    state.catalog = up_catalog;
                }
                if current
                    .and_then(|current| sections.get(current))
                    .is_some_and(|section| section.direction == Direction::Down)
                {
                    state.up_catalog = Some(state.catalog.clone());
                }
                state.session = new_session(&search_path, &sections, current);
                state.section = current;
            }

            let key = statement_key(hashes.with_section[root], &id);
            let (stmt, _, unparsable) = AnalyserDiagnosticsMapper.map(doc, id.clone(), stmt_range);
            let skipped = match (&stmt, &unparsable) {
                (_, Some(unparsable)) => {
                    if pgls_analyser::may_change_catalog(&unparsable.sql) {
                        state.catalog.taint();
                    }
                    true
                }
                (None, None) => true,
                // The names of statements with identifier parameters like `:name` are
                // placeholders.
                (Some(stmt), None) if stmt.has_identifier_parameters => {
                    if !pgls_analyser::is_plain_query(&stmt.root) {
                        state.catalog.taint();
                    }
                    true
                }
                (Some(_), None) => false,
            };
            if in_range {
                let cached = match cache.statements.get(&key) {
                    Some(cached) => cached.clone(),
                    None => {
                        let children = match (&stmt, id.is_root()) {
                            (Some(stmt), true) => get_sql_fn_bodies(&stmt.root, id.content())
                                .into_iter()
                                .map(|body| (id.create_child(&body.body), body.range))
                                .collect(),
                            _ => Arc::default(),
                        };
                        let cached = CachedStatement {
                            offsets: stmt
                                .as_ref()
                                .map(|stmt| stmt.offsets.clone())
                                .unwrap_or_default(),
                            references: stmt
                                .as_ref()
                                .filter(|_| !skipped)
                                .map(|stmt| resolve_statement(stmt, &state).into()),
                            children,
                        };
                        cache.statements.insert(key, cached.clone());
                        cached
                    }
                };
                if let Some(references) = cached.references {
                    statements.push(WalkedStatement {
                        id,
                        range: stmt_range,
                        offsets: cached.offsets,
                        references,
                    });
                }
            }
            let Some(stmt) = stmt.filter(|_| !skipped) else {
                continue;
            };
            // Statements in the bodies of SQL functions are not executed.
            if stmt.function.is_none() {
                state.catalog.apply(&stmt.root, state.session.search_path());
                state.session.apply(&stmt.root);
            }
        }

        prune(&mut cache, doc, &sections, key);
        statements
    }
}

/// Whether the statement starts after the range. A statement that starts where a non-empty range
/// ends is after it.
fn is_after(statement: TextRange, range: TextRange) -> bool {
    statement.start() > range.end() || (statement.start() == range.end() && !range.is_empty())
}

fn new_session(
    search_path: &[String],
    sections: &[MigrationSection],
    section: Option<usize>,
) -> Session {
    Session::new(search_path.to_vec()).with_transaction_mode(
        section
            .and_then(|section| sections.get(section))
            .map_or(TransactionMode::Unknown, MigrationSection::transaction_mode),
    )
}

fn resolve_statement(stmt: &AnalysableStatement, state: &WalkState) -> Vec<Reference> {
    let Some(sql) = stmt.sql.as_deref() else {
        return Vec::new();
    };
    resolve(ResolveParams {
        stmt: &stmt.root,
        catalog: &state.catalog,
        search_path: state.session.search_path(),
        function: stmt.function.as_ref(),
        sql: Some(sql),
        collect_references: true,
    })
    .references
}

/// The statements in `range` from the cache, if all of them are cached.
fn cached_statements(
    cache: &WalkCache,
    doc: &Document,
    hashes: &WalkHashes,
    first: usize,
    end: usize,
    range: TextRange,
) -> Option<Vec<WalkedStatement>> {
    let content = doc.get_document_content();
    let mut statements = Vec::new();
    for (root, root_range) in doc
        .statement_ranges()
        .iter()
        .enumerate()
        .take(end)
        .skip(first)
    {
        let id = StatementId::new(&content[*root_range]);
        let cached = cache
            .statements
            .get(&statement_key(hashes.with_section[root], &id))?;
        let children = Arc::clone(&cached.children);
        if let Some(references) = &cached.references {
            statements.push(WalkedStatement {
                id,
                range: *root_range,
                offsets: cached.offsets.clone(),
                references: Arc::clone(references),
            });
        }
        for (child, child_range) in children.iter() {
            let child_range = *child_range + root_range.start();
            if is_after(child_range, range) || child_range.intersect(range).is_none() {
                continue;
            }
            let cached = cache
                .statements
                .get(&statement_key(hashes.with_section[root], child))?;
            if let Some(references) = &cached.references {
                statements.push(WalkedStatement {
                    id: child.clone(),
                    range: child_range,
                    offsets: cached.offsets.clone(),
                    references: Arc::clone(references),
                });
            }
        }
    }
    Some(statements)
}

fn save_checkpoint(cache: &mut WalkCache, root: usize, hash: u64, state: &WalkState) {
    let index = cache
        .checkpoints
        .partition_point(|checkpoint| checkpoint.root < root);
    let checkpoint = Checkpoint {
        root,
        hash,
        state: state.clone(),
    };
    match cache.checkpoints.get_mut(index) {
        Some(existing) if existing.root == root => {
            if existing.hash != hash {
                *existing = checkpoint;
            }
        }
        _ => cache.checkpoints.insert(index, checkpoint),
    }
}

/// Drops the references of statements that are no longer in the document, once there are many
/// more than the document has statements.
fn prune(cache: &mut WalkCache, doc: &Document, sections: &[MigrationSection], key: u64) {
    let roots = doc.statement_ranges().len();
    if cache.statements.len() <= 2 * roots + 1024 {
        return;
    }
    let hashes = WalkHashes::new(doc, sections, key, roots);
    let current: FxHashSet<u64> = doc
        .iter_from(IdMapper, 0)
        .scan(usize::MAX, |root, (id, _)| {
            if id.is_root() {
                *root = root.wrapping_add(1);
            }
            Some(statement_key(hashes.with_section[*root], &id))
        })
        .collect();
    cache.statements.retain(|key, _| current.contains(key));
    cache
        .checkpoints
        .retain(|checkpoint| hashes.before.get(checkpoint.root) == Some(&checkpoint.hash));
}

/// The cached walks of the open documents.
pub(super) type EditorWalks = Mutex<std::collections::HashMap<PgLSPath, Arc<Mutex<WalkCache>>>>;
