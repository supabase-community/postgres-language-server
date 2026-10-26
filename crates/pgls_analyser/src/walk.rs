//! The walk over the statements of a file. Lint and the workspace's editor features (hover,
//! completions, inlay hints, semantic tokens, code actions) share it, so that they see the same
//! catalog and session at each statement.

use std::{ops::Range, sync::Arc};

use pgls_catalog::{Catalog, CatalogBase, Session, TransactionMode};
use pgls_text_size::{TextRange, TextSize};

use crate::{
    AnalysableStatement, AnalysedFileContext, Direction, MigrationSection, UnparsableStatement,
};

pub struct WalkParams<'a> {
    /// The statements that parse, in file order, including the statements of SQL function
    /// bodies after their function.
    pub stmts: &'a [AnalysableStatement],
    /// The top-level statements that don't parse, in file order.
    pub unparsable: &'a [UnparsableStatement],
    /// The database snapshot the catalog starts from.
    pub catalog_base: Option<Arc<CatalogBase>>,
    /// Explicit search path at the start of the file.
    pub search_path: &'a [String],
    /// The sections of a migration file, like dbmate's up and down migrations. Without sections,
    /// the whole file is one migration.
    pub sections: &'a [MigrationSection],
    /// The walk visits the statements that start before `end`.
    pub end: TextSize,
    /// Whether the catalog resolves names as well as it can after statements it can't follow,
    /// like a `DO` block, see [`Catalog::best_effort`]. Editor features show what names most
    /// likely refer to; lint only reports what is certain.
    pub best_effort: bool,
}

/// Walks the statements like the database runs them: the catalog and the session follow the
/// statements, so that each statement sees the objects that the statements before it create,
/// change or drop. `visit` gets each statement with the file context as it is before it.
/// Returns the catalog as a statement at [`WalkParams::end`] sees it.
///
/// - Each migration section runs in a session and transaction of its own. A down section runs
///   after the up sections before it, and its changes don't carry into the next section.
/// - A top-level statement that doesn't parse taints the catalog if it may have changed it.
/// - The statements of SQL function bodies are visited, but they don't run.
/// - The names of a statement with identifier parameters, like psql's `:name`, are placeholders.
///   Such a statement is visited, but if it may change the catalog, it taints it instead of being
///   applied.
///
/// The file context only knows the statements that the walk is given. When they stop before the
/// end of the file, [`AnalysedFileContext::stmt_count`] and the implicit transaction blocks of
/// the last query are those of the statements given, which only rules rely on.
pub fn walk_statements(
    params: WalkParams<'_>,
    mut visit: impl FnMut(&AnalysableStatement, &AnalysedFileContext),
) -> Catalog {
    let roots: Vec<&pgls_query::NodeEnum> = params.stmts.iter().map(|s| &*s.root).collect();

    // The catalog after the up sections so far.
    let mut catalog = Catalog::new(params.catalog_base);
    if params.best_effort {
        catalog = catalog.best_effort();
    }
    let mut unparsable = params.unparsable.iter().peekable();
    for unit in migration_units(params.stmts, params.sections) {
        if unit.start >= params.end {
            break;
        }
        let is_down = unit
            .section
            .is_some_and(|section| section.direction == Direction::Down);
        let unit_catalog = if is_down {
            catalog.clone()
        } else {
            std::mem::replace(&mut catalog, Catalog::new(None))
        };
        // Each section runs in a session and transaction of its own.
        let session = Session::new(params.search_path.to_vec()).with_transaction_mode(
            unit.section
                .map_or(TransactionMode::Unknown, MigrationSection::transaction_mode),
        );
        let stmts = &params.stmts[unit.stmts.clone()];
        let unit_range = TextRange::new(unit.start, unit.end);
        let last_end = params
            .unparsable
            .iter()
            .filter(|skipped| unit_range.contains(skipped.range.start()))
            .map(|skipped| skipped.range.end())
            .chain(stmts.last().map(|stmt| stmt.range.end()))
            .max()
            .unwrap_or_default();
        // The statements the unit runs, including those that don't parse, but not those in
        // function bodies.
        let stmt_count = |range: TextRange| {
            stmts
                .iter()
                .filter(|stmt| stmt.function.is_none() && range.contains(stmt.range.start()))
                .count()
                + params
                    .unparsable
                    .iter()
                    .filter(|skipped| range.contains(skipped.range.start()))
                    .count()
        };
        // Outside a transaction, the queries with more than one statement run in an implicit
        // transaction block.
        let implicit_blocks: Vec<TextRange> = unit
            .section
            .filter(|section| !section.transaction)
            .map(|section| {
                section
                    .queries
                    .iter()
                    .copied()
                    .filter(|query| stmt_count(*query) > 1)
                    .collect()
            })
            .unwrap_or_default();
        let mut file_context =
            AnalysedFileContext::new(&roots[unit.stmts.clone()], unit_catalog, session)
                .with_positions(
                    stmts.iter().map(|stmt| stmt.range.start()).collect(),
                    last_end,
                )
                .with_stmt_count(stmt_count(unit_range));
        let mut implicit_block = None;

        for stmt in stmts {
            if stmt.range.start() >= params.end {
                break;
            }
            let runs = stmt.function.is_none();
            if runs {
                let block = implicit_blocks
                    .iter()
                    .position(|block| block.contains(stmt.range.start()));
                if block != implicit_block {
                    if implicit_block.is_some() {
                        file_context.end_implicit_block();
                    }
                    if block.is_some() {
                        file_context.begin_implicit_block();
                    }
                    implicit_block = block;
                }
            }
            skip_unparsable(&mut unparsable, stmt.range.start(), &mut file_context);

            visit(stmt, &file_context);

            let placeholders = stmt.has_identifier_parameters && !is_plain_query(&stmt.root);
            if runs && placeholders {
                file_context.taint_catalog();
            }
            file_context.next(runs && !placeholders);
        }

        skip_unparsable(&mut unparsable, unit.end.min(params.end), &mut file_context);
        // The section that contains `end` decides what a statement there sees.
        if params.end < unit.end {
            return file_context.into_catalog();
        }
        if !is_down {
            catalog = file_context.into_catalog();
        }
    }
    catalog
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
    /// The last unit ends at the end of all text, so that it contains the end of the file.
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
