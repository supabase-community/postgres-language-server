use pgt_lexer::{SyntaxKind, Token, TokenType};

use super::{
    Parser,
    data::at_statement_start,
    ddl::{alter, create},
    dml::{cte, delete, insert, select, update},
};

pub fn source(p: &mut Parser) {
    loop {
        match p.peek() {
            Token {
                kind: SyntaxKind::Eof,
                ..
            } => {
                break;
            }
            Token {
                // we might want to ignore TokenType::NoKeyword here too
                // but this will lead to invalid statements to not being picked up
                token_type: TokenType::Whitespace,
                ..
            } => {
                p.advance();
            }
            _ => {
                statement(p);
            }
        }
    }
}

pub(crate) fn statement(p: &mut Parser) {
    p.start_stmt();
    match p.peek().kind {
        SyntaxKind::With => {
            cte(p);
        }
        SyntaxKind::Select => {
            select(p);
        }
        SyntaxKind::Insert => {
            insert(p);
        }
        SyntaxKind::Update => {
            update(p);
        }
        SyntaxKind::DeleteP => {
            delete(p);
        }
        SyntaxKind::Create => {
            create(p);
        }
        SyntaxKind::Alter => {
            alter(p);
        }
        _ => {
            unknown(p, &[]);
        }
    }
    p.close_stmt();
}

pub(crate) fn parenthesis(p: &mut Parser) {
    p.expect(SyntaxKind::Ascii40);

    let mut depth = 1;

    loop {
        match p.peek().kind {
            SyntaxKind::Ascii40 => {
                p.advance();
                depth += 1;
            }
            SyntaxKind::Ascii41 | SyntaxKind::Eof => {
                p.advance();
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            _ => {
                p.advance();
            }
        }
    }
}

pub(crate) fn case(p: &mut Parser) {
    p.expect(SyntaxKind::Case);

    loop {
        match p.peek().kind {
            SyntaxKind::EndP => {
                p.advance();
                break;
            }
            _ => {
                p.advance();
            }
        }
    }
}

pub(crate) fn unknown(p: &mut Parser, exclude: &[SyntaxKind]) {
    loop {
        match p.peek() {
            Token {
                kind: SyntaxKind::Ascii59,
                ..
            } => {
                p.advance();
                break;
            }
            Token {
                kind: SyntaxKind::Newline | SyntaxKind::Eof,
                ..
            } => {
                break;
            }
            Token {
                kind: SyntaxKind::Case,
                ..
            } => {
                case(p);
            }
            Token {
                kind: SyntaxKind::Ascii40,
                ..
            } => {
                parenthesis(p);
            }
            t => match at_statement_start(t.kind, exclude) {
                Some(SyntaxKind::Select) => {
                    let prev = p.look_back().map(|t| t.kind);
                    if [
                        // for policies, with for select
                        SyntaxKind::For,
                        // for create view / table as
                        SyntaxKind::As,
                        // for create rule
                        SyntaxKind::On,
                        // for create rule
                        SyntaxKind::Also,
                        // for create rule
                        SyntaxKind::Instead,
                        // for UNION
                        SyntaxKind::Union,
                        // for UNION ALL
                        SyntaxKind::All,
                        // for UNION ... EXCEPT
                        SyntaxKind::Except,
                    ]
                    .iter()
                    .all(|x| Some(x) != prev.as_ref())
                    {
                        break;
                    }

                    p.advance();
                }
                Some(SyntaxKind::Insert) | Some(SyntaxKind::Update) | Some(SyntaxKind::DeleteP) => {
                    let prev = p.look_back().map(|t| t.kind);
                    if [
                        // for create trigger
                        SyntaxKind::Before,
                        SyntaxKind::After,
                        // for policies, e.g. for insert
                        SyntaxKind::For,
                        // e.g. on insert or delete
                        SyntaxKind::Or,
                        // for create rule
                        SyntaxKind::On,
                        // for create rule
                        SyntaxKind::Also,
                        // for create rule
                        SyntaxKind::Instead,
                    ]
                    .iter()
                    .all(|x| Some(x) != prev.as_ref())
                    {
                        break;
                    }
                    p.advance();
                }
                Some(SyntaxKind::With) => {
                    let next = p.look_ahead().map(|t| t.kind);
                    if [
                        // WITH ORDINALITY should not start a new statement
                        SyntaxKind::Ordinality,
                        // WITH CHECK should not start a new statement
                        SyntaxKind::Check,
                    ]
                    .iter()
                    .all(|x| Some(x) != next.as_ref())
                    {
                        break;
                    }
                    p.advance();
                }
                Some(_) => {
                    break;
                }
                None => {
                    p.advance();
                }
            },
        }
    }
}
