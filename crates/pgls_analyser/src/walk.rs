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
    /// The ranges of all top-level statements of the file, parsed or not, in order. The walk
    /// counts the statements of a section and of its queries with them, also when it is only
    /// given some of the statements, see [`MigrationSection::implicit_blocks`].
    pub statement_ranges: &'a [TextRange],
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
    /// Resumes a walk where it stopped instead of starting at the beginning of the file. The
    /// statements are then those from [`WalkState::at`] on.
    pub from: Option<WalkState>,
    /// The starts of top-level statements where to record the state of the walk, in file
    /// order, see [`Walk::checkpoints`].
    pub checkpoints: &'a [TextSize],
}

/// The state of a walk before a top-level statement: the catalog and the session after the
/// statements before it. It only depends on the text before the statement, the section the
/// statement is in, and the parameters of the walk other than the statements.
#[derive(Clone)]
pub struct WalkState {
    at: TextSize,
    /// The start of the migration section that `at` is in, or zero before the first section.
    section_start: TextSize,
    catalog: Catalog,
    session: Session,
    /// In a down section, the catalog after the up sections before it.
    up_catalog: Option<Catalog>,
    /// The start of the query whose implicit transaction block the session is in.
    implicit_block: Option<TextSize>,
}

impl WalkState {
    /// The start of the statement before which the walk stopped.
    pub fn at(&self) -> TextSize {
        self.at
    }

    /// Whether `sections` put the statement at [`Self::at`] in the section it was in when the
    /// walk stopped. A marker inserted right before the statement changes that, even though the
    /// text before the statement's old start is the same.
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
    /// The catalog as a statement at [`WalkParams::end`] sees it.
    pub catalog: Catalog,
    /// The state of the walk at each of [`WalkParams::checkpoints`] that it got to: before the
    /// first statement that starts at or after it. Resuming from it walks like walking from the
    /// start of the file, as long as the text before it doesn't change.
    pub checkpoints: Vec<WalkState>,
}

