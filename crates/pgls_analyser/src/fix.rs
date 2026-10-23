//! Helpers to compute the edits of rule fixes from the tokens of a statement.

use pgls_query::protobuf::{KeywordKind, ScanToken, Token};
use pgls_text_size::{TextRange, TextSize};

use crate::{LinterRule, LinterRuleContext, RuleFix};

/// The tokens of a statement, with offsets relative to the statement. Comments are not tokens,
/// so the token after `timestamp` in `timestamp /* local */ without time zone` is `without`.
#[derive(Clone, Copy)]
pub struct StatementTokens<'a> {
    sql: &'a str,
    tokens: &'a [ScanToken],
    comments: &'a [ScanToken],
}

impl<'a> StatementTokens<'a> {
    pub(crate) fn new(sql: &'a str, tokens: &'a [ScanToken], comments: &'a [ScanToken]) -> Self {
        Self {
            sql,
            tokens,
            comments,
        }
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

    /// A fix that inserts the keywords ` <text>` after the token at `index`, in the case of that
    /// token if it is a keyword, or else of the statement's first keyword.
    pub fn insert_after(&self, index: usize, message: &str, text: &str) -> RuleFix {
        let is_keyword = |index: usize| self.tokens[index].keyword_kind() != KeywordKind::NoKeyword;
        let like = Some(index)
            .filter(|index| is_keyword(*index))
            .or_else(|| (0..self.tokens.len()).find(|index| is_keyword(*index)))
            .map_or("", |index| self.text(index));
        RuleFix::new(message).insert(
            self.range(index).end(),
            format!(" {}", in_case_of(text, like)),
        )
    }

    /// A fix that replaces the token at `index` with `text`, in the case of the token, e.g.
    /// `TIMESTAMP` with `TIMESTAMPTZ`.
    pub fn replace(&self, index: usize, message: &str, text: &str) -> RuleFix {
        RuleFix::new(message).replace(self.range(index), in_case_of(text, self.text(index)))
    }

    /// Replaces the text in `range` with `text`, but keeps the comments in it, e.g. to replace
    /// `varchar /* name */ (255)` with `text /* name */`.
    pub fn replace_keeping_comments(&self, fix: RuleFix, range: TextRange, text: &str) -> RuleFix {
        let mut replacement = text.to_string();
        for comment in self.comments {
            let comment = TextRange::new(
                TextSize::from(comment.start as u32),
                TextSize::from(comment.end as u32),
            );
            if range.contains_range(comment) {
                replacement.push(' ');
                replacement.push_str(&self.sql[comment]);
            }
        }
        fix.replace(range, replacement)
    }
}

/// `text` in uppercase if `like` is, and in lowercase otherwise.
pub(crate) fn in_case_of(text: &str, like: &str) -> String {
    if like.chars().any(|c| c.is_ascii_uppercase()) && !like.chars().any(|c| c.is_ascii_lowercase())
    {
        text.to_uppercase()
    } else {
        text.to_lowercase()
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
