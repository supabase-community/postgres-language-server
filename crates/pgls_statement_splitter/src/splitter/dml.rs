use pgls_lexer::SyntaxKind;

use crate::splitter::common::SplitterResult;

use super::{Splitter, ddl::create};

pub(crate) fn cte(p: &mut Splitter) -> SplitterResult {
    p.expect(SyntaxKind::WITH_KW)?;
    // RECURSIVE is unreserved, so `WITH recursive AS (...)` names a CTE
    if !matches!(p.look_ahead(), SyntaxKind::AS_KW | SyntaxKind::L_PAREN) {
        p.eat(SyntaxKind::RECURSIVE_KW)?;
    }

    loop {
        p.expect_col_id()?;
        if p.current() == SyntaxKind::L_PAREN {
            p.parenthesis()?;
        }
        p.expect(SyntaxKind::AS_KW)?;
        // Handle optional [NOT] MATERIALIZED hint (PostgreSQL 12+)
        p.eat(SyntaxKind::NOT_KW)?;
        p.eat(SyntaxKind::MATERIALIZED_KW)?;
        p.parenthesis()?;
        search_and_cycle(p)?;

        if p.current() == SyntaxKind::COMMA {
            p.advance()?;
        } else {
            break;
        }
    }

    match p.current() {
        SyntaxKind::SELECT_KW
        | SyntaxKind::INSERT_KW
        | SyntaxKind::UPDATE_KW
        | SyntaxKind::DELETE_KW => query(p),
        _ => p.unknown(SyntaxKind::WITH_KW),
    }
}

/// Skips the `SEARCH ... SET column` and `CYCLE ... USING column` clauses after a CTE. A clause
/// without its end, e.g. while it's typed, ends at the next `;`.
///
/// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L13061
fn search_and_cycle(p: &mut Splitter) -> SplitterResult {
    for (start, end) in [
        (SyntaxKind::SEARCH_KW, SyntaxKind::SET_KW),
        (SyntaxKind::CYCLE_KW, SyntaxKind::USING_KW),
    ] {
        if p.current() != start {
            continue;
        }
        loop {
            match p.current() {
                kind if kind == end => {
                    p.advance()?;
                    p.expect_col_id()?;
                    break;
                }
                SyntaxKind::SEMICOLON | SyntaxKind::EOF => return Ok(()),
                _ => {
                    p.advance()?;
                }
            }
        }
    }
    Ok(())
}

/// `EXPLAIN [ ANALYZE ] [ VERBOSE ] <statement>` and
/// `EXPLAIN ( option [, ...] ) <statement>`
///
/// EXPLAIN is a prefix to the statement it explains, so after consuming the
/// prefix we delegate to the splitter function of the inner statement instead
/// of treating its leading keyword as a new statement start.
pub(crate) fn explain(p: &mut Splitter) -> SplitterResult {
    p.expect(SyntaxKind::EXPLAIN_KW)?;

    // EXPLAIN (ANALYZE, BUFFERS, FORMAT JSON, ...) <statement>
    if p.current() == SyntaxKind::L_PAREN {
        p.parenthesis()?;
    } else {
        // legacy EXPLAIN [ANALYZE | ANALYSE] [VERBOSE] <statement>
        p.eat(SyntaxKind::ANALYZE_KW)?;
        p.eat(SyntaxKind::ANALYSE_KW)?;
        p.eat(SyntaxKind::VERBOSE_KW)?;
    }

    match p.current() {
        SyntaxKind::WITH_KW => cte(p),
        SyntaxKind::SELECT_KW
        | SyntaxKind::INSERT_KW
        | SyntaxKind::UPDATE_KW
        | SyntaxKind::DELETE_KW => query(p),
        // EXPLAIN CREATE TABLE AS / CREATE MATERIALIZED VIEW AS
        SyntaxKind::CREATE_KW => create(p),
        // MERGE, VALUES, EXECUTE, DECLARE
        _ => p.unknown(SyntaxKind::EXPLAIN_KW),
    }
}

/// `SELECT`, `INSERT INTO`, `UPDATE` or `DELETE FROM`.
pub(crate) fn query(p: &mut Splitter) -> SplitterResult {
    let kind = p.current();
    p.advance()?;
    match kind {
        SyntaxKind::INSERT_KW => p.expect(SyntaxKind::INTO_KW)?,
        SyntaxKind::DELETE_KW => p.expect(SyntaxKind::FROM_KW)?,
        _ => {}
    }

    p.unknown(kind)
}
