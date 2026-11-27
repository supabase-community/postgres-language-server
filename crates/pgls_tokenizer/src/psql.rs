//! What psql and query builders put into SQL: psql variables and meta-commands, and named
//! parameters. The COPY data rows after `COPY … FROM STDIN` are read by `pgls_lexer`, which
//! follows the statements.

use crate::class::CharClass;
use crate::cursor::{Cursor, EOF_CHAR};
use crate::quoted::Quote;
use crate::{NamedParamKind, TokenKind};

impl Cursor<'_> {
    /// Eats a psql variable after its `:`, if one follows, or returns a colon. psql reads `:name`,
    /// `:'name'`, `:"name"` and `:{?name}` only when the name is variable characters; otherwise
    /// it sends the `:` as is:
    /// https://github.com/postgres/postgres/blob/REL_18_6/src/fe_utils/psqlscan.l#L717-L775
    ///
    /// https://www.postgresql.org/docs/current/app-psql.html#APP-PSQL-INTERPOLATION
    pub(crate) fn psql_variable(&mut self) -> TokenKind {
        let rest = self.as_str();
        let (kind, name, close) = if let Some(name) = rest.strip_prefix('\'') {
            (NamedParamKind::ColonString, name, "'")
        } else if let Some(name) = rest.strip_prefix('"') {
            (NamedParamKind::ColonIdentifier, name, "\"")
        } else if let Some(name) = rest.strip_prefix("{?") {
            (NamedParamKind::ColonDefined, name, "}")
        } else if Quote::prefixed(rest).is_some() {
            return TokenKind::Colon;
        } else {
            (NamedParamKind::ColonRaw, rest, "")
        };
        let name_len = name
            .find(|c: char| !CharClass::VariableChar.contains(c))
            .unwrap_or(name.len());
        let Some(after) = name.get(name_len..) else {
            return TokenKind::Colon;
        };
        if name_len == 0 || !after.starts_with(close) {
            return TokenKind::Colon;
        }
        self.bump_bytes(rest.len() - after.len() + close.len());
        // psql replaces `:'name'` with a string constant, which strings on the next lines continue:
        // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L224-L230
        if kind == NamedParamKind::ColonString && self.eat_quote_continuation() {
            self.quoted_text(Quote::Xq);
        }
        TokenKind::NamedParam { kind }
    }

    /// Eats a named parameter like `@name` after its `@`.
    ///
    /// Used in sqlc: https://docs.sqlc.dev/en/latest/howto/named_parameters.html
    pub(crate) fn at_param(&mut self) -> TokenKind {
        self.eat_while(|c| CharClass::IdentCont.contains(c));
        TokenKind::NamedParam {
            kind: NamedParamKind::AtPrefix,
        }
    }

    /// Eats what follows a `\`. `\;` and `\:` put a `;` or `:` into the query, any other `\`
    /// starts a meta-command:
    /// https://github.com/postgres/postgres/blob/REL_18_6/src/fe_utils/psqlscan.l#L695-L715
    pub(crate) fn backslash(&mut self) -> TokenKind {
        match self.first() {
            ';' | ':' => TokenKind::Backslash,
            _ => self.meta_command(),
        }
    }

    /// Eats a psql meta-command after its `\`: the command name, and its arguments up to the end
    /// of the line or to an unquoted `\`. A `\\` there ends the command and is eaten with it, so
    /// the text after it is SQL again; any other `\` starts the next meta-command.
    ///
    /// psql scans a meta-command one line at a time, so nothing in it spans lines. The name runs
    /// to whitespace or a `\` (`xslashcmd`). `'…'` (with `''` and backslash escapes), `` `…` ``
    /// and `"…"` quote a `\` in the arguments (`xslasharg`, `xslashquote`, ...). Variables like
    /// `:'x'` don't change where the arguments end, so they aren't told apart. A few commands
    /// take the rest of the line as their argument (`OT_WHOLE_LINE`), and `\g`, `\o` and `\w`
    /// do when it starts with `|` (`OT_FILEPIPE`).
    ///
    /// https://github.com/postgres/postgres/blob/REL_18_6/src/bin/psql/psqlscanslash.l#L142-L447
    /// https://github.com/postgres/postgres/blob/REL_18_6/src/bin/psql/command.c#L355-L477
    fn meta_command(&mut self) -> TokenKind {
        let rest = self.as_str();
        self.eat_while(|c| {
            !CharClass::NonNewlineSpace.contains(c) && !CharClass::Newline.contains(c) && c != '\\'
        });
        let name = self.eaten_since(rest);
        self.eat_while(|c| CharClass::NonNewlineSpace.contains(c));

        let whole_line = matches!(
            name,
            "copy" | "ef" | "ev" | "h" | "help" | "sf" | "sf+" | "sv" | "sv+" | "unrestrict" | "!"
        ) || (matches!(name, "g" | "gx" | "o" | "out" | "w" | "write")
            && self.first() == '|');
        if whole_line {
            self.eat_while(|c| !CharClass::Newline.contains(c));
            return TokenKind::MetaCommand;
        }

        loop {
            match self.first() {
                '\\' => {
                    if self.second() == '\\' {
                        self.bump();
                        self.bump();
                    }
                    break;
                }
                '\'' => {
                    self.bump();
                    loop {
                        match self.first() {
                            '\'' => {
                                self.bump();
                                if self.first() != '\'' {
                                    break;
                                }
                                self.bump();
                            }
                            '\\' => {
                                self.bump();
                                if !CharClass::Newline.contains(self.first()) {
                                    self.bump();
                                }
                            }
                            c if CharClass::Newline.contains(c) => break,
                            EOF_CHAR if self.is_eof() => break,
                            _ => {
                                self.bump();
                            }
                        }
                    }
                }
                quote @ ('`' | '"') => {
                    self.bump();
                    self.eat_while(|c| c != quote && !CharClass::Newline.contains(c));
                    if self.first() == quote {
                        self.bump();
                    }
                }
                c if CharClass::Newline.contains(c) => break,
                EOF_CHAR if self.is_eof() => break,
                _ => {
                    self.bump();
                }
            }
        }
        TokenKind::MetaCommand
    }
}
