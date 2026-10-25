//! The walk over the statements of a file. The analyser and the workspace's hover, completions
//! and inlay hints share it, so that they see the same catalog at each statement.

use std::{ops::Range, sync::Arc};

use pgls_catalog::{Catalog, CatalogBase, Session, TransactionMode};
use pgls_text_size::TextSize;

use crate::{
    AnalysableStatement, AnalysedFileContext, Direction, MigrationSection, UnparsableStatement,
};

pub struct WalkParams<'a> {
    /// The statements that parse, in file order, including the bodies of SQL functions.
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
}

/// Walks the statements like the database runs them: the catalog and the session follow the
/// statements, so that each statement sees the objects that the statements before it create,
/// change or drop. `visit` gets each statement with the file context as it is before it.
///
/// Each migration section runs in a session and transaction of its own. A down section runs after
/// the up sections before it, and its changes don't carry into the next section.
///
/// Returns the catalog as a statement at `end` sees it.
pub fn walk_statements(
    params: WalkParams<'_>,
    mut visit: impl FnMut(&AnalysableStatement, &AnalysedFileContext),
) -> Catalog {
    let roots: Vec<&pgls_query::NodeEnum> = params.stmts.iter().map(|s| &*s.root).collect();

    // The catalog after the up sections so far.
    let mut catalog = Catalog::new(params.catalog_base);
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
        let session = Session::new(params.search_path.to_vec()).with_transaction_mode(
            unit.section
                .map_or(TransactionMode::Unknown, MigrationSection::transaction_mode),
        );
        let mut file_context =
            AnalysedFileContext::new(&roots[unit.stmts.clone()], unit_catalog, session);

        for stmt in &params.stmts[unit.stmts.clone()] {
            if stmt.range.start() >= params.end {
                break;
            }
            while let Some(skipped) =
                unparsable.next_if(|skipped| skipped.range.start() < stmt.range.start())
            {
                if may_change_catalog(&skipped.sql) {
                    file_context.taint_catalog();
                }
            }

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

        let end = unit.end.min(params.end);
        while let Some(skipped) = unparsable.next_if(|skipped| skipped.range.start() < end) {
            if may_change_catalog(&skipped.sql) {
                file_context.taint_catalog();
            }
        }
        if params.end < unit.end {
            return file_context.into_catalog();
        }
        if !is_down {
            catalog = file_context.into_catalog();
        }
    }

    catalog
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
