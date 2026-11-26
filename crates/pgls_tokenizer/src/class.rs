/// A character class of Postgres' scanner, or of psql's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharClass {
    /// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L332
    /// `ident_start [A-Za-z\200-\377_]`
    IdentStart,
    /// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L333
    /// `ident_cont [A-Za-z\200-\377_0-9\$]`
    IdentCont,
    /// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L209
    /// `non_newline_space [ \t\f\v]`
    NonNewlineSpace,
    /// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L210
    /// `newline [\n\r]`
    Newline,
    /// The characters of operators.
    ///
    /// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L367
    /// `` op_chars [\~\!\@\#\^\&\|\`\?\+\-\*\/\%\<\>\=] ``
    OpChar,
    /// The characters that are a token of their own, unless they're in a longer operator.
    ///
    /// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L366
    /// `self [,()\[\].;\:\+\-\*\/\%\^\<\>\=]`
    SelfChar,
    /// The characters of a psql variable's name.
    ///
    /// https://github.com/postgres/postgres/blob/REL_18_6/src/fe_utils/psqlscan.l#L372
    /// `variable_char [A-Za-z\200-\377_0-9]`
    VariableChar,
}

impl CharClass {
    /// Whether `c` is in the class. `\200-\377` in a class are bytes, so every non-ASCII
    /// character is in a class that has them: all bytes of its UTF-8 encoding are in that range.
    pub fn contains(self, c: char) -> bool {
        match self {
            CharClass::IdentStart => matches!(c, 'a'..='z' | 'A'..='Z' | '_') || !c.is_ascii(),
            CharClass::IdentCont => {
                CharClass::IdentStart.contains(c) || matches!(c, '0'..='9' | '$')
            }
            CharClass::NonNewlineSpace => matches!(c, ' ' | '\t' | '\u{000C}' | '\u{000B}'),
            CharClass::Newline => matches!(c, '\n' | '\r'),
            CharClass::OpChar => "~!@#^&|`?+-*/%<>=".contains(c),
            CharClass::SelfChar => ",()[].;:+-*/%^<>=".contains(c),
            CharClass::VariableChar => CharClass::IdentStart.contains(c) || c.is_ascii_digit(),
        }
    }
}
