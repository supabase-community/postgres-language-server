mod common;
mod data;
mod ddl;
mod dml;

pub(crate) use common::source;

use pgls_lexer::{KeywordCategory, Lexed, SyntaxKind};
use pgls_text_size::{TextRange, TextSize};

use crate::splitter::common::{ReachedEOFException, SplitterResult};

pub(crate) struct SplitResult {
    pub ranges: Vec<TextRange>,
    pub errors: Vec<SplitError>,
    /// The statement keywords after a token that needs an operand, see [`Splitter::hold_back`].
    pub operand_keywords: OperandKeywords,
}

/// The offsets of the statement keywords after a token that needs an operand, in order, see
/// [`Splitter::hold_back`].
#[derive(Debug, Default)]
pub(crate) struct OperandKeywords(Vec<TextSize>);

impl OperandKeywords {
    /// Whether one of the keywords is in `range`.
    pub(crate) fn any_in(&self, range: TextRange) -> bool {
        let first = self.0.partition_point(|&offset| offset < range.start());
        self.0
            .get(first)
            .is_some_and(|&offset| offset < range.end())
    }

    /// Whether `range` starts at one of the keywords.
    pub(crate) fn start(&self, range: TextRange) -> bool {
        self.0.binary_search(&range.start()).is_ok()
    }
}

pub(crate) static TRIVIA_TOKENS: &[SyntaxKind] = &[
    SyntaxKind::SPACE,
    SyntaxKind::TAB,
    SyntaxKind::VERTICAL_TAB,
    SyntaxKind::FORM_FEED,
    SyntaxKind::COMMENT,
    // LINE_ENDING is relevant
];

/// Whether `kind` is a `ColId`, a name Postgres accepts for a table or column:
/// an identifier, or an unreserved or column name keyword.
///
/// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L17632
fn is_col_id(kind: SyntaxKind) -> bool {
    kind == SyntaxKind::IDENT
        || matches!(
            kind.keyword_category(),
            Some(KeywordCategory::Unreserved | KeywordCategory::ColName)
        )
}

/// Tokens a statement can't end with: operators, and keywords that need an operand or a name
/// after them. A statement keyword after one of them is an operand or a name, e.g. the column in
/// `where update = 1` or `select id, alter from t`, unless the statement before it is unfinished
/// and misses its `;`. `SELECT` can end a statement, but is followed by its target list in
/// practice, e.g. `select update, delete from permissions`.
static NEEDS_OPERAND: &[SyntaxKind] = &[
    SyntaxKind::COMMA,
    SyntaxKind::DOT,
    SyntaxKind::L_PAREN,
    SyntaxKind::L_BRACK,
    SyntaxKind::COLON,
    SyntaxKind::DOUBLE_COLON,
    SyntaxKind::EQ,
    SyntaxKind::L_ANGLE,
    SyntaxKind::R_ANGLE,
    SyntaxKind::PLUS,
    SyntaxKind::MINUS,
    SyntaxKind::SLASH,
    SyntaxKind::PERCENT,
    SyntaxKind::CARET,
    SyntaxKind::TILDE,
    SyntaxKind::BANG,
    SyntaxKind::AT,
    SyntaxKind::POUND,
    SyntaxKind::AMP,
    SyntaxKind::PIPE,
    SyntaxKind::QUESTION,
    SyntaxKind::SELECT_KW,
    SyntaxKind::DISTINCT_KW,
    SyntaxKind::FROM_KW,
    SyntaxKind::JOIN_KW,
    SyntaxKind::ON_KW,
    SyntaxKind::USING_KW,
    SyntaxKind::WHERE_KW,
    SyntaxKind::HAVING_KW,
    SyntaxKind::BY_KW,
    SyntaxKind::SET_KW,
    SyntaxKind::INTO_KW,
    SyntaxKind::TABLE_KW,
    SyntaxKind::RETURNING_KW,
    SyntaxKind::AS_KW,
    SyntaxKind::AND_KW,
    SyntaxKind::OR_KW,
    SyntaxKind::NOT_KW,
    SyntaxKind::IS_KW,
    SyntaxKind::IN_KW,
    SyntaxKind::LIKE_KW,
    SyntaxKind::ILIKE_KW,
    SyntaxKind::BETWEEN_KW,
    SyntaxKind::WHEN_KW,
    SyntaxKind::THEN_KW,
    SyntaxKind::ELSE_KW,
];

/// Internal error type used during splitting
#[derive(Debug, Clone)]
pub struct SplitError {
    pub msg: String,
    pub token: usize,
}

#[derive(Debug)]
pub(crate) struct Splitter<'a> {
    lexed: &'a Lexed<'a>,
    current_pos: usize,
    stmt_ranges: Vec<(usize, usize)>,
    errors: Vec<SplitError>,
    current_stmt_start: Option<usize>,
    /// Whether a blank line terminates the current statement.
    blank_line_is_boundary: bool,
    /// Whether to hold back statement starts, see [`Splitter::hold_back`].
    hold_back_statement_starts: bool,
    /// The tokens of the statement keywords after a token that needs an operand.
    operand_keywords: Vec<usize>,
}

