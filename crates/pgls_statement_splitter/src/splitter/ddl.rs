use pgls_lexer::SyntaxKind;

use crate::splitter::common::SplitterResult;

use super::Splitter;

pub(crate) fn create(p: &mut Splitter) -> SplitterResult {
    p.expect(SyntaxKind::CREATE_KW)?;

    // `CREATE SCHEMA` holds the statements that create its elements.
    match p.current() {
        SyntaxKind::SCHEMA_KW => p.unknown(SyntaxKind::SCHEMA_KW),
        _ => p.unknown(SyntaxKind::CREATE_KW),
    }
}

pub(crate) fn alter(p: &mut Splitter) -> SplitterResult {
    p.expect(SyntaxKind::ALTER_KW)?;

    // `ALTER EXTENSION name UPDATE [TO version]` updates the extension, its `UPDATE` starts no
    // statement.
    // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L5280
    if p.current() == SyntaxKind::EXTENSION_KW && p.nth(2) == SyntaxKind::UPDATE_KW {
        p.advance()?;
        p.advance()?;
        p.advance()?;
    }

    p.unknown(SyntaxKind::ALTER_KW)
}
