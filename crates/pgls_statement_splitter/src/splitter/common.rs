use std::error::Error;

use super::TRIVIA_TOKENS;
use pgls_lexer::SyntaxKind;

use super::{
    Splitter,
    data::at_statement_start,
    ddl::{alter, create},
    dml::{cte, delete, insert, select, update},
};

#[derive(Debug)]
pub struct ReachedEOFException;

impl std::fmt::Display for ReachedEOFException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ReachedEOFException")
    }
}

impl Error for ReachedEOFException {}

pub(crate) type SplitterResult = std::result::Result<(), ReachedEOFException>;

pub fn source(p: &mut Splitter) -> SplitterResult {
    loop {
        match p.current() {
            SyntaxKind::EOF => {
                break;
            }
            kind if TRIVIA_TOKENS.contains(&kind) || kind == SyntaxKind::LINE_ENDING => {
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
        _ => unknown(p, &[]),
    };

    p.close_stmt();

    Ok(())
}

pub(crate) fn begin_end(p: &mut Splitter) -> SplitterResult {
    p.expect(SyntaxKind::BEGIN_KW)?;

    let mut depth = 1;

    loop {
        match p.current() {
            SyntaxKind::BEGIN_KW => {
                p.advance()?;
                depth += 1;
            }
            SyntaxKind::END_KW => {
                if p.current() == SyntaxKind::END_KW {
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

pub(crate) fn parenthesis(p: &mut Splitter) -> SplitterResult {
    p.expect(SyntaxKind::L_PAREN)?;

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
                p.current_pos += 1;
            }
        }
    }

    Ok(())
}

pub(crate) fn case(p: &mut Splitter) -> SplitterResult {
    p.expect(SyntaxKind::CASE_KW)?;

    loop {
        match p.current() {
            SyntaxKind::END_KW => {
                p.advance()?;
                break;
            }
            _ => {
                p.advance()?;
            }
        }
    }

    Ok(())
}

pub(crate) fn unknown(p: &mut Splitter, exclude: &[SyntaxKind]) -> SplitterResult {
    loop {
        match p.current() {
            SyntaxKind::SEMICOLON => {
                p.advance()?;
                break;
            }
            SyntaxKind::LINE_ENDING => {
                if p.look_back(true).is_some_and(|t| t == SyntaxKind::COMMA) {
                    p.advance()?;
                } else {
                    break;
                }
            }
            SyntaxKind::CASE_KW => {
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
            SyntaxKind::BEGIN_KW => {
                if p.look_ahead(true) != SyntaxKind::SEMICOLON {
                    // BEGIN; should be treated as a statement terminator
                    begin_end(p)?;
                } else {
                    p.advance()?;
                }
            }
            t => match at_statement_start(t, exclude) {
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
                    {
                        break;
                    }

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
                    ]
                    .iter()
                    .all(|x| Some(x) != prev.as_ref())
                    {
                        break;
                    }
                    p.advance()?;
                }
                Some(SyntaxKind::WITH_KW) => {
                    let next = p.look_ahead(true);
                    if [
                        // WITH ORDINALITY should not start a new statement
                        SyntaxKind::ORDINALITY_KW,
                        // WITH CHECK should not start a new statement
                        SyntaxKind::CHECK_KW,
                        // TIMESTAMP WITH TIME ZONE should not start a new statement
                        SyntaxKind::TIME_KW,
                        SyntaxKind::GRANT_KW,
                        SyntaxKind::ADMIN_KW,
                        SyntaxKind::INHERIT_KW,
                        SyntaxKind::SET_KW,
                    ]
                    .iter()
                    .all(|x| x != &next)
                    {
                        break;
                    }
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
