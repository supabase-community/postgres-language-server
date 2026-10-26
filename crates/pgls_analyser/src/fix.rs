//! Helpers to compute the edits of rule fixes from the tokens of a statement.

use pgls_query::protobuf::{KeywordKind, ScanToken, Token};
use pgls_text_size::{TextRange, TextSize};

use crate::{LinterRule, LinterRuleContext, RuleFix};

/// The tokens of a statement, with offsets relative to the statement.
#[derive(Clone, Copy)]
pub struct StatementTokens<'a> {
    sql: &'a str,
    tokens: &'a [ScanToken],
}

impl<'a> StatementTokens<'a> {
    pub(crate) fn new(sql: &'a str, tokens: &'a [ScanToken]) -> Self {
        Self { sql, tokens }
    }

    /// The range of the token at `index`.
    pub fn range(&self, index: usize) -> TextRange {
        let token = &self.tokens[index];
        TextRange::new(
            TextSize::from(token.start as u32),
            TextSize::from(token.end as u32),
        )
    }

    /// The text of the token at `index`.
    pub fn text(&self, index: usize) -> &'a str {
        &self.sql[self.range(index)]
    }

    pub fn kind(&self, index: usize) -> Option<Token> {
        self.tokens.get(index).map(ScanToken::token)
    }

    /// The first token of the given kind that is not inside parentheses.
    pub fn find(&self, kind: Token) -> Option<usize> {
        self.find_any(&[kind])
    }

    /// The first token of any of the given kinds that is not inside parentheses.
    pub fn find_any(&self, kinds: &[Token]) -> Option<usize> {
        let mut depth = 0usize;
        for (index, token) in self.tokens.iter().enumerate() {
            match token.token() {
                Token::Ascii40 => depth += 1,
                Token::Ascii41 => depth = depth.saturating_sub(1),
                kind if depth == 0 && kinds.contains(&kind) => return Some(index),
                _ => {}
            }
        }
        None
    }

    /// The token that starts at `offset`, e.g. at the location of a node.
    pub fn at(&self, offset: i32) -> Option<usize> {
        self.tokens.iter().position(|token| token.start == offset)
    }

    /// The closing parenthesis that matches the opening one at `open`.
    pub fn closing_paren(&self, open: usize) -> Option<usize> {
        let mut depth = 0usize;
        for (index, token) in self.tokens.iter().enumerate().skip(open) {
            match token.token() {
                Token::Ascii40 => depth += 1,
                Token::Ascii41 => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(index);
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// The last token of the element that starts at `start`: everything up to the next `,` or
    /// `;` that is not inside parentheses, e.g. a constraint in `ALTER TABLE`.
    pub fn element_end(&self, start: usize) -> usize {
        let mut depth = 0usize;
        let mut last = start;
        for (index, token) in self.tokens.iter().enumerate().skip(start) {
            match token.token() {
                Token::Ascii40 => depth += 1,
                Token::Ascii41 if depth == 0 => break,
                Token::Ascii41 => depth -= 1,
                Token::Ascii44 | Token::Ascii59 if depth == 0 => break,
                Token::SqlComment | Token::CComment => continue,
                _ => {}
            }
            last = index;
        }
        last
    }

    /// The words of the type name that starts at the token `start`, e.g. `pg_catalog.varchar`
    /// or `character varying`: the index of the last token before the type modifiers, array
    /// bounds, or whatever follows the type.
    pub fn type_name_end(&self, start: usize) -> usize {
        let mut last = start;
        let mut index = start + 1;
        while let Some(kind) = self.kind(index) {
            let is_word =
                kind == Token::Ident || self.tokens[index].keyword_kind() != KeywordKind::NoKeyword;
            let continues = match kind {
                Token::Ascii46 => true,
                // Only the keywords that can be part of a multi-word type name.
                Token::Varying | Token::Precision => true,
                _ => is_word && self.kind(last) == Some(Token::Ascii46),
            };
            if !continues {
                break;
            }
            last = index;
            index += 1;
        }
        last
    }

    /// The type modifiers right after the token `index`, e.g. `(255)`: the indexes of the
    /// opening and closing parentheses.
    pub fn type_modifiers(&self, index: usize) -> Option<(usize, usize)> {
        let open = index + 1;
        if self.kind(open)? != Token::Ascii40 {
            return None;
        }
        Some((open, self.closing_paren(open)?))
    }

    /// The ranges of the keyword pairs `first second` that are not inside parentheses, e.g. the
    /// `DROP DEFAULT` of each command of an `ALTER TABLE`.
    pub fn keyword_pairs(&self, first: Token, second: Token) -> Vec<TextRange> {
        let mut depth = 0usize;
        let mut pairs = Vec::new();
        for (index, token) in self.tokens.iter().enumerate() {
            match token.token() {
                Token::Ascii40 => depth += 1,
                Token::Ascii41 => depth = depth.saturating_sub(1),
                kind if depth == 0 && kind == first && self.kind(index + 1) == Some(second) => {
                    pairs.push(self.range(index).cover(self.range(index + 1)));
                }
                _ => {}
            }
        }
        pairs
    }

    /// A fix that inserts ` <text>` after the token at `index`.
    pub fn insert_after(&self, index: usize, message: &str, text: &str) -> RuleFix {
        RuleFix::new(message).insert(self.range(index).end(), format!(" {text}"))
    }
}

/// A fix that inserts `CONCURRENTLY` after the token at `index`, for a statement that Postgres
/// can't run in a transaction block. There is none in a transaction block. If we only assume that
/// the statement runs outside of one, because we don't know the migration tool, the fix is unsafe.
pub(crate) fn add_concurrently<R: LinterRule + 'static>(
    ctx: &LinterRuleContext<R>,
    tokens: &StatementTokens,
    index: usize,
) -> Option<RuleFix> {
    if ctx.in_transaction() {
        return None;
    }
    let fix = tokens.insert_after(index, "Add CONCURRENTLY", "CONCURRENTLY");
    Some(if ctx.knows_transaction() {
        fix
    } else {
        fix.unsafe_fix()
    })
}
