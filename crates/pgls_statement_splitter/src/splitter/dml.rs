use pgls_lexer::SyntaxKind;

use crate::splitter::common::SplitterResult;

use super::{
    Splitter,
    common::{parenthesis, unknown},
    ddl::create,
};

pub(crate) fn cte(p: &mut Splitter) -> SplitterResult {
    p.expect(SyntaxKind::WITH_KW)?;
    p.eat(SyntaxKind::RECURSIVE_KW)?;

    loop {
        p.expect(SyntaxKind::IDENT)?;
        p.expect(SyntaxKind::AS_KW)?;
        // Handle optional [NOT] MATERIALIZED hint (PostgreSQL 12+)
        p.eat(SyntaxKind::NOT_KW)?;
        p.eat(SyntaxKind::MATERIALIZED_KW)?;
        parenthesis(p)?;

        if p.current() == SyntaxKind::COMMA {
            p.advance()?;
        } else {
            break;
        }
    }

    unknown(
        p,
        &[
            SyntaxKind::SELECT_KW,
            SyntaxKind::INSERT_KW,
            SyntaxKind::UPDATE_KW,
            SyntaxKind::DELETE_KW,
            SyntaxKind::MERGE_KW,
        ],
    )?;
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
        parenthesis(p)?;
    } else {
        // legacy EXPLAIN [ANALYZE | ANALYSE] [VERBOSE] <statement>
        p.eat(SyntaxKind::ANALYZE_KW)?;
        p.eat(SyntaxKind::ANALYSE_KW)?;
        p.eat(SyntaxKind::VERBOSE_KW)?;
    }

    match p.current() {
        SyntaxKind::WITH_KW => cte(p),
        SyntaxKind::SELECT_KW => select(p),
        SyntaxKind::INSERT_KW => insert(p),
        SyntaxKind::UPDATE_KW => update(p),
        SyntaxKind::DELETE_KW => delete(p),
        // EXPLAIN CREATE TABLE AS / CREATE MATERIALIZED VIEW AS
        SyntaxKind::CREATE_KW => create(p),
        // MERGE, VALUES, EXECUTE, DECLARE
        _ => unknown(p, &[]),
    }
}

pub(crate) fn select(p: &mut Splitter) -> SplitterResult {
    p.expect(SyntaxKind::SELECT_KW)?;

    unknown(p, &[])
}

pub(crate) fn insert(p: &mut Splitter) -> SplitterResult {
    p.expect(SyntaxKind::INSERT_KW)?;
    p.expect(SyntaxKind::INTO_KW)?;

    unknown(p, &[SyntaxKind::SELECT_KW])
}

pub(crate) fn update(p: &mut Splitter) -> SplitterResult {
    p.expect(SyntaxKind::UPDATE_KW)?;

    unknown(p, &[])
}

pub(crate) fn delete(p: &mut Splitter) -> SplitterResult {
    p.expect(SyntaxKind::DELETE_KW)?;
    p.expect(SyntaxKind::FROM_KW)?;

    unknown(p, &[])
}
