use super::TRIVIA_TOKENS;
use pgls_lexer::SyntaxKind;

use super::{
    Splitter,
    ddl::{alter, create},
    dml::{cte, explain, query},
};

#[derive(Debug)]
pub(crate) struct ReachedEOFException;

pub(crate) type SplitterResult = std::result::Result<(), ReachedEOFException>;

pub(crate) fn source(p: &mut Splitter) -> SplitterResult {
    loop {
        match p.kind(p.current_pos) {
            SyntaxKind::EOF => {
                break;
            }
            SyntaxKind::META_COMMAND | SyntaxKind::COPY_DATA => {
                p.skip_psql_input();
            }
            kind if TRIVIA_TOKENS.contains(&kind) || kind == SyntaxKind::LINE_ENDING => {
                p.advance()?;
            }
            // An empty statement, e.g. the second `;` of `select 1;;`, which Postgres discards.
            // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L1116
            SyntaxKind::SEMICOLON => {
                p.advance()?;
            }
            _ => {
                statement(p)?;
            }
        }
    }

    Ok(())
}

pub(crate) fn statement(p: &mut Splitter) -> SplitterResult {
    p.start_stmt();

    // Currently, Err means that we reached EOF.
    // Regardless of whether we reach EOF or we complete the statement, we want to close it.
    // We might want to handle other kinds of errors differently in the future.
    let _ = match p.current() {
        SyntaxKind::WITH_KW => cte(p),
        SyntaxKind::SELECT_KW
        | SyntaxKind::INSERT_KW
        | SyntaxKind::UPDATE_KW
        | SyntaxKind::DELETE_KW => query(p),
        SyntaxKind::CREATE_KW => create(p),
        SyntaxKind::ALTER_KW => alter(p),
        SyntaxKind::EXPLAIN_KW => explain(p),
        kind => p.unknown(kind),
    };

    p.close_stmt();

    Ok(())
}

impl Splitter<'_> {
    /// Skips a `BEGIN ... END` block, e.g. the `BEGIN ATOMIC` body of a function, or a `CASE
    /// ... END` expression, including the blocks and `CASE` expressions nested in it.
    ///
    /// `CASE` also ends with `END`, so it opens a level just like `BEGIN`, which is how psql finds
    /// the end of the statement:
    /// https://github.com/postgres/postgres/blob/REL_18_6/src/fe_utils/psqlscan.l#L1080
    ///
    /// The statements of a `BEGIN ATOMIC` body hold no blocks, and neither does an expression, so
    /// a `begin` there is a column name, as `begin` is an unreserved keyword.
    ///
    /// The depth is a counter, so nesting costs neither stack nor memory, and every iteration
    /// moves by one token.
    fn block(&mut self) -> SplitterResult {
        let begin = self.current() == SyntaxKind::BEGIN_KW;
        let begin_opens = begin && self.look_ahead() != SyntaxKind::ATOMIC_KW;
        self.advance()?;

        let mut depth: usize = 1;

        loop {
            match self.current() {
                SyntaxKind::BEGIN_KW if begin_opens && !self.at_label() => depth += 1,
                SyntaxKind::CASE_KW if !self.at_label() => depth += 1,
                SyntaxKind::END_KW if !self.at_label() => {
                    self.advance()?;
                    // `END CASE` closes a PL/pgSQL `CASE` statement, the `CASE` does not open
                    // another level.
                    if begin {
                        self.eat(SyntaxKind::CASE_KW)?;
                    }
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                    continue;
                }
                _ => {}
            }
            self.advance()?;
        }

        Ok(())
    }

    /// Whether the current keyword is a column label rather than a keyword: Postgres accepts any
    /// keyword, even a reserved one like `end` or `case`, after `AS` and after a `.`, e.g.
    /// `SELECT t.end AS case FROM t`.
    ///
    /// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L17655
    pub(super) fn at_label(&self) -> bool {
        matches!(self.look_back(), Some(SyntaxKind::AS_KW | SyntaxKind::DOT))
    }

    pub(crate) fn parenthesis(&mut self) -> SplitterResult {
        // Without the `(`, e.g. in half-written SQL, there's no `)` to look for: the next one
        // could be statements away.
        if self.current() != SyntaxKind::L_PAREN {
            return self.expect(SyntaxKind::L_PAREN);
        }
        self.advance()?;

        let mut depth = 1;

        loop {
            match self.current() {
                SyntaxKind::L_PAREN => {
                    self.advance()?;
                    depth += 1;
                }
                SyntaxKind::R_PAREN => {
                    self.advance()?;
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {
                    self.advance()?;
                }
            }
        }

        Ok(())
    }

    /// Skips the rest of a statement up to its `;` or the next statement start, see
    /// [`Splitter::statement_start`]. `statement` is the keyword the statement starts with, or
    /// `SCHEMA` for `CREATE SCHEMA`.
    pub(crate) fn unknown(&mut self, statement: SyntaxKind) -> SplitterResult {
        // The statements the rest is in: `statement`, and `INSERT` once an `INSERT INTO` is
        // skipped, e.g. in `PREPARE p AS INSERT INTO ...`, whose query may follow.
        let mut within = [statement; 2];
        // Set by CTEs in a statement that takes a query, e.g. `PREPARE p AS WITH ...`: the
        // statement after the CTEs is that query.
        let mut cte_query = false;

        loop {
            match self.current() {
                SyntaxKind::SEMICOLON => {
                    // A semicolon definitively terminates the current statement.
                    // Move by one raw token so callers cannot mistake the first
                    // keyword of the next statement for a clause of this one.
                    self.step()?;
                    break;
                }
                SyntaxKind::LINE_ENDING => {
                    if self.look_back() == Some(SyntaxKind::COMMA) {
                        self.advance()?;
                    } else {
                        break;
                    }
                }
                SyntaxKind::CASE_KW if !self.at_label() => {
                    self.block()?;
                }
                SyntaxKind::L_PAREN => {
                    self.parenthesis()?;
                }
                // `BEGIN` on its own or with a transaction mode starts a transaction, otherwise
                // a block.
                SyntaxKind::BEGIN_KW => match self.look_ahead() {
                    SyntaxKind::SEMICOLON
                    | SyntaxKind::TRANSACTION_KW
                    | SyntaxKind::WORK_KW
                    | SyntaxKind::ISOLATION_KW
                    | SyntaxKind::READ_KW
                    | SyntaxKind::WRITE_KW
                    | SyntaxKind::ONLY_KW
                    | SyntaxKind::DEFERRABLE_KW
                    | SyntaxKind::NOT_KW => {
                        self.advance()?;
                    }
                    _ => {
                        self.block()?;
                    }
                },
                kind => {
                    if let Some(start) = self.statement_start(&within) {
                        if !start.continues_after(self.look_back(), cte_query) {
                            if !self.at_operand() {
                                break;
                            }
                            // Noted either way: [`crate::split`] decides with Postgres' parser
                            // whether it starts a statement after all.
                            self.operand_keywords.push(self.current_pos);
                            if !self.hold_back_statement_starts {
                                break;
                            }
                        }
                        if kind == SyntaxKind::WITH_KW {
                            cte_query = true;
                        } else if start.query {
                            cte_query = false;
                        }
                        if kind == SyntaxKind::INSERT_KW && self.look_ahead() == SyntaxKind::INTO_KW
                        {
                            within[1] = SyntaxKind::INSERT_KW;
                        }
                    }
                    self.advance()?;
                }
            }
        }
        Ok(())
    }
}
