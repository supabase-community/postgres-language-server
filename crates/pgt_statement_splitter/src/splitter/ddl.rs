use pgt_lexer::SyntaxKind;

use crate::splitter::common::SplitterException;

use super::{Splitter, common::unknown};

pub(crate) fn create(p: &mut Splitter) -> Result<(), SplitterException> {
    p.expect(SyntaxKind::CREATE_KW)?;

    unknown(p, &[SyntaxKind::WITH_KW])
}

pub(crate) fn alter(p: &mut Splitter) -> Result<(), SplitterException> {
    p.expect(SyntaxKind::ALTER_KW)?;

    unknown(p, &[SyntaxKind::ALTER_KW])
}
