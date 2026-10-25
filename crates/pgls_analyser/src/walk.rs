//! The walk over the statements of a file. The analyser and the workspace's hover, completions
//! and inlay hints share it, so that they see the same catalog at each statement.

use std::{ops::Range, sync::Arc};

use pgls_catalog::{Catalog, CatalogBase, Session, TransactionMode};
use pgls_text_size::TextSize;

use crate::{
    AnalysableStatement, AnalysedFileContext, Direction, MigrationSection, UnparsableStatement,
};

pub struct WalkParams<'a> {
    /// The statements that parse, in file order, including the bodies of SQL functions. When the
    /// walk resumes `from` a state, the statements from where it stopped on.
    pub stmts: &'a [AnalysableStatement],
    /// The top-level statements that don't parse, in file order.
    pub unparsable: &'a [UnparsableStatement],
    /// The database snapshot the catalog starts from.
    pub catalog_base: Option<Arc<CatalogBase>>,
    /// Explicit search path at the start of the file.
    pub search_path: &'a [String],
    /// The sections of a migration file, like dbmate's up and down migrations.
    pub sections: &'a [MigrationSection],
    /// The walk visits the statements that start before `end`.
    pub end: TextSize,
    /// Resumes a walk where it stopped, instead of starting at the beginning of the file. The
    /// file context then only has the statements from there on.
    pub from: Option<WalkState>,
    /// Where to record the state of the walk, in file order, see [`Walk::checkpoints`].
    pub checkpoints: &'a [TextSize],
}

/// The state of a walk at the start of a top-level statement: the catalog and the session after
/// the statements before it. It only depends on the text before it, the sections that start
/// before it, and the parameters of the walk.
#[derive(Clone)]
pub struct WalkState {
    at: TextSize,
    /// The start of the migration section that `at` is in, or zero before the first section.
    section_start: TextSize,
    catalog: Catalog,
    session: Session,
    /// In a down section, the catalog after the up sections before it.
    up_catalog: Option<Catalog>,
}

impl WalkState {
    /// Where the walk stopped.
    pub fn at(&self) -> TextSize {
        self.at
    }

    /// Whether `sections` put the place where the walk stopped in the same section as when it
    /// stopped.
    pub fn fits(&self, sections: &[MigrationSection]) -> bool {
        section_start(sections, self.at) == self.section_start
    }
}

/// The start of the section that `offset` is in, or zero before the first section.
fn section_start(sections: &[MigrationSection], offset: TextSize) -> TextSize {
    sections
        .iter()
        .take_while(|section| section.range.start() <= offset)
        .last()
        .map_or(TextSize::from(0), |section| section.range.start())
}

pub struct Walk {
    /// The catalog as a statement at `end` sees it.
    pub catalog: Catalog,
    /// For each of [`WalkParams::checkpoints`] that the walk got to, the state of the walk before
    /// the first statement that starts at or after it. Resuming from it walks the same as walking
    /// from the start of the file, as long as the text before it doesn't change.
    pub checkpoints: Vec<WalkState>,
}

/// Walks the statements like the database runs them: the catalog and the session follow the
/// statements, so that each statement sees the objects that the statements before it create,
/// change or drop. `visit` gets each statement with the file context as it is before it.
///
/// Each migration section runs in a session and transaction of its own. A down section runs after
/// the up sections before it, and its changes don't carry into the next section.
pub fn walk_statements(
    params: WalkParams<'_>,
    mut visit: impl FnMut(&AnalysableStatement, &AnalysedFileContext),
) -> Walk {
    let roots: Vec<&pgls_query::NodeEnum> = params.stmts.iter().map(|s| &*s.root).collect();

    // The catalog after the up sections so far.
    let mut catalog = Catalog::new(params.catalog_base);
    let mut unparsable = params.unparsable.iter().peekable();
    let mut from = params.from;
    // The walk can't record the state before where it resumes.
    let resumes_at = from.as_ref().map_or(TextSize::from(0), |from| from.at);
    let mut checkpoint_at = params
        .checkpoints
        .iter()
        .copied()
        .filter(|at| *at >= resumes_at)
        .peekable();
    let mut checkpoints = Vec::new();
    for unit in migration_units(params.stmts, params.sections) {
        if unit.start >= params.end {
            break;
        }
        if from.as_ref().is_some_and(|from| unit.end <= from.at) {
            continue;
        }
        let is_down = unit
            .section
            .is_some_and(|section| section.direction == Direction::Down);
        let (unit_catalog, session) = match from.take() {
            Some(from) => {
                debug_assert_eq!(from.section_start, unit.start);
                if let Some(up_catalog) = from.up_catalog {
                    catalog = up_catalog;
                }
                while unparsable
                    .next_if(|skipped| skipped.range.start() < from.at)
                    .is_some()
                {}
                (from.catalog, from.session)
            }
            None => {
                let unit_catalog = if is_down {
                    catalog.clone()
                } else {
                    std::mem::replace(&mut catalog, Catalog::new(None))
                };
                let session = Session::new(params.search_path.to_vec()).with_transaction_mode(
                    unit.section
                        .map_or(TransactionMode::Unknown, MigrationSection::transaction_mode),
                );
                (unit_catalog, session)
            }
        };
        let mut file_context =
            AnalysedFileContext::new(&roots[unit.stmts.clone()], unit_catalog, session);
        let state =
            |at: TextSize, file_context: &AnalysedFileContext, catalog: &Catalog| WalkState {
                at,
                section_start: unit.start,
                catalog: file_context.catalog().clone(),
                session: file_context.session().clone(),
                up_catalog: is_down.then(|| catalog.clone()),
            };

        for stmt in &params.stmts[unit.stmts.clone()] {
            if stmt.range.start() >= params.end {
                break;
            }
            while let Some(at) = checkpoint_at.next_if(|at| stmt.range.start() >= *at) {
                skip_unparsable(&mut unparsable, at, &mut file_context);
                checkpoints.push(state(at, &file_context, &catalog));
            }
            skip_unparsable(&mut unparsable, stmt.range.start(), &mut file_context);

            visit(stmt, &file_context);

            // Statements in the bodies of SQL functions don't run. The names of statements with
            // identifier parameters like `:name` are placeholders, so the catalog can't follow
            // their changes.
            let runs = stmt.function.is_none();
            let placeholders = stmt.has_identifier_parameters && !is_plain_query(&stmt.root);
            if runs && placeholders {
                file_context.taint_catalog();
            }
            file_context.next(runs && !placeholders);
        }

        while let Some(at) = checkpoint_at.next_if(|at| *at < unit.end && *at <= params.end) {
            skip_unparsable(&mut unparsable, at, &mut file_context);
            checkpoints.push(state(at, &file_context, &catalog));
        }
        let end = unit.end.min(params.end);
        skip_unparsable(&mut unparsable, end, &mut file_context);
        if params.end < unit.end {
            return Walk {
                catalog: file_context.into_catalog(),
                checkpoints,
            };
        }
        if !is_down {
            catalog = file_context.into_catalog();
        }
    }

    Walk {
        catalog,
        checkpoints,
    }
}

