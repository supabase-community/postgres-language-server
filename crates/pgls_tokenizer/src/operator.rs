//! Operators: a run of operator characters, as scan.l's `{operator}` rule cuts it.

use crate::TokenKind;
use crate::class::CharClass;
use crate::cursor::Cursor;

/// The operator characters that no SQL operator has. A run with one of them keeps a trailing `+`
/// or `-`.
const NOT_IN_SQL_OPERATORS: &str = "~!@#^&|`?%";

impl Cursor<'_> {
    /// Eats an operator after its first character `first`. Postgres reads a run of operator
    /// characters as one operator. The run stops before `/*` and `--`, which start a comment, and
    /// trailing `+` and `-` are put back unless the run has a character that no SQL operator has,
    /// so `=-1` is `=` and `-1`, but `?-` is one operator:
    /// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L886-L945
    ///
    /// The run also stops before a named parameter, so `id=@id` keeps `@id`. A one-character
    /// operator is the kind of its character, a longer one is [`TokenKind::Operator`].
    pub(crate) fn operator(&mut self, first: char) -> TokenKind {
        let rest = self.as_str();
        // The operator characters are ASCII.
        let bytes = rest.as_bytes();
        let mut len = 0;
        while let Some(&b) = bytes.get(len) {
            let comment = matches!(
                (b, bytes.get(len + 1)),
                (b'/', Some(b'*')) | (b'-', Some(b'-'))
            );
            if !CharClass::OpChar.contains(char::from(b))
                || comment
                || (b == b'@'
                    && Cursor::new(rest.get(len + 1..).unwrap_or_default()).at_identifier())
            {
                break;
            }
            len += 1;
        }
        let mut run = rest.get(..len).unwrap_or_default();
        if run.ends_with(['+', '-'])
            && !NOT_IN_SQL_OPERATORS.contains(first)
            && !run.contains(|c| NOT_IN_SQL_OPERATORS.contains(c))
        {
            run = run.trim_end_matches(['+', '-']);
        }
        if !run.is_empty() {
            self.bump_bytes(run.len());
            return TokenKind::Operator;
        }
        match first {
            '~' => TokenKind::Tilde,
            '!' => TokenKind::Bang,
            '@' => TokenKind::At,
            '#' => TokenKind::Pound,
            '^' => TokenKind::Caret,
            '&' => TokenKind::And,
            '|' => TokenKind::Or,
            '`' => TokenKind::Backtick,
            '?' => TokenKind::Question,
            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '*' => TokenKind::Star,
            '/' => TokenKind::Slash,
            '%' => TokenKind::Percent,
            '<' => TokenKind::Lt,
            '>' => TokenKind::Gt,
            '=' => TokenKind::Eq,
            _ => TokenKind::Unknown,
        }
    }
}
