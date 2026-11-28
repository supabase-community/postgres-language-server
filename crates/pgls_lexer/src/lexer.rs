use pgls_tokenizer::{TokenKind, tokenize};

use crate::SyntaxKind;
use crate::lexed::{LexError, Lexed};

/// How many words of a statement psql looks at to tell `COPY … FROM STDIN` (`init_idents`):
/// https://github.com/postgres/postgres/blob/REL_18_6/src/include/fe_utils/psqlscan_int.h#L129-L130
const COPY_WORDS_MAX: usize = 8;

/// How far the first words of the current statement match `COPY … FROM STDIN`, see
/// [`Lexer::track_copy`].
#[derive(Clone, Copy, PartialEq, Eq)]
enum CopyFromStdin {
    /// The statement isn't `COPY … FROM STDIN`.
    No,
    /// `COPY`, without `FROM` yet.
    Copy,
    /// `COPY … FROM`.
    From,
    /// `COPY … FROM STDIN`.
    Stdin,
}

/// Lexer that processes input text into tokens and diagnostics
pub struct Lexer<'a> {
    text: &'a str,
    kind: Vec<SyntaxKind>,
    start: Vec<u32>,
    error: Vec<LexError>,
    offset: usize,
    /// we store line ending counts outside of SyntaxKind because of the u16 represenation of SyntaxKind
    line_ending_counts: Vec<usize>,
    has_blank_line: bool,
    /// The words of the current statement outside parentheses so far, up to
    /// [`COPY_WORDS_MAX`], and the depth of its parentheses, see [`Lexer::track_copy`].
    words: usize,
    paren_depth: usize,
    copy: CopyFromStdin,
    /// Whether the first word of a statement was `COPY`.
    has_copy: bool,
    /// Whether COPY data starts after the next line break.
    copy_data_next: bool,
    /// Whether no line after the current offset is `\.`, so no COPY data follows any more.
    copy_end_missing: bool,
    copy_data: Vec<u32>,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given text
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            kind: Vec::new(),
            start: Vec::new(),
            error: Vec::new(),
            offset: 0,
            line_ending_counts: Vec::new(),
            has_blank_line: false,
            words: 0,
            paren_depth: 0,
            copy: CopyFromStdin::No,
            has_copy: false,
            copy_data_next: false,
            copy_end_missing: false,
            copy_data: Vec::new(),
        }
    }

    /// Lex the input text and return the result
    pub fn lex(mut self) -> Lexed<'a> {
        let mut tokens = tokenize(self.text);
        while let Some(token) = tokens.next() {
            let token_text = &self.text[self.offset..][..token.len as usize];
            if self.copy_data_next && matches!(token.kind, TokenKind::LineEnding { .. }) {
                self.copy_data_next = false;
                if self.copy_data(token_text) {
                    tokens = tokenize(&self.text[self.offset..]);
                    continue;
                }
            }
            self.track_copy(&token.kind, token_text);
            self.extend_token(&token.kind, token_text);
        }

        // Add EOF token
        self.push(SyntaxKind::EOF, 0, None, None);

        Lexed {
            text: self.text,
            kind: self.kind,
            start: self.start,
            error: self.error,
            line_ending_counts: self.line_ending_counts,
            has_blank_line: self.has_blank_line,
            has_copy: self.has_copy,
            copy_data: self.copy_data,
        }
    }

    fn push(
        &mut self,
        kind: SyntaxKind,
        len: usize,
        err: Option<&str>,
        line_ending_count: Option<usize>,
    ) {
        self.kind.push(kind);
        self.start.push(self.offset as u32);
        self.offset += len;

        assert!(
            kind != SyntaxKind::LINE_ENDING || line_ending_count.is_some(),
            "Line ending token must have a line ending count"
        );

        self.line_ending_counts.push(line_ending_count.unwrap_or(0));
        self.has_blank_line |=
            kind == SyntaxKind::LINE_ENDING && line_ending_count.is_some_and(|count| count >= 2);

        if let Some(err) = err {
            let token = (self.kind.len() - 1) as u32;
            let msg = err.to_owned();
            self.error.push(LexError { msg, token });
        }
    }

    /// Follows the words, parentheses and `;` of the statements like psql does, to tell `COPY …
    /// FROM STDIN`, whose data rows psql reads from the lines after it. Its first word is
    /// `COPY`, and the word after its first `FROM` among its first [`COPY_WORDS_MAX`] words is
    /// `STDIN` or `STDOUT`. Quoted identifiers and the words in parentheses, like a column list,
    /// don't count. A `;` outside parentheses ends the statement, and so does a meta-command,
    /// since the splitter ends the statement there. `\copy … from stdin` reads its data the same
    /// way, and takes the arguments of `COPY`.
    ///
    /// https://github.com/postgres/postgres/blob/REL_18_6/src/fe_utils/psqlscan.l#L960-L1046
    /// https://github.com/postgres/postgres/blob/REL_18_6/src/bin/psql/copy.c#L224-L228
    fn track_copy(&mut self, kind: &TokenKind, text: &str) {
        match kind {
            TokenKind::OpenParen => self.paren_depth += 1,
            TokenKind::CloseParen => self.paren_depth = self.paren_depth.saturating_sub(1),
            TokenKind::Ident if self.paren_depth == 0 && self.words < COPY_WORDS_MAX => {
                let is = |word: &str| text.eq_ignore_ascii_case(word);
                self.copy = match self.copy {
                    _ if self.words == 0 && is("copy") => {
                        self.has_copy = true;
                        CopyFromStdin::Copy
                    }
                    _ if self.words == 0 => CopyFromStdin::No,
                    // The word after `FROM` has to be among the first words too.
                    CopyFromStdin::Copy if is("from") && self.words < COPY_WORDS_MAX - 1 => {
                        CopyFromStdin::From
                    }
                    CopyFromStdin::From if is("stdin") || is("stdout") => CopyFromStdin::Stdin,
                    CopyFromStdin::From => CopyFromStdin::No,
                    copy => copy,
                };
                self.words += 1;
            }
            TokenKind::Semi if self.paren_depth == 0 => self.end_statement(),
            TokenKind::MetaCommand => {
                self.end_statement();
                let Some(copy) = text
                    .strip_prefix('\\')
                    .filter(|copy| copy.starts_with("copy"))
                else {
                    return;
                };
                let mut rest = copy;
                for token in tokenize(copy) {
                    let Some((token_text, after)) = rest.split_at_checked(token.len as usize)
                    else {
                        break;
                    };
                    rest = after;
                    // A meta-command after `\copy`'s arguments is its own.
                    if token.kind != TokenKind::MetaCommand {
                        self.track_copy(&token.kind, token_text);
                    }
                }
                self.end_statement();
            }
            _ => {}
        }
    }

    /// Ends the statement for [`Lexer::track_copy`]. COPY data follows the next line break if it
    /// was `COPY … FROM STDIN`.
    fn end_statement(&mut self) {
        self.copy_data_next |= self.copy == CopyFromStdin::Stdin;
        self.words = 0;
        self.paren_depth = 0;
        self.copy = CopyFromStdin::No;
    }

    /// Pushes the first line break of `line_ending`, and the COPY data rows after it as one
    /// token, up to and including a line that is exactly `\.`, and returns whether it did. psql
    /// reads the rows with `fgets`, so only a `\n` ends one.
    ///
    /// psql reads to the end of the file if no `\.` comes, but here the lines stay SQL: a file
    /// is mostly edited, not run, and a `COPY … FROM STDIN` typed above other statements
    /// shouldn't turn all of them into data. It also keeps the statements after a COPY in
    /// Postgres' regression files, whose data `preprocess` blanks. Once no `\.` is left, the
    /// lines aren't searched again, so all COPY statements together cost one pass at most.
    ///
    /// https://github.com/postgres/postgres/blob/REL_18_6/src/bin/psql/copy.c#L626-L640
    fn copy_data(&mut self, line_ending: &str) -> bool {
        if self.copy_end_missing {
            return false;
        }
        let line_break = if line_ending.starts_with("\r\n") {
            2
        } else {
            1
        };
        let mut len = 0;
        let rows = self
            .text
            .get(self.offset + line_break..)
            .unwrap_or_default();
        for line in rows.split_inclusive('\n') {
            if matches!(line, "\\.\n" | "\\.\r\n" | "\\.") {
                self.push(SyntaxKind::LINE_ENDING, line_break, None, Some(1));
                // The line break after `\.` is a token of its own.
                self.copy_data.push(self.kind.len() as u32);
                self.push(SyntaxKind::COPY_DATA, len + 2, None, None);
                return true;
            }
            len += line.len();
        }
        self.copy_end_missing = true;
        false
    }

    fn extend_token(&mut self, kind: &pgls_tokenizer::TokenKind, token_text: &str) {
        let mut err = "";
        let mut line_ending_count = None;

        let syntax_kind = {
            match kind {
                pgls_tokenizer::TokenKind::LineComment => SyntaxKind::COMMENT,
                pgls_tokenizer::TokenKind::BlockComment { terminated } => {
                    if !terminated {
                        err = "Missing trailing `*/` symbols to terminate the block comment";
                    }
                    SyntaxKind::COMMENT
                }
                pgls_tokenizer::TokenKind::Space => SyntaxKind::SPACE,
                pgls_tokenizer::TokenKind::Tab => SyntaxKind::TAB,
                pgls_tokenizer::TokenKind::LineEnding { count } => {
                    line_ending_count = Some(*count);
                    SyntaxKind::LINE_ENDING
                }
                pgls_tokenizer::TokenKind::VerticalTab => SyntaxKind::VERTICAL_TAB,
                pgls_tokenizer::TokenKind::FormFeed => SyntaxKind::FORM_FEED,
                pgls_tokenizer::TokenKind::Ident => {
                    SyntaxKind::from_keyword(token_text).unwrap_or(SyntaxKind::IDENT)
                }
                pgls_tokenizer::TokenKind::Literal { kind, .. } => {
                    self.extend_literal(token_text.len(), kind);
                    return;
                }
                pgls_tokenizer::TokenKind::Semi => SyntaxKind::SEMICOLON,
                pgls_tokenizer::TokenKind::Comma => SyntaxKind::COMMA,
                pgls_tokenizer::TokenKind::Dot => SyntaxKind::DOT,
                pgls_tokenizer::TokenKind::DotDot => SyntaxKind::DOT_DOT,
                pgls_tokenizer::TokenKind::OpenParen => SyntaxKind::L_PAREN,
                pgls_tokenizer::TokenKind::CloseParen => SyntaxKind::R_PAREN,
                pgls_tokenizer::TokenKind::OpenBracket => SyntaxKind::L_BRACK,
                pgls_tokenizer::TokenKind::CloseBracket => SyntaxKind::R_BRACK,
                pgls_tokenizer::TokenKind::At => SyntaxKind::AT,
                pgls_tokenizer::TokenKind::Pound => SyntaxKind::POUND,
                pgls_tokenizer::TokenKind::Tilde => SyntaxKind::TILDE,
                pgls_tokenizer::TokenKind::Question => SyntaxKind::QUESTION,
                pgls_tokenizer::TokenKind::Colon => SyntaxKind::COLON,
                pgls_tokenizer::TokenKind::DoubleColon => SyntaxKind::DOUBLE_COLON,
                pgls_tokenizer::TokenKind::ColonEquals => SyntaxKind::COLON_EQUALS,
                pgls_tokenizer::TokenKind::Operator => SyntaxKind::OPERATOR,
                pgls_tokenizer::TokenKind::Eq => SyntaxKind::EQ,
                pgls_tokenizer::TokenKind::Bang => SyntaxKind::BANG,
                pgls_tokenizer::TokenKind::Lt => SyntaxKind::L_ANGLE,
                pgls_tokenizer::TokenKind::Gt => SyntaxKind::R_ANGLE,
                pgls_tokenizer::TokenKind::Minus => SyntaxKind::MINUS,
                pgls_tokenizer::TokenKind::And => SyntaxKind::AMP,
                pgls_tokenizer::TokenKind::Or => SyntaxKind::PIPE,
                pgls_tokenizer::TokenKind::Plus => SyntaxKind::PLUS,
                pgls_tokenizer::TokenKind::Star => SyntaxKind::STAR,
                pgls_tokenizer::TokenKind::Slash => SyntaxKind::SLASH,
                pgls_tokenizer::TokenKind::Caret => SyntaxKind::CARET,
                pgls_tokenizer::TokenKind::Percent => SyntaxKind::PERCENT,
                pgls_tokenizer::TokenKind::Unknown => SyntaxKind::ERROR,
                pgls_tokenizer::TokenKind::Backslash => SyntaxKind::BACKSLASH,
                pgls_tokenizer::TokenKind::MetaCommand => SyntaxKind::META_COMMAND,
                pgls_tokenizer::TokenKind::UnknownPrefix => {
                    err = "unknown literal prefix";
                    SyntaxKind::IDENT
                }
                pgls_tokenizer::TokenKind::Eof => SyntaxKind::EOF,
                pgls_tokenizer::TokenKind::Backtick => SyntaxKind::BACKTICK,
                pgls_tokenizer::TokenKind::PositionalParam => SyntaxKind::POSITIONAL_PARAM,
                pgls_tokenizer::TokenKind::NamedParam { .. } => SyntaxKind::NAMED_PARAM,
                pgls_tokenizer::TokenKind::QuotedIdent { terminated } => {
                    if !terminated {
                        err = "Missing trailing \" to terminate the quoted identifier"
                    }
                    SyntaxKind::IDENT
                }
            }
        };

        let err = if err.is_empty() { None } else { Some(err) };
        self.push(syntax_kind, token_text.len(), err, line_ending_count);
    }

    fn extend_literal(&mut self, len: usize, kind: &pgls_tokenizer::LiteralKind) {
        let mut err = "";

        let syntax_kind = match *kind {
            pgls_tokenizer::LiteralKind::Int { empty_int, base: _ } => {
                if empty_int {
                    err = "Missing digits after the integer base prefix";
                }
                SyntaxKind::INT_NUMBER
            }
            pgls_tokenizer::LiteralKind::Float {
                empty_exponent,
                base: _,
            } => {
                if empty_exponent {
                    err = "Missing digits after the exponent symbol";
                }
                SyntaxKind::FLOAT_NUMBER
            }
            pgls_tokenizer::LiteralKind::Str { terminated } => {
                if !terminated {
                    err = "Missing trailing `'` symbol to terminate the string literal";
                }
                SyntaxKind::STRING
            }
            pgls_tokenizer::LiteralKind::ByteStr { terminated } => {
                if !terminated {
                    err = "Missing trailing `'` symbol to terminate the hex bit string literal";
                }
                SyntaxKind::BYTE_STRING
            }
            pgls_tokenizer::LiteralKind::BitStr { terminated } => {
                if !terminated {
                    err = "Missing trailing `'` symbol to terminate the bit string literal";
                }
                SyntaxKind::BIT_STRING
            }
            pgls_tokenizer::LiteralKind::DollarQuotedString { terminated } => {
                if !terminated {
                    err = "Unterminated dollar quoted string literal";
                }
                SyntaxKind::DOLLAR_QUOTED_STRING
            }
            pgls_tokenizer::LiteralKind::UnicodeEscStr { terminated } => {
                if !terminated {
                    err = "Missing trailing `'` symbol to terminate the unicode escape string literal";
                }
                SyntaxKind::BYTE_STRING
            }
            pgls_tokenizer::LiteralKind::EscStr { terminated } => {
                if !terminated {
                    err = "Missing trailing `'` symbol to terminate the escape string literal";
                }
                SyntaxKind::ESC_STRING
            }
        };

        let err = if err.is_empty() { None } else { Some(err) };
        self.push(syntax_kind, len, err, None);
    }
}
