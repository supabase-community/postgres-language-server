use pgls_lexer::SyntaxKind;

use crate::splitter::common::SplitterResult;

use super::{
    Splitter,
    common::{parenthesis, unknown},
};

pub(crate) fn create(p: &mut Splitter) -> SplitterResult {
    p.expect(SyntaxKind::CREATE_KW)?;

    // `CREATE SCHEMA s CREATE TABLE t (...) CREATE VIEW ...` creates the objects in the schema.
    // psql sends it as one statement, up to the `;`.
    // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L1517
    let exclude: &[SyntaxKind] = if p.current() == SyntaxKind::SCHEMA_KW {
        &[SyntaxKind::CREATE_KW]
    } else {
        &[]
    };

    loop {
        unknown(p, exclude)?;

        // `unknown` keeps the CTEs of a query after `AS` in the statement. Other `WITH`s it stops
        // at are clauses of the statement, e.g. `WITH (fillfactor = 10)`.
        if p.current() != SyntaxKind::WITH_KW {
            return Ok(());
        }

        p.expect(SyntaxKind::WITH_KW)?;
        if p.current() == SyntaxKind::L_PAREN {
            parenthesis(p)?;
        }
    }
}

pub(crate) fn alter(p: &mut Splitter) -> SplitterResult {
    p.expect(SyntaxKind::ALTER_KW)?;

    unknown(p, &[SyntaxKind::ALTER_KW, SyntaxKind::WITH_KW])
}