/// Moves past the statements that don't parse and start before `before`. Those that may have
/// changed the catalog taint it.
fn skip_unparsable<'a>(
    unparsable: &mut std::iter::Peekable<impl Iterator<Item = &'a UnparsableStatement>>,
    before: TextSize,
    file_context: &mut AnalysedFileContext,
) {
    while let Some(skipped) = unparsable.next_if(|skipped| skipped.range.start() < before) {
        if may_change_catalog(&skipped.sql) {
            file_context.taint_catalog();
        }
    }
}

/// Statements that run together: those before the first section marker, then each section.
struct MigrationUnit<'a> {
    section: Option<&'a MigrationSection>,
    /// The indices of the statements.
    stmts: Range<usize>,
    start: TextSize,
    /// The end of the last section is the end of all text, so that it contains the end of the
    /// file.
    end: TextSize,
}

fn migration_units<'a>(
    stmts: &[AnalysableStatement],
    sections: &'a [MigrationSection],
) -> Vec<MigrationUnit<'a>> {
    let end_of_text = TextSize::from(u32::MAX);
    let before_sections = sections
        .first()
        .map_or(end_of_text, |section| section.range.start());
    let units = std::iter::once((None, TextSize::from(0), before_sections)).chain(
        sections.iter().enumerate().map(|(i, section)| {
            let end = if i + 1 == sections.len() {
                end_of_text
            } else {
                section.range.end()
            };
            (Some(section), section.range.start(), end)
        }),
    );

    let mut first = 0;
    units
        .map(|(section, start, end)| {
            let last = first + stmts[first..].partition_point(|stmt| stmt.range.start() < end);
            let unit = MigrationUnit {
                section,
                stmts: first..last,
                start,
                end,
            };
            first = last;
            unit
        })
        .collect()
}

/// Whether a statement that doesn't parse may have changed the catalog. Queries, `COPY`,
/// maintenance, cursors, prepared statements, comments and privileges can't create, drop or
/// rename objects, unless they are `SELECT ... INTO` or `EXPLAIN ANALYZE CREATE TABLE AS`. For
/// anything else we can't tell.
fn may_change_catalog(sql: &str) -> bool {
    use pgls_query::protobuf::Token;
    let Ok(scan) = pgls_query::scan(sql) else {
        return true;
    };
    let mut tokens = scan
        .tokens
        .iter()
        .map(|token| token.token())
        .filter(|token| !matches!(token, Token::SqlComment | Token::CComment | Token::Ascii40));
    match tokens.next() {
        Some(
            Token::Insert
            | Token::Update
            | Token::DeleteP
            | Token::Merge
            | Token::Copy
            | Token::Analyse
            | Token::Analyze
            | Token::Vacuum
            | Token::Reindex
            | Token::Cluster
            | Token::Checkpoint
            | Token::LockP
            | Token::Listen
            | Token::Notify
            | Token::Unlisten
            | Token::Show
            | Token::Declare
            | Token::Fetch
            | Token::Move
            | Token::Close
            | Token::Prepare
            | Token::Execute
            | Token::Deallocate
            | Token::Comment
            | Token::Grant
            | Token::Revoke,
        ) => false,
        Some(Token::Select | Token::With | Token::Values | Token::Table | Token::Explain) => {
            tokens.any(|token| matches!(token, Token::Into | Token::Create))
        }
        _ => true,
    }
}

/// Whether the statement only reads or writes rows, without changing the catalog or the
/// session.
pub(crate) fn is_plain_query(root: &pgls_query::NodeEnum) -> bool {
    use pgls_query::NodeEnum;
    match root {
        NodeEnum::SelectStmt(select) => select.into_clause.is_none(),
        NodeEnum::InsertStmt(_)
        | NodeEnum::UpdateStmt(_)
        | NodeEnum::DeleteStmt(_)
        | NodeEnum::MergeStmt(_) => true,
        _ => false,
    }
}
