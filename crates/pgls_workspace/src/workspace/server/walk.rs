//! The walk over the statements of a document for the editor features: hover, completions,
//! inlay hints, semantic tokens and the expand-`*` action.
//!
//! It is lint's walk, [`pgls_analyser::walk_statements`], resumed from the checkpoints that the
//! document keeps, and what it resolves is kept for the next requests, see
//! [`super::document_cache::WalkCache`]. A request that only needs what is cached doesn't walk.

#![warn(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use std::sync::Arc;

use pgls_analyser::{AnalysableStatement, AnalysedFileContext, STOP_CHECK_INTERVAL, WalkParams};
use pgls_catalog::{
    Catalog, CatalogBase, Origin, Snapshot,
    resolve::{ColumnSource, Reference, ResolveParams, resolve},
};
use pgls_fs::PgLSPath;
use pgls_text_size::{TextRange, TextSize};

use super::{
    WorkspaceServer,
    document::{AnalyserDiagnosticsMapper, Document, DocumentStatement},
    document_cache::{CacheKey, ResolvedStatement},
    document_sections,
};
use crate::settings::Settings;

/// What an editor request on a document walks with.
pub(super) struct EditorRequest<'a> {
    /// The path of the document, to check whether it changed while the request runs.
    pub(super) path: &'a PgLSPath,
    /// The settings of the document.
    pub(super) settings: Option<&'a Settings>,
    /// The database snapshot the catalog starts from.
    pub(super) snapshot: Arc<Snapshot>,
    pub(super) key: CacheKey,
}

