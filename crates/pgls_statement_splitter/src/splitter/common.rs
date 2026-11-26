use std::error::Error;

use super::TRIVIA_TOKENS;
use pgls_lexer::SyntaxKind;

use super::{
    Splitter,
    data::at_statement_start,
    ddl::{alter, create},
    dml::{cte, delete, explain, insert, select, update},
    is_col_id,
};

#[derive(Debug)]
pub(crate) struct ReachedEOFException;

impl std::fmt::Display for ReachedEOFException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ReachedEOFException")
    }
}

impl Error for ReachedEOFException {}

pub(crate) type SplitterResult = std::result::Result<(), ReachedEOFException>;

pub(crate) fn source(p: &mut Splitter) -> SplitterResult {
    loop {
        match p.current() {
            SyntaxKind::EOF => {
                break;
            }
            kind if TRIVIA_TOKENS.contains(&kind) || kind == SyntaxKind::LINE_ENDING => {
                p.advance()?;
            }
            // An empty statement, e.g. the second `;` of `select 1;;`, which Postgres discards.
            // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L1116
            SyntaxKind::SEMICOLON => {
                p.advance()?;
            }
            SyntaxKind::BACKSLASH => {
                plpgsql_command(p)?;
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
        SyntaxKind::SELECT_KW => select(p),
        SyntaxKind::INSERT_KW => insert(p),
        SyntaxKind::UPDATE_KW => update(p),
        SyntaxKind::DELETE_KW => delete(p),
        SyntaxKind::CREATE_KW => create(p),
        SyntaxKind::ALTER_KW => alter(p),
        SyntaxKind::EXPLAIN_KW => explain(p),
        _ => unknown(p, &[]),
    };

    p.close_stmt();

    Ok(())
}

/// Skips a `BEGIN ... END` block, e.g. the `BEGIN ATOMIC` body of a function.
///
/// `CASE` also ends with `END`, so it opens a level just like `BEGIN`, which is how psql finds
/// the end of the statement:
/// https://github.com/postgres/postgres/blob/REL_18_6/src/fe_utils/psqlscan.l#L1080
///
/// The statements of a `BEGIN ATOMIC` body hold no blocks, so a `begin` there is a column name,
/// as `begin` is an unreserved keyword.
///
/// The depth is a counter, so nesting costs neither stack nor memory, and every iteration moves
/// by one token.
pub(crate) fn begin_end(p: &mut Splitter) -> SplitterResult {
    let atomic = p.look_ahead(true) == SyntaxKind::ATOMIC_KW;
    p.expect(SyntaxKind::BEGIN_KW)?;

    let mut depth: usize = 1;

    loop {
        match p.current() {
            SyntaxKind::BEGIN_KW if !atomic && !at_label(p) => {
                p.advance()?;
                depth += 1;
            }
            SyntaxKind::CASE_KW if !at_label(p) => {
                p.advance()?;
                depth += 1;
            }
            SyntaxKind::END_KW if !at_label(p) => {
                p.advance()?;
                // `END CASE` closes a PL/pgSQL `CASE` statement, the `CASE` does not open
                // another level.
                p.eat(SyntaxKind::CASE_KW)?;
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            _ => {
                p.advance()?;
            }
        }
    }

    Ok(())
}

/// Whether the current keyword is a column label rather than a keyword: Postgres accepts any
/// keyword, even a reserved one like `end` or `case`, after `AS` and after a `.`, e.g.
/// `SELECT t.end AS case FROM t`.
///
/// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L17655
fn at_label(p: &Splitter) -> bool {
    matches!(p.look_back(true), Some(SyntaxKind::AS_KW | SyntaxKind::DOT))
}

pub(crate) fn parenthesis(p: &mut Splitter) -> SplitterResult {
    // Without the `(`, e.g. in half-written SQL, there's no `)` to look for: the next one could
    // be statements away.
    if p.current() != SyntaxKind::L_PAREN {
        return p.expect(SyntaxKind::L_PAREN);
    }
    p.advance()?;

    let mut depth = 1;

    loop {
        match p.current() {
            SyntaxKind::L_PAREN => {
                p.advance()?;
                depth += 1;
            }
            SyntaxKind::R_PAREN => {
                if p.current() == SyntaxKind::R_PAREN {
                    p.advance()?;
                }
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            _ => {
                p.advance()?;
            }
        }
    }

    Ok(())
}

pub(crate) fn plpgsql_command(p: &mut Splitter) -> SplitterResult {
    p.expect(SyntaxKind::BACKSLASH)?;

    loop {
        match p.current() {
            SyntaxKind::LINE_ENDING => {
                p.advance()?;
                break;
            }
            _ => {
                // advance the splitter to the next token without ignoring irrelevant tokens
                // we would skip a newline with `advance()`
                p.step()?;
            }
        }
    }

    Ok(())
}

/// Skips a `CASE ... END` expression, including the `CASE` expressions nested in it. The depth
/// is a counter, so nesting costs neither stack nor memory, and every iteration moves by one
/// token.
pub(crate) fn case(p: &mut Splitter) -> SplitterResult {
    p.expect(SyntaxKind::CASE_KW)?;

    let mut depth: usize = 1;

    loop {
        match p.current() {
            SyntaxKind::CASE_KW if !at_label(p) => {
                p.advance()?;
                depth += 1;
            }
            SyntaxKind::END_KW if !at_label(p) => {
                p.advance()?;
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            _ => {
                p.advance()?;
            }
        }
    }

    Ok(())
}

/// Whether the `WITH` at the current token starts common table expressions, rather than being
/// part of a clause like `COPY ... WITH (...)`, `DECLARE ... WITH HOLD` or `WITH NO DATA`. A CTE
/// is `WITH RECURSIVE`, `WITH name (` or `WITH name AS [NOT] MATERIALIZED (`. The end of the text
/// counts as a match, so a half-written CTE still starts a statement. Looks at most three tokens
/// ahead.
///
/// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L13040
pub(crate) fn at_cte(p: &Splitter) -> bool {
    match p.nth(1) {
        SyntaxKind::RECURSIVE_KW | SyntaxKind::EOF => true,
        name if is_col_id(name) => match p.nth(2) {
            SyntaxKind::L_PAREN | SyntaxKind::EOF => true,
            SyntaxKind::AS_KW => matches!(
                p.nth(3),
                SyntaxKind::L_PAREN
                    | SyntaxKind::NOT_KW
                    | SyntaxKind::MATERIALIZED_KW
                    | SyntaxKind::EOF
            ),
            _ => false,
        },
        _ => false,
    }
}

pub(crate) fn unknown(p: &mut Splitter, exclude: &[SyntaxKind]) -> SplitterResult {
    // Set by CTEs in a statement that takes a query, e.g. `PREPARE p AS WITH ...`: the statement
    // after the CTEs is that query.
    let mut cte_query = false;
    // Set by an `INSERT INTO` in a statement that takes one, e.g. `PREPARE p AS INSERT INTO ...`:
    // a `SELECT` or `WITH` after it is its query, like in `insert`.
    let mut insert_query = false;

    loop {
        match p.current() {
            SyntaxKind::SEMICOLON => {
                // A semicolon definitively terminates the current statement.
                // Move by one raw token so callers cannot mistake the first
                // keyword of the next statement for a clause of this one.
                p.step()?;
                break;
            }
            SyntaxKind::LINE_ENDING => {
                if p.look_back(true).is_some_and(|t| t == SyntaxKind::COMMA) {
                    p.advance()?;
                } else {
                    break;
                }
            }
            SyntaxKind::CASE_KW if !at_label(p) => {
                case(p)?;
            }
            SyntaxKind::BACKSLASH => {
                // pgsql commands
                // we want to check if the previous token non-trivia token is a LINE_ENDING
                // we cannot use the is_trivia() method because that would exclude LINE_ENDINGs
                // with count > 1
                if (0..p.current_pos)
                    .rev()
                    .find_map(|idx| {
                        let kind = p.kind(idx);
                        if !TRIVIA_TOKENS.contains(&kind) {
                            Some(kind)
                        } else {
                            None
                        }
                    })
                    .is_some_and(|t| t == SyntaxKind::LINE_ENDING)
                {
                    break;
                }
                p.advance()?;
            }
            SyntaxKind::L_PAREN => {
                parenthesis(p)?;
            }
            SyntaxKind::BEGIN_KW => match p.look_ahead(true) {
                SyntaxKind::SEMICOLON => {
                    p.advance()?;
                }
                SyntaxKind::ATOMIC_KW => {
                    begin_end(p)?;
                }
                SyntaxKind::TRANSACTION_KW
                | SyntaxKind::WORK_KW
                | SyntaxKind::ISOLATION_KW
                | SyntaxKind::READ_KW
                | SyntaxKind::WRITE_KW
                | SyntaxKind::ONLY_KW
                | SyntaxKind::DEFERRABLE_KW
                | SyntaxKind::NOT_KW => {
                    p.advance()?;
                }
                _ => {
                    begin_end(p)?;
                }
            },
            t => match at_statement_start(t, exclude).filter(|_| {
                !(insert_query && matches!(t, SyntaxKind::WITH_KW | SyntaxKind::SELECT_KW))
                    // A keyword before a `.` qualifies a name, e.g. the schema in `alter.c`.
                    && p.look_ahead(true) != SyntaxKind::DOT
            }) {
                Some(SyntaxKind::SELECT_KW) => {
                    let prev = p.look_back(true);
                    if [
                        // for policies, with for select
                        SyntaxKind::FOR_KW,
                        // for create view / table as
                        SyntaxKind::AS_KW,
                        // for create rule
                        SyntaxKind::ON_KW,
                        // for create rule
                        SyntaxKind::ALSO_KW,
                        // for create rule
                        SyntaxKind::INSTEAD_KW,
                        // for UNION
                        SyntaxKind::UNION_KW,
                        // for UNION ALL
                        SyntaxKind::ALL_KW,
                        // for UNION ... EXCEPT
                        SyntaxKind::EXCEPT_KW,
                        // for INTERSECT and UNION DISTINCT
                        // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L12935
                        SyntaxKind::INTERSECT_KW,
                        SyntaxKind::DISTINCT_KW,
                        // for grant
                        SyntaxKind::GRANT_KW,
                        // for revoke
                        SyntaxKind::REVOKE_KW,
                        SyntaxKind::COMMA,
                        // for BEGIN ATOMIC
                        SyntaxKind::ATOMIC_KW,
                    ]
                    .iter()
                    .all(|x| Some(x) != prev.as_ref())
                        && !cte_query
                        // a column label after a `.`, e.g. `t.select`
                        && !(prev == Some(SyntaxKind::DOT) && p.hold_back())
                    {
                        break;
                    }

                    cte_query = false;
                    p.advance()?;
                }
                // DELETE is also an unreserved keyword used by hstore's delete() function, and
                // in `ON COMMIT DELETE ROWS`. A DELETE statement must be followed by FROM, so
                // neither `delete(` nor `DELETE ROWS` can start one.
                // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L4674
                Some(SyntaxKind::DELETE_KW)
                    if matches!(
                        p.look_ahead(true),
                        SyntaxKind::L_PAREN | SyntaxKind::ROWS_KW
                    ) =>
                {
                    p.advance()?;
                }
                Some(SyntaxKind::INSERT_KW)
                | Some(SyntaxKind::UPDATE_KW)
                | Some(SyntaxKind::DELETE_KW) => {
                    let prev = p.look_back(true);
                    if [
                        // for create trigger
                        SyntaxKind::BEFORE_KW,
                        SyntaxKind::AFTER_KW,
                        // for policies, e.g. for insert
                        SyntaxKind::FOR_KW,
                        // e.g. on insert or delete
                        SyntaxKind::OR_KW,
                        // e.g. INSTEAD OF INSERT
                        SyntaxKind::OF_KW,
                        // for create rule
                        SyntaxKind::ON_KW,
                        // for create rule
                        SyntaxKind::ALSO_KW,
                        // for create rule
                        SyntaxKind::INSTEAD_KW,
                        // for grant
                        SyntaxKind::GRANT_KW,
                        // for revoke
                        SyntaxKind::REVOKE_KW,
                        SyntaxKind::COMMA,
                        // Do update in INSERT stmt
                        SyntaxKind::DO_KW,
                        // FOR NO KEY UPDATE
                        SyntaxKind::KEY_KW,
                        // WHEN MATCHED THEN
                        SyntaxKind::THEN_KW,
                        // PREPARE p AS INSERT
                        // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L12107
                        SyntaxKind::AS_KW,
                        // LOCK t IN SHARE UPDATE EXCLUSIVE MODE
                        // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L12469
                        SyntaxKind::SHARE_KW,
                    ]
                    .iter()
                    .all(|x| Some(x) != prev.as_ref())
                        && !cte_query
                        // the keyword is unreserved, e.g. a column in `where update = 1`
                        && !p.hold_back()
                    {
                        break;
                    }
                    cte_query = false;
                    insert_query |=
                        t == SyntaxKind::INSERT_KW && p.look_ahead(true) == SyntaxKind::INTO_KW;
                    p.advance()?;
                }
                // Part of a clause, e.g. `WITH ORDINALITY`, `WITH CHECK` or `WITH TIME ZONE`.
                Some(SyntaxKind::WITH_KW) if !at_cte(p) => {
                    p.advance()?;
                }
                Some(SyntaxKind::WITH_KW) => {
                    let prev = p.look_back(true);
                    // The query of a statement that takes one: `CREATE VIEW v AS`, `PREPARE p
                    // AS`, `DECLARE c CURSOR FOR` and the actions of a rule.
                    if !matches!(
                        prev,
                        Some(
                            SyntaxKind::AS_KW
                                | SyntaxKind::FOR_KW
                                | SyntaxKind::DO_KW
                                | SyntaxKind::INSTEAD_KW
                                | SyntaxKind::ALSO_KW
                        )
                    )
                        // a column label after a `.`, e.g. `t.with`; held back with its query
                        && !(prev == Some(SyntaxKind::DOT) && p.hold_back())
                    {
                        break;
                    }
                    cte_query = true;
                    p.advance()?;
                }
                // ALTER is unreserved too, e.g. a column in `select id, alter from t`, and a
                // privilege in `REVOKE [GRANT OPTION FOR] ALTER SYSTEM ON PARAMETER ...`.
                // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L7754
                Some(SyntaxKind::ALTER_KW)
                    if matches!(
                        p.look_back(true),
                        Some(SyntaxKind::GRANT_KW | SyntaxKind::REVOKE_KW | SyntaxKind::FOR_KW)
                    ) || p.hold_back() =>
                {
                    p.advance()?;
                }
                Some(SyntaxKind::CREATE_KW) => {
                    let prev = p.look_back(true);
                    if [
                        // for grant
                        SyntaxKind::GRANT_KW,
                        // for revoke
                        SyntaxKind::REVOKE_KW,
                        SyntaxKind::COMMA,
                    ]
                    .iter()
                    .all(|x| Some(x) != prev.as_ref())
                        // a column label after a `.`, e.g. `t.create`
                        && !(prev == Some(SyntaxKind::DOT) && p.hold_back())
                    {
                        break;
                    }

                    p.advance()?;
                }
                Some(_) => {
                    break;
                }
                None => {
                    p.advance()?;
                }
            },
        }
    }
    Ok(())
}
