use pgls_lexer::SyntaxKind;

use crate::splitter::common::SplitterResult;

use super::{
    Splitter,
    common::{parenthesis, unknown},
    dml::cte,
};

pub(crate) fn create(p: &mut Splitter) -> SplitterResult {
    p.expect(SyntaxKind::CREATE_KW)?;

    loop {
        unknown(p, &[])?;

        if p.current() != SyntaxKind::WITH_KW {
            return Ok(());
        }

        if p.look_back(true) == Some(SyntaxKind::AS_KW) {
            return cte(p);
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
