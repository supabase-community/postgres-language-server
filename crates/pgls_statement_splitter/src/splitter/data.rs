use pgls_lexer::SyntaxKind;

// All tokens listed here must be explicitly handled in the `unknown` function to ensure that we do
// not break in the middle of another statement that contains a statement start token.
//
// All of these statements must have a dedicated splitter function called from the `statement` function
static STATEMENT_START_TOKENS: &[SyntaxKind] = &[
    SyntaxKind::WITH_KW,
    SyntaxKind::SELECT_KW,
    SyntaxKind::INSERT_KW,
    SyntaxKind::UPDATE_KW,
    SyntaxKind::DELETE_KW,
    SyntaxKind::CREATE_KW,
    SyntaxKind::ALTER_KW,
];

pub(crate) fn at_statement_start(kind: SyntaxKind, exclude: &[SyntaxKind]) -> Option<&SyntaxKind> {
    STATEMENT_START_TOKENS
        .iter()
        .filter(|&x| !exclude.contains(x))
        .find(|&x| x == &kind)
}