impl<'a> Splitter<'a> {
    /// Splits on `;` and on structural statement starts. If `blank_line_is_boundary`, a blank
    /// line is a statement boundary too: a recovery pass on fragments PostgreSQL cannot parse,
    /// where there is no better signal to resynchronize on. If `hold_back_statement_starts`, a
    /// statement keyword after a token that needs an operand doesn't start a statement, see
    /// [`Splitter::hold_back`].
    pub(crate) fn new(
        lexed: &'a Lexed<'a>,
        blank_line_is_boundary: bool,
        hold_back_statement_starts: bool,
    ) -> Self {
        Self {
            lexed,
            current_pos: 0,
            stmt_ranges: Vec::new(),
            errors: Vec::new(),
            current_stmt_start: None,
            blank_line_is_boundary,
            hold_back_statement_starts,
            operand_keywords: Vec::new(),
        }
    }

    pub(crate) fn finish(self) -> SplitResult {
        let ranges = self
            .stmt_ranges
            .iter()
            .map(|(start_token_pos, end_token_pos)| {
                let from = self.lexed.range(*start_token_pos).start();
                let to = self.lexed.range(*end_token_pos).end();
                TextRange::new(from, to)
            })
            .collect();

        let operand_keywords = OperandKeywords(
            self.operand_keywords
                .iter()
                .map(|&token| self.lexed.range(token).start())
                .collect(),
        );

        SplitResult {
            ranges,
            errors: self.errors,
            operand_keywords,
        }
    }

    /// Whether the statement keyword at the current token continues the statement because it
    /// follows a token that needs an operand, see [`NEEDS_OPERAND`], and statement starts are held
    /// back. Notes the keyword in either case: [`crate::split`] decides with Postgres' parser
    /// whether it starts a statement after all.
    fn hold_back(&mut self) -> bool {
        let Some(prev) = (0..self.current_pos)
            .rev()
            .find(|&idx| !self.is_trivia(idx))
        else {
            return false;
        };
        if !NEEDS_OPERAND.contains(&self.lexed.kind(prev)) && !self.star_needs_operand(prev) {
            return false;
        }
        self.operand_keywords.push(self.current_pos);
        self.hold_back_statement_starts
    }

    /// Whether the `*` at `idx` needs an operand: it ends an operator of several characters, e.g.
    /// `~*`, or it multiplies, e.g. `2*`. A `*` on its own can end a statement, e.g. `select *`
    /// or `select t.*`. Postgres reads a run of operator characters as one operator:
    /// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L367-L368
    /// and a `*` after an operand is `a_expr '*' a_expr`:
    /// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L14988
    fn star_needs_operand(&self, idx: usize) -> bool {
        if self.lexed.kind(idx) != SyntaxKind::STAR {
            return false;
        }
        let multiplies = (0..idx)
            .rev()
            .find(|&before| !self.is_trivia(before))
            .is_some_and(|before| {
                matches!(
                    self.lexed.kind(before),
                    SyntaxKind::INT_NUMBER
                        | SyntaxKind::FLOAT_NUMBER
                        | SyntaxKind::IDENT
                        | SyntaxKind::STRING
                        | SyntaxKind::ESC_STRING
                        | SyntaxKind::BIT_STRING
                        | SyntaxKind::BYTE_STRING
                        | SyntaxKind::DOLLAR_QUOTED_STRING
                        | SyntaxKind::POSITIONAL_PARAM
                        | SyntaxKind::NAMED_PARAM
                        | SyntaxKind::R_PAREN
                        | SyntaxKind::R_BRACK
                )
            });
        multiplies
            || idx.checked_sub(1).is_some_and(|before| {
                matches!(
                    self.lexed.kind(before),
                    SyntaxKind::TILDE
                        | SyntaxKind::BANG
                        | SyntaxKind::AT
                        | SyntaxKind::POUND
                        | SyntaxKind::CARET
                        | SyntaxKind::AMP
                        | SyntaxKind::PIPE
                        | SyntaxKind::BACKTICK
                        | SyntaxKind::QUESTION
                        | SyntaxKind::PLUS
                        | SyntaxKind::MINUS
                        | SyntaxKind::STAR
                        | SyntaxKind::SLASH
                        | SyntaxKind::PERCENT
                        | SyntaxKind::L_ANGLE
                        | SyntaxKind::R_ANGLE
                        | SyntaxKind::EQ
                )
            })
    }

    pub(crate) fn start_stmt(&mut self) {
        assert!(
            self.current_stmt_start.is_none(),
            "cannot start statement within statement",
        );
        self.current_stmt_start = Some(self.current_pos);
    }

    pub(crate) fn close_stmt(&mut self) {
        assert!(
            self.current_stmt_start.is_some(),
            "Must start statement before closing it."
        );

        let Some(start_token_pos) = self.current_stmt_start else {
            return;
        };

        assert!(
            self.current_pos > start_token_pos,
            "Must close the statement on a token that's later than the start token: {} > {}",
            self.current_pos,
            start_token_pos
        );

        // The start token isn't trivia, so this finds it at the latest.
        let end_token_pos = (0..self.current_pos)
            .rev()
            .find(|&idx| !self.is_trivia(idx))
            .unwrap_or(start_token_pos);

        self.stmt_ranges.push((start_token_pos, end_token_pos));

        self.current_stmt_start = None;
    }

