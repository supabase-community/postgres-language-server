use pgt_lexer::SyntaxKind;

use super::{
    Parser,
    common::{parenthesis, unknown},
};

pub(crate) fn cte(p: &mut Parser) {
    p.expect(SyntaxKind::With);

    loop {
        p.expect(SyntaxKind::Ident);
        p.expect(SyntaxKind::As);
        parenthesis(p);

        if !p.advance_if_kind(SyntaxKind::Ascii44) {
            break;
        }
    }

    unknown(
        p,
        &[
            SyntaxKind::Select,
            SyntaxKind::Insert,
            SyntaxKind::Update,
            SyntaxKind::DeleteP,
            SyntaxKind::Merge,
        ],
    );
}

pub(crate) fn select(p: &mut Parser) {
    p.expect(SyntaxKind::Select);

    unknown(p, &[]);
}

pub(crate) fn insert(p: &mut Parser) {
    p.expect(SyntaxKind::Insert);
    p.expect(SyntaxKind::Into);

    unknown(p, &[SyntaxKind::Select]);
}

pub(crate) fn update(p: &mut Parser) {
    p.expect(SyntaxKind::Update);

    unknown(p, &[]);
}

pub(crate) fn delete(p: &mut Parser) {
    p.expect(SyntaxKind::DeleteP);
    p.expect(SyntaxKind::From);

    unknown(p, &[]);
}