/// Walks the statements like the database runs them: the catalog and the session follow the
/// statements, so that each statement sees the objects that the statements before it create,
/// change or drop. `visit` gets each statement with the file context as it is before it.
/// Returns the catalog as a statement at [`WalkParams::end`] sees it, and the states at the
/// checkpoints.
///
/// - Each migration section runs in a session and transaction of its own. A down section runs
///   after the up sections before it, and its changes don't carry into the next section.
/// - A top-level statement that doesn't parse taints the catalog if it may have changed it.
/// - The statements of SQL function bodies are visited, but they don't run.
/// - The names of a statement with identifier parameters, like psql's `:name`, are placeholders.
///   Such a statement is visited and applied to the session (its locks and its effect on the
///   transaction), but if it may change the catalog, it taints it instead of being applied to it.
///
/// The file context only knows the statements that the walk is given, except for
/// [`AnalysedFileContext::stmt_count`] and the queries that run in an implicit transaction
/// block, which [`WalkParams::statement_ranges`] tell.
pub fn walk_statements(
    params: WalkParams<'_>,
    mut visit: impl FnMut(&AnalysableStatement, &AnalysedFileContext),
) -> Walk {
    let roots: Vec<&pgls_query::NodeEnum> = params.stmts.iter().map(|s| &*s.root).collect();

    // The catalog after the up sections so far.
    let mut catalog = Catalog::new(params.catalog_base);
    if params.best_effort {
        catalog = catalog.best_effort();
    }
    let mut unparsable = params.unparsable.iter().peekable();
    let mut from = params.from;
    // The walk can't record the states before where it resumes.
    let resumes_at = from.as_ref().map_or(TextSize::from(0), WalkState::at);
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
        let mut implicit_block = None;
        let (unit_catalog, session) = match from.take() {
            Some(from) => {
                debug_assert_eq!(
                    from.section_start, unit.start,
                    "the state fits the sections"
                );
                if let Some(up_catalog) = from.up_catalog {
                    catalog = up_catalog;
                }
                implicit_block = from.implicit_block;
                (from.catalog, from.session)
            }
            None => {
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
                (unit_catalog, session)
            }
        };
        let stmts = &params.stmts[unit.stmts.clone()];
        let last_end = {
            let unparsable_before = |end: TextSize| {
                params
                    .unparsable
                    .partition_point(|skipped| skipped.range.start() < end)
            };
            params.unparsable[unparsable_before(unit.start)..unparsable_before(unit.end)]
                .iter()
                .map(|skipped| skipped.range.end())
                .chain(stmts.last().map(|stmt| stmt.range.end()))
                .max()
                .unwrap_or_default()
        };
        let statements_before = |offset: TextSize| {
            params
                .statement_ranges
                .partition_point(|statement| statement.start() < offset)
        };
        // The blocks that don't end before the statement, which comes after those before it.
        let mut implicit_blocks = unit
            .section
            .into_iter()
            .flat_map(|section| section.implicit_blocks(params.statement_ranges))
            .peekable();
        let mut file_context =
            AnalysedFileContext::new(&roots[unit.stmts.clone()], unit_catalog, session)
                .with_positions(
                    stmts.iter().map(|stmt| stmt.range.start()).collect(),
                    last_end,
                )
                .with_stmt_count(statements_before(unit.end) - statements_before(unit.start));
        let state = |at: TextSize,
                     file_context: &AnalysedFileContext,
                     catalog: &Catalog,
                     implicit_block: Option<TextSize>| WalkState {
            at,
            section_start: unit.start,
            catalog: file_context.catalog().clone(),
            session: file_context.session().clone(),
            up_catalog: is_down.then(|| catalog.clone()),
            implicit_block,
        };

        for stmt in stmts {
            if stmt.range.start() >= params.end {
                break;
            }
            while let Some(at) = checkpoint_at.next_if(|at| stmt.range.start() >= *at) {
                skip_unparsable(&mut unparsable, at, &mut file_context);
                checkpoints.push(state(at, &file_context, &catalog, implicit_block));
            }
            let runs = stmt.function.is_none();
            if runs {
                while implicit_blocks
                    .next_if(|block| block.end() <= stmt.range.start())
                    .is_some()
                {}
                let block = implicit_blocks
                    .peek()
                    .filter(|block| block.contains(stmt.range.start()))
                    .map(|block| block.start());
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

            // The names of a statement with identifier parameters are placeholders, so the
            // catalog can't follow its changes. Its locks and its effect on the transaction are
            // still those of the statement.
            let placeholders = stmt.has_identifier_parameters && !is_plain_query(&stmt.root);
            if runs && placeholders {
                file_context.taint_catalog();
            }
            file_context.next(runs, !placeholders);
        }

        while let Some(at) = checkpoint_at.next_if(|at| *at < unit.end && *at <= params.end) {
            skip_unparsable(&mut unparsable, at, &mut file_context);
            checkpoints.push(state(at, &file_context, &catalog, implicit_block));
        }
        skip_unparsable(&mut unparsable, unit.end.min(params.end), &mut file_context);
        // The section that contains `end` decides what a statement there sees.
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

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use pgls_text_size::{TextRange, TextSize};

    use super::{WalkParams, walk_statements};
    use crate::{AnalysableStatement, Direction, MigrationSection, UnparsableStatement};

    /// Walks the statements of `sql` in a section without a transaction, with the given queries.
    /// Returns each statement that parses with whether it runs in an implicit transaction block,
    /// and how long the walk took.
    fn walk_queries(sql: &str, queries: Vec<TextRange>) -> (Vec<(String, bool)>, Duration) {
        let mut stmts = Vec::new();
        let mut unparsable = Vec::new();
        let statement_ranges = pgls_statement_splitter::split(sql).ranges;
        for &range in &statement_ranges {
            let text = &sql[range];
            match pgls_query::parse(text) {
                Ok(parsed) => stmts.push(
                    AnalysableStatement::new(parsed.into_root().unwrap(), range).with_sql(text),
                ),
                Err(_) => unparsable.push(UnparsableStatement {
                    range,
                    sql: text.to_string(),
                }),
            }
        }
        let sections = [MigrationSection {
            range: TextRange::up_to(TextSize::of(sql)),
            direction: Direction::Up,
            transaction: false,
            queries,
        }];
        let mut visited = Vec::new();
        let start = Instant::now();
        walk_statements(
            WalkParams {
                stmts: &stmts,
                unparsable: &unparsable,
                statement_ranges: &statement_ranges,
                catalog_base: None,
                search_path: &[],
                sections: &sections,
                end: TextSize::of(sql),
                best_effort: false,
                from: None,
                checkpoints: &[],
            },
            |stmt, file_context| {
                visited.push((
                    stmt.sql.clone().unwrap_or_default(),
                    file_context.session().in_implicit_block(),
                ))
            },
        );
        (visited, start.elapsed())
    }

    #[test]
    fn queries_with_several_statements_run_in_implicit_blocks() {
        let sql = "select 1; select 2; select 3; selec 4; select 5; select 6;";
        let query = |from: &str, to: &str| {
            TextRange::new(
                TextSize::try_from(sql.find(from).unwrap()).unwrap(),
                TextSize::try_from(sql.find(to).unwrap() + to.len()).unwrap(),
            )
        };
        let (visited, _) = walk_queries(
            sql,
            vec![
                query("select 1", "select 2;"),
                // A single statement, and one with a statement that doesn't parse.
                query("select 3", "select 3;"),
                query("selec 4", "select 5;"),
            ],
        );
        assert_eq!(
            visited,
            [
                ("select 1;".to_string(), true),
                ("select 2;".to_string(), true),
                ("select 3;".to_string(), false),
                ("select 5;".to_string(), true),
                ("select 6;".to_string(), false),
            ]
        );
    }

    #[test]
    fn many_queries_are_walked_in_linear_time() {
        // Two statements per query, like goose `StatementBegin` blocks. Looking up the
        // statements of each query among all statements took seconds.
        let queries = 8_000;
        let sql = "select 1; select 2;\n".repeat(queries);
        let blocks = (0..queries)
            .map(|i| TextRange::at(TextSize::try_from(i * 20).unwrap(), 19.into()))
            .collect();
        let (visited, elapsed) = walk_queries(&sql, blocks);
        assert!(visited.iter().all(|(_, in_block)| *in_block));
        assert!(elapsed < Duration::from_secs(1), "{elapsed:?}");
    }
}