impl WorkspaceServer {
    /// What a request of `project_key` on a document with `settings` walks with, starting from
    /// the database `snapshot`.
    pub(super) fn editor_request<'a>(
        &self,
        project_key: crate::workspace::ProjectKey,
        path: &'a PgLSPath,
        settings: Option<&'a Settings>,
        snapshot: Arc<Snapshot>,
    ) -> EditorRequest<'a> {
        EditorRequest {
            path,
            settings,
            key: self.cache_key(project_key, Some(&snapshot)),
            snapshot,
        }
    }

    /// The indexed database snapshot, built once per snapshot.
    pub(super) fn catalog_base(&self, snapshot: &Arc<Snapshot>) -> Arc<CatalogBase> {
        let mut cached = self.catalog_base.lock();
        if let Some(base) = cached.as_ref().filter(|base| base.is_built_from(snapshot)) {
            return Arc::clone(base);
        }
        let base = Arc::new(CatalogBase::new(Arc::clone(snapshot)));
        *cached = Some(Arc::clone(&base));
        base
    }

    /// Walks the statements of `doc` that start before `end`. The walk resumes from the last
    /// checkpoint before the top-level statement that contains `start` or comes after it, and
    /// records checkpoints there and at the statement where the last edit starts, so that the
    /// next keystroke there resumes close to it. `visit` gets each statement with the file
    /// context as it is before it. Returns the catalog as a statement at `end` sees it, and the
    /// statements that were walked. `None` if the walk stopped because the document changed or
    /// the request was cancelled; it then records nothing.
    ///
    /// Cost: O(statements between the checkpoint and `end`), plus a copy of the catalog's maps
    /// for each checkpoint recorded.
    fn walk(
        &self,
        request: &EditorRequest,
        doc: &Document,
        start: TextSize,
        end: TextSize,
        visit: impl FnMut(&AnalysableStatement, &AnalysedFileContext),
    ) -> Option<(Catalog, Vec<AnalysableStatement>)> {
        let should_stop = self.should_stop(request.path, doc);
        let (search_path, sections) = request
            .settings
            .map(|settings| {
                (
                    settings.typecheck.search_path.clone(),
                    document_sections(settings, doc),
                )
            })
            .unwrap_or_default();

        let ranges = doc.statement_ranges();
        let request_at = ranges
            .get(ranges.partition_point(|range| range.end() <= start))
            .map_or(end, |range| range.start())
            .min(end);
        let edit_at = doc
            .edited_at()
            .and_then(|edited_at| {
                let after = ranges.partition_point(|range| range.start() <= edited_at);
                Some(ranges.get(after.checked_sub(1)?)?.start())
            })
            .filter(|edit_at| *edit_at < request_at);
        let checkpoints: Vec<TextSize> = edit_at.into_iter().chain([request_at]).collect();
        let from = doc
            .cache()
            .with_walk(&request.key, doc.generation(), |walk| {
                walk.checkpoint_before(request_at, &sections)
            });

        let mut stmts = Vec::new();
        let mut unparsable = Vec::new();
        let walk_start = from.as_ref().map_or(TextSize::from(0), |from| from.at());
        for (i, (stmt, _, skipped)) in doc
            .iter_from(AnalyserDiagnosticsMapper, walk_start)
            .enumerate()
        {
            if (i + 1).is_multiple_of(STOP_CHECK_INTERVAL) && should_stop() {
                return None;
            }
            let Some(stmt_start) = stmt
                .as_ref()
                .map(|stmt| stmt.range.start())
                .or_else(|| skipped.as_ref().map(|skipped| skipped.range.start()))
            else {
                continue;
            };
            if stmt_start >= end {
                break;
            }
            stmts.extend(stmt);
            unparsable.extend(skipped);
        }
        #[cfg(test)]
        self.walked_statements.fetch_add(
            stmts.len() + unparsable.len(),
            std::sync::atomic::Ordering::Relaxed,
        );

        let walk = pgls_analyser::walk_statements(
            WalkParams {
                stmts: &stmts,
                unparsable: &unparsable,
                catalog_base: Some(self.catalog_base(&request.snapshot)),
                search_path: &request.snapshot.expand_search_path(&search_path),
                sections: &sections,
                end,
                // Editor features show what names most likely refer to, so a `DO` block or a
                // statement that doesn't parse doesn't turn them off for the rest of the file.
                best_effort: true,
                from,
                checkpoints: &checkpoints,
                should_stop: &should_stop,
            },
            visit,
        );
        if walk.stopped {
            return None;
        }
        doc.cache()
            .with_walk(&request.key, doc.generation(), |cache| {
                for state in walk.checkpoints {
                    cache.record_checkpoint(state);
                }
            });
        Some((walk.catalog, stmts))
    }

    /// The statements of the top-level statements that intersect `range` and start before its
    /// end (or at it, for an empty range), with their references, top-level statement first. Statements that don't parse and statements with
    /// identifier parameters, whose names are placeholders, have none.
    ///
    /// Cost: what is cached is looked up, O(log statements) each. Otherwise a walk from the
    /// last checkpoint before the first statement that isn't cached to the end of `range`, see
    /// [`Self::walk`].
    pub(super) fn resolved_statements(
        &self,
        request: &EditorRequest,
        doc: &Document,
        range: TextRange,
    ) -> Vec<Arc<[ResolvedStatement]>> {
        let ranges = doc.statement_ranges();
        let first = ranges.partition_point(|statement| statement.end() < range.start());
        // Like a walk to the end of `range`, which visits the statements that start before it, or
        // at it if `range` is empty.
        let last = ranges.partition_point(|statement| {
            statement.start() < range.end() || statement.start() <= range.start()
        });
        let tops = ranges.get(first..last).unwrap_or_default();
        let mut cached: Vec<Option<Arc<[ResolvedStatement]>>> =
            doc.cache()
                .with_walk(&request.key, doc.generation(), |walk| {
                    tops.iter().map(|top| walk.resolved(top.start())).collect()
                });
        let Some(missing_start) = tops
            .iter()
            .zip(&cached)
            .find_map(|(top, cached)| cached.is_none().then(|| top.start()))
        else {
            return cached.into_iter().flatten().collect();
        };
        let end = tops.last().map_or(range.end(), |top| top.end());
        // The index in `tops` of the top-level statement of a statement.
        let top_of = |stmt: &AnalysableStatement| {
            tops.partition_point(|top| top.start() <= stmt.range.start())
                .checked_sub(1)
        };
        let mut references = Vec::new();
        let walked = self.walk(request, doc, missing_start, end, |stmt, file_context| {
            let Some(sql) = stmt.sql.as_deref() else {
                return;
            };
            if stmt.has_identifier_parameters
                || top_of(stmt)
                    .and_then(|top| cached.get(top))
                    .is_none_or(Option::is_some)
            {
                return;
            }
            references.push((
                stmt.range,
                resolve(ResolveParams {
                    stmt: &stmt.root,
                    catalog: file_context.catalog(),
                    search_path: file_context.session().search_path(),
                    function: stmt.function.as_ref(),
                    sql: Some(sql),
                    collect_references: true,
                })
                .references,
            ));
        });

        // An outdated or cancelled request has no result.
        let Some((_, stmts)) = walked else {
            return Vec::new();
        };
        // The statements were visited in order, so the references are in the same order.
        let mut resolved: Vec<Vec<ResolvedStatement>> = tops.iter().map(|_| Vec::new()).collect();
        let mut references = references.into_iter().peekable();
        for stmt in stmts {
            let Some((_, stmt_references)) =
                references.next_if(|(stmt_range, _)| *stmt_range == stmt.range)
            else {
                continue;
            };
            if let Some(resolved) = top_of(&stmt).and_then(|top| resolved.get_mut(top)) {
                resolved.push(ResolvedStatement {
                    stmt,
                    references: stmt_references,
                });
            }
        }
        doc.cache()
            .with_walk(&request.key, doc.generation(), |walk| {
                for ((top, statements), cached) in tops.iter().zip(resolved).zip(&mut cached) {
                    if cached.is_none() {
                        let statements: Arc<[ResolvedStatement]> = statements.into();
                        walk.record_resolved(top.start(), Arc::clone(&statements));
                        *cached = Some(statements);
                    }
                }
            });
        cached.into_iter().flatten().collect()
    }

    /// The database snapshot as `stmt` sees it: with the objects that the statements before it
    /// create, change, or drop. The statements of a function body also see the function.
    ///
    /// Cost: a lookup if it was the last one built for the document. Otherwise a walk to the
    /// statement, see [`Self::walk`], and building the snapshot, which is linear in the objects
    /// that the file changed.
    pub(super) fn snapshot_before(
        &self,
        request: &EditorRequest,
        doc: &Document,
        stmt: &DocumentStatement,
    ) -> Arc<Snapshot> {
        let ranges = doc.statement_ranges();
        let top = ranges
            .get(ranges.partition_point(|range| range.end() <= stmt.range.start()))
            .map_or(stmt.range.start(), |range| range.start());
        let in_body = stmt.id.is_child();
        if let Some(snapshot) = doc
            .cache()
            .with_walk(&request.key, doc.generation(), |walk| {
                walk.snapshot(top, in_body)
            })
        {
            return snapshot;
        }
        // An outdated or cancelled request gets the database snapshot. Its result is discarded.
        let Some((catalog, _)) = self.walk(request, doc, top, stmt.range.start(), |_, _| {}) else {
            return Arc::clone(&request.snapshot);
        };
        let snapshot = catalog
            .snapshot()
            .unwrap_or_else(|| Arc::clone(&request.snapshot));
        doc.cache()
            .with_walk(&request.key, doc.generation(), |walk| {
                walk.record_snapshot(top, in_body, Arc::clone(&snapshot))
            });
        snapshot
    }

    /// For hover on `stmt`: the references whose span contains `position`, and the database
    /// snapshot as the statement sees it.
    pub(super) fn hover_references(
        &self,
        request: &EditorRequest,
        doc: &Document,
        stmt: &DocumentStatement,
        position: TextSize,
    ) -> (Vec<Reference>, Arc<Snapshot>) {
        let resolved = self.resolved_statements(request, doc, stmt.range);
        let Some(resolved) = resolved
            .iter()
            .flat_map(|statements| statements.iter())
            .find(|resolved| resolved.stmt.range == stmt.range)
        else {
            // A statement that doesn't parse, or that has identifier parameters.
            return (Vec::new(), self.snapshot_before(request, doc, stmt));
        };
        let position = resolved.stmt.statement_offset(position);
        let (references, others): (Vec<_>, Vec<_>) =
            resolved.references.iter().cloned().partition(|reference| {
                reference
                    .span()
                    .is_some_and(|span| span.contains_inclusive(position))
            });
        // Objects that the file didn't change are the same in the database snapshot.
        let snapshot = if in_database(&references, &others) {
            Arc::clone(&request.snapshot)
        } else {
            self.snapshot_before(request, doc, stmt)
        };
        (references, snapshot)
    }

    /// The references whose span contains `position`, in the innermost statement at `position`
    /// (the body of a SQL function rather than its `CREATE FUNCTION`). Their spans are relative
    /// to the start of that statement, which is returned with them. Empty if the statement's
    /// text isn't the source text, like a function body with escaped quotes, since the spans
    /// would then not point into the document.
    pub(super) fn references_at(
        &self,
        request: &EditorRequest,
        doc: &Document,
        position: TextSize,
    ) -> (TextSize, Vec<Reference>) {
        let resolved = self.resolved_statements(request, doc, TextRange::empty(position));
        let Some(innermost) = resolved
            .iter()
            .flat_map(|statements| statements.iter())
            .rfind(|resolved| resolved.stmt.range.contains(position))
            .filter(|resolved| resolved.stmt.offsets.is_verbatim())
        else {
            return (TextSize::default(), Vec::new());
        };
        let start = innermost.stmt.range.start();
        let position = position - start;
        let references = innermost
            .references
            .iter()
            .filter(|reference| {
                reference
                    .span()
                    .is_some_and(|span| span.contains_inclusive(position))
            })
            .cloned()
            .collect();
        (start, references)
    }
}

