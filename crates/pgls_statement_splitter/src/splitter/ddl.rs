use pgls_lexer::SyntaxKind;

use crate::splitter::common::SplitterResult;

use super::{Splitter, common::unknown};

pub(crate) fn create(p: &mut Splitter) -> SplitterResult {
    p.expect(SyntaxKind::CREATE_KW)?;

    // `CREATE SCHEMA s CREATE TABLE t (...) CREATE VIEW ...` creates the objects in the schema.
    // psql sends it as one statement, up to the `;`.
    // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L1517
    if p.current() == SyntaxKind::SCHEMA_KW {
        return unknown(p, &[SyntaxKind::CREATE_KW]);
    }

    unknown(p, &[])
}

pub(crate) fn alter(p: &mut Splitter) -> SplitterResult {
    p.expect(SyntaxKind::ALTER_KW)?;

    unknown(p, &[SyntaxKind::ALTER_KW, SyntaxKind::WITH_KW])
}
