// based on: https://github.com/rust-lang/rust/blob/d1b7355d3d7b4ead564dbecb1d240fcc74fff21b/compiler/rustc_lexer/src/lib.rs#L58
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenKind {
    /// Used when there's an error of some sort while lexing.
    Unknown,
    /// Examples: `12u8`, `1.0e-40`, `b"123"`. Note that `_` is an invalid
    /// suffix, but may be present here on string and float literals. Users of
    /// this type will need to check for and reject that case.
    ///
    /// See [`LiteralKind`] for more details.
    Literal {
        kind: LiteralKind,
    },
    /// Whitespace characters.
    Space,
    Tab,
    VerticalTab,
    FormFeed,
    // Handles \n, \r, and sequences
    LineEnding {
        count: usize,
    },
    /// Identifier
    ///
    /// case-sensitive
    Ident,
    /// `;`
    Semi,
    /// End of file
    Eof,
    /// `/`
    Slash,
    /// `\`
    Backslash,
    /// `-- foo`
    LineComment,
    /// ```
    /// /*
    /// foo
    /// */
    /// ```
    BlockComment {
        terminated: bool,
    },
    /// `-`
    Minus,
    /// `:`
    Colon,
    /// `::`
    DoubleColon,
    /// `.`
    Dot,
    /// `=`
    Eq,
    /// `>`
    Gt,
    /// `&`
    And,
    /// `<`
    Lt,
    /// `!`
    Bang,
    /// `+`
    Plus,
    /// `~`
    Tilde,
    /// `#`
    Pound,
    /// `?`
    Question,
    /// `|`
    Or,
    /// `%`
    Percent,
    /// `^`
    Caret,
    /// `*`
    Star,
    /// `` ` ``
    Backtick,
    /// `@`
    At,
    /// `]`
    CloseBracket,
    /// `[`
    OpenBracket,
    /// `)`
    CloseParen,
    /// `(`
    OpenParen,
    /// `,`
    Comma,
    /// Error case that we need to report later on.
    UnknownPrefix,
    /// Positional Parameter, e.g., `$1`
    ///
    /// see: <https://www.postgresql.org/docs/16/sql-expressions.html#SQL-EXPRESSIONS-PARAMETERS-POSITIONAL>
    PositionalParam,
    /// Named Parameter, e.g., `@name`
    ///
    /// This is used in some ORMs and query builders, like sqlc.
    NamedParam {
        kind: NamedParamKind,
    },
    /// Quoted Identifier, e.g., `"update"` in `update "my_table" set "a" = 5;`
    ///
    /// These are case-sensitive, unlike [`TokenKind::Ident`]
    ///
    /// see: <https://www.postgresql.org/docs/16/sql-syntax-lexical.html#SQL-SYNTAX-IDENTIFIERS>
    QuotedIdent {
        terminated: bool,
    },
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum NamedParamKind {
    /// e.g. `@name`
    ///
    /// Used in:
    /// - sqlc: https://docs.sqlc.dev/en/latest/howto/named_parameters.html
    AtPrefix,

    /// e.g. `:name` (raw substitution)
    ///
    /// Used in: psql
    ColonRaw,

    /// e.g. `:'name'` (quoted string substitution)
    ///
    /// Used in: psql
    ColonString { terminated: bool },

    /// e.g. `:"name"` (quoted identifier substitution)
    ///
    /// Used in: psql
    ColonIdentifier { terminated: bool },

    /// e.g. `$name`
    DollarRaw,
}

/// Parsed token.
/// It doesn't contain information about data that has been parsed,
/// only the type of the token and its size.
#[derive(Debug, Clone, Copy)]
pub struct Token {
    pub kind: TokenKind,
    pub len: u32,
}

impl Token {
    pub(crate) fn new(kind: TokenKind, len: u32) -> Token {
        Token { kind, len }
    }
}

/// Base of numeric literal encoding according to its prefix.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Base {
    /// Literal starts with "0b".
    Binary = 2,
    /// Literal starts with "0o".
    Octal = 8,
    /// Literal doesn't contain a prefix.
    Decimal = 10,
    /// Literal starts with "0x".
    Hexadecimal = 16,
}

// Enum representing the literal types supported by the lexer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LiteralKind {
    /// Integer Numeric, e.g., `42`
    ///
    /// see: <https://www.postgresql.org/docs/16/sql-syntax-lexical.html#SQL-SYNTAX-CONSTANTS-NUMERIC>
    Int { base: Base, empty_int: bool },
    /// Float Numeric, e.g., `1.925e-3`
    ///
    /// see: <https://www.postgresql.org/docs/16/sql-syntax-lexical.html#SQL-SYNTAX-CONSTANTS-NUMERIC>
    Float { base: Base, empty_exponent: bool },
    /// String, e.g., `'foo'`
    ///
    /// see: <https://www.postgresql.org/docs/16/sql-syntax-lexical.html#SQL-SYNTAX-STRINGS>
    Str { terminated: bool },
    /// Hexidecimal Bit String, e.g., `X'1FF'`
    ///
    /// see: <https://www.postgresql.org/docs/16/sql-syntax-lexical.html#SQL-SYNTAX-BIT-STRINGS>
    ByteStr { terminated: bool },
    /// Bit String, e.g., `B'1001'`
    ///
    /// see: <https://www.postgresql.org/docs/16/sql-syntax-lexical.html#SQL-SYNTAX-BIT-STRINGS>
    BitStr { terminated: bool },
    /// Dollar Quoted String, e.g., `$$Dianne's horse$$`
    ///
    /// see: <https://www.postgresql.org/docs/16/sql-syntax-lexical.html#SQL-SYNTAX-DOLLAR-QUOTING>
    DollarQuotedString { terminated: bool },
    /// Unicode Escape String, e.g., `U&'d\0061t\+000061'`
    ///
    /// see: <https://www.postgresql.org/docs/16/sql-syntax-lexical.html#SQL-SYNTAX-STRINGS-UESCAPE>
    UnicodeEscStr { terminated: bool },
    /// Escape String, e.g, `E'foo'`
    ///
    /// see: <https://www.postgresql.org/docs/16/sql-syntax-lexical.html>
    EscStr { terminated: bool },
}