/// Whether hover finds what `references` resolve to in the database snapshot: objects that the
/// file didn't change, and columns of CTEs and subqueries. `others` are the other references of
/// the statement.
fn in_database(references: &[Reference], others: &[Reference]) -> bool {
    // The statement references each relation that it takes columns from.
    let unchanged_relation = |schema: &str, name: &str| {
        references.iter().chain(others).any(|reference| {
            matches!(
                reference,
                Reference::Relation { relation, .. }
                    if relation.schema == schema
                        && relation.name == name
                        && relation.origin == Origin::Database
            )
        })
    };
    !references.is_empty()
        && references.iter().all(|reference| match reference {
            Reference::Relation { relation, .. } => relation.origin == Origin::Database,
            Reference::Function { function, .. } => function.origin == Origin::Database,
            Reference::Column {
                source: ColumnSource::Relation { schema, name },
                ..
            } => unchanged_relation(schema, name),
            Reference::Column { .. } => true,
            Reference::Type { type_info, .. } | Reference::EnumLiteral { type_info, .. } => {
                type_info.origin == Origin::Database
            }
            // Parameters belong to a function of the file.
            Reference::Parameter { .. } => false,
            // The columns of a `*` can come from objects that the file changed.
            Reference::Star { .. } => false,
        })
}