    /// The kind of the current token. A psql meta-command is `EOF` here: it ends the statement
    /// before it wherever it is, e.g. the query that `\gset` sends in `select 1 \gset`. Only
    /// [`source`] sees it, and skips it.
    fn current(&self) -> SyntaxKind {
        match self.lexed.kind(self.current_pos) {
            SyntaxKind::META_COMMAND => SyntaxKind::EOF,
            kind => kind,
        }
    }

    /// Moves past the psql meta-command at the current token, which belongs to no statement.
    fn skip_meta_command(&mut self) {
        debug_assert_eq!(self.lexed.kind(self.current_pos), SyntaxKind::META_COMMAND);
        self.current_pos += 1;
    }

    fn eat(&mut self, kind: SyntaxKind) -> Result<bool, ReachedEOFException> {
        if self.current() == kind {
            self.advance()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn kind(&self, idx: usize) -> SyntaxKind {
        self.lexed.kind(idx)
    }

    /// Advances the parser to the next relevant token and returns it.
    ///
    /// NOTE: This will skip trivia tokens.
    fn advance(&mut self) -> Result<SyntaxKind, ReachedEOFException> {
        if self.current() == SyntaxKind::EOF {
            return Err(ReachedEOFException);
        }

        // The EOF token isn't trivia, so this finds it at the latest.
        let Some(pos) = (self.current_pos + 1..self.lexed.len()).find(|&idx| !self.is_trivia(idx))
        else {
            return Err(ReachedEOFException);
        };

        self.current_pos = pos;
        Ok(self.lexed.kind(pos))
    }

    /// Advances the parser by exactly one token and returns it.
    ///
    /// NOTE: Unlike [`advance`], this does NOT skip trivia tokens. Use this
    /// when the caller needs to walk the raw token stream (e.g. searching
    /// for a single-newline `LINE_ENDING`, which `advance` would skip).
    fn step(&mut self) -> Result<SyntaxKind, ReachedEOFException> {
        if self.current() == SyntaxKind::EOF {
            return Err(ReachedEOFException);
        }

        self.current_pos += 1;
        Ok(self.lexed.kind(self.current_pos))
    }

    /// The next token, or EOF at the end.
    fn look_ahead(&self, ignore_trivia: bool) -> SyntaxKind {
        let pos = if ignore_trivia {
            (self.current_pos + 1..self.lexed.len()).find(|&idx| !self.is_trivia(idx))
        } else {
            (self.current_pos + 1..self.lexed.len()).next()
        };
        // There is nothing after EOF, e.g. after a `WITH` at the end of the text.
        pos.map_or(SyntaxKind::EOF, |pos| self.lexed.kind(pos))
    }

    /// The `n`th token after the current one (`n` starts at 1), ignoring trivia, or EOF after
    /// the end. Looks at most `n` tokens ahead, so it costs no more than [`look_ahead`] for a
    /// constant `n`.
    fn nth(&self, n: usize) -> SyntaxKind {
        (self.current_pos + 1..self.lexed.len())
            .filter(|&idx| !self.is_trivia(idx))
            .nth(n.saturating_sub(1))
            .map_or(SyntaxKind::EOF, |pos| self.lexed.kind(pos))
    }

    /// Returns `None` if there are no previous relevant tokens
    fn look_back(&self, ignore_trivia: bool) -> Option<SyntaxKind> {
        if ignore_trivia {
            (0..self.current_pos)
                .rev()
                .find(|&idx| !self.is_trivia(idx))
                .map(|idx| self.lexed.kind(idx))
        } else {
            (0..self.current_pos)
                .next_back()
                .map(|idx| self.lexed.kind(idx))
        }
    }

    fn is_trivia(&self, idx: usize) -> bool {
        match self.lexed.kind(idx) {
            k if TRIVIA_TOKENS.contains(&k) => true,
            SyntaxKind::LINE_ENDING => {
                !self.blank_line_is_boundary || self.lexed.line_ending_count(idx) < 2
            }
            _ => false,
        }
    }

    /// Like [`expect`] for a `ColId`, see [`is_col_id`].
    fn expect_col_id(&mut self) -> SplitterResult {
        if is_col_id(self.current()) {
            self.advance()?;
            Ok(())
        } else {
            self.expect(SyntaxKind::IDENT)
        }
    }

    /// Will advance if the `kind` matches the current token.
    /// Otherwise, will add a diagnostic to the internal `errors`.
    fn expect(&mut self, kind: SyntaxKind) -> SplitterResult {
        if self.current() == kind {
            self.advance()?;
        } else {
            let token = if self.current() == SyntaxKind::EOF {
                self.current_pos - 1
            } else {
                self.current_pos
            };

            self.errors.push(SplitError {
                msg: format!("Expected {}", kind.description()),
                token,
            });
        };

        Ok(())
    }
}
