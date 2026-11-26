use crate::cursor::Cursor;
use crate::{Base, LiteralKind};

impl Base {
    /// The base of a `0b`, `0o` or `0x` prefix, by its letter.
    fn of_prefix(letter: char) -> Option<Base> {
        match letter {
            'b' | 'B' => Some(Base::Binary),
            'o' | 'O' => Some(Base::Octal),
            'x' | 'X' => Some(Base::Hexadecimal),
            _ => None,
        }
    }
}

// The number rules of Postgres' scanner, in their order:
// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L386-L399
//
// The junk rules (`integer_junk`, ...) aren't read: `1abc` is a number and an identifier.
impl Cursor<'_> {
    /// Eats a number after its first digit.
    ///
    /// https://www.postgresql.org/docs/current/sql-syntax-lexical.html#SQL-SYNTAX-CONSTANTS-NUMERIC
    pub(crate) fn number(&mut self, first_digit: char) -> LiteralKind {
        // hexinteger		0[xX](_?{hexdigit})+
        // octinteger		0[oO](_?{octdigit})+
        // bininteger		0[bB](_?{bindigit})+
        //
        // These have no fraction or exponent, so `0b1.a` is `0b1`, `.` and `a`.
        if first_digit == '0'
            && let Some(base) = Base::of_prefix(self.first())
        {
            self.bump();
            let empty_int = !self.eat_digits(base);
            return LiteralKind::Int { base, empty_int };
        }

        // decinteger		{decdigit}(_?{decdigit})*
        self.eat_digits(Base::Decimal);

        // numeric			(({decinteger}\.{decinteger}?)|(\.{decinteger}))
        // numericfail		{decinteger}\.\.
        //
        // So `1..10` is `1`, `..` and `10`.
        if self.first() == '.' && self.second() != '.' {
            self.bump();
            return self.fraction();
        }

        self.exponent(LiteralKind::Int {
            base: Base::Decimal,
            empty_int: false,
        })
    }

    /// Eats the rest of a number after its `.`: the digits of the fraction, if any, and the
    /// exponent. A number has one fraction, so `.1.2` is `.1` and `.2`.
    pub(crate) fn fraction(&mut self) -> LiteralKind {
        if self.first().is_ascii_digit() {
            self.eat_digits(Base::Decimal);
        }
        self.exponent(LiteralKind::Float {
            base: Base::Decimal,
            empty_exponent: false,
        })
    }

    /// Eats the exponent of a number, if one follows, or returns `number`.
    ///
    /// `real ({decinteger}|{numeric})[Ee][-+]?{decinteger}`
    /// `realfail ({decinteger}|{numeric})[Ee][-+]`
    fn exponent(&mut self, number: LiteralKind) -> LiteralKind {
        if !matches!(self.first(), 'e' | 'E') {
            return number;
        }
        self.bump();
        if matches!(self.first(), '-' | '+') {
            self.bump();
        }
        LiteralKind::Float {
            base: Base::Decimal,
            empty_exponent: !self.eat_digits(Base::Decimal),
        }
    }

    /// Eats the digits of a number in `base`, and `_` between them, and returns true if there
    /// was a digit. A binary or octal number takes decimal digits too, so `0b12` is one token,
    /// which Postgres rejects as junk.
    fn eat_digits(&mut self, base: Base) -> bool {
        let mut has_digits = false;
        loop {
            match self.first() {
                '_' => {}
                c if c.is_ascii_digit() || (base == Base::Hexadecimal && c.is_ascii_hexdigit()) => {
                    has_digits = true;
                }
                _ => return has_digits,
            }
            self.bump();
        }
    }
}
