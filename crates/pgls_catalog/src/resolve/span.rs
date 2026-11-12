//! The spans of what findings and references point at: names, types, `*`, operators, casts,
//! constants and aliases. The parse tree only has the location where each starts, so the end is
//! found in the tokens of Postgres' scanner, scanned once per statement.

use pgls_query::protobuf::{KeywordKind, Token as Kind};

/// A token of the statement. Comments are left out.
#[derive(Debug, Clone, Copy)]
pub(super) struct Token {
    pub start: usize,
    pub end: usize,
    pub kind: Kind,
    /// Whether the token is a keyword, reserved or not.
    pub keyword: bool,
}

/// The tokens of a statement.
pub(super) struct Tokens<'a> {
    sql: &'a str,
    tokens: Vec<Token>,
}

impl<'a> Tokens<'a> {
    /// Scans `sql` with Postgres' scanner. `None` if the scanner rejects it, which the parser
    /// would have done too.
    pub fn scan(sql: &'a str) -> Option<Self> {
        let scanned = pgls_query::scan(sql).ok()?;
        let tokens = scanned
            .tokens
            .iter()
            .filter(|token| !matches!(token.token(), Kind::SqlComment | Kind::CComment))
            .filter_map(|token| {
                Some(Token {
                    start: usize::try_from(token.start).ok()?,
                    end: usize::try_from(token.end).ok()?,
                    kind: token.token(),
                    keyword: token.keyword_kind() != KeywordKind::NoKeyword,
                })
            })
            .filter(|token| sql.get(token.start..token.end).is_some())
            .collect();
        Some(Self { sql, tokens })
    }

    /// The tokens of the statement, in order.
    pub fn tokens(&self) -> &[Token] {
        &self.tokens
    }

    /// The text of `token`.
    pub fn text(&self, token: &Token) -> &'a str {
        self.sql.get(token.start..token.end).unwrap_or_default()
    }

    /// The index of the token that starts at `start`.
    fn at(&self, start: usize) -> Option<usize> {
        self.tokens
            .binary_search_by_key(&start, |token| token.start)
            .ok()
    }

    fn kind(&self, index: usize) -> Option<Kind> {
        self.tokens.get(index).map(|token| token.kind)
    }

    fn end(&self, index: usize) -> Option<usize> {
        self.tokens.get(index).map(|token| token.end)
    }

    /// Whether the token at `index` is an identifier, quoted or not, or a keyword.
    fn is_identifier(&self, index: usize) -> bool {
        self.tokens
            .get(index)
            .is_some_and(|token| token.keyword || matches!(token.kind, Kind::Ident | Kind::Uident))
    }

    /// Whether the token at `index` can be a part of a name: an identifier or keyword, or a
    /// number or parameter, which stand for themselves where an expression is expected.
    fn is_name_part(&self, index: usize) -> bool {
        self.is_identifier(index)
            || matches!(
                self.kind(index),
                Some(Kind::Iconst | Kind::Fconst | Kind::Param)
            )
    }

    /// Whether the token at `index` is the unquoted word `word`, in any case.
    fn is_word(&self, index: usize, word: &str) -> bool {
        self.tokens.get(index).is_some_and(|token| {
            (token.keyword || token.kind == Kind::Ident)
                && self.text(token).eq_ignore_ascii_case(word)
        })
    }

    /// The index of the last token of the (possibly qualified) name whose first token is at
    /// `index`. `None` if a dot isn't followed by another part, as in `t.*`.
    fn name_last(&self, mut index: usize) -> Option<usize> {
        loop {
            if !self.is_name_part(index) {
                return None;
            }
            if self.kind(index + 1) != Some(Kind::Ascii46) {
                return Some(index);
            }
            index += 2;
        }
    }

    /// The index of the `)` that closes the `(` at `open`.
    fn closing_paren(&self, open: usize) -> Option<usize> {
        let mut depth = 0usize;
        for (index, token) in self.tokens.iter().enumerate().skip(open) {
            match token.kind {
                Kind::Ascii40 => depth += 1,
                Kind::Ascii41 => {
                    depth = depth.checked_sub(1)?;
                    if depth == 0 {
                        return Some(index);
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// The end of the (possibly qualified) name starting at `start`, like `public.users`.
    pub fn name_end(&self, start: usize) -> Option<usize> {
        self.end(self.name_last(self.at(start)?)?)
    }

    /// The end of the type name starting at `start`, with all the words of the SQL-standard
    /// multi-word types like `double precision`, `character varying`, or `timestamp with time
    /// zone`. Modifiers are left out, except the precision of `time` and `timestamp`, which can
    /// come before `with time zone`.
    pub fn type_name_end(&self, start: usize) -> Option<usize> {
        let first = self.at(start)?;
        let last = self.name_last(first)?;
        // The index of the last of `words`, if they follow `index`.
        let words = |index: usize, words: &[&str]| {
            words.iter().try_fold(index, |index, word| {
                self.is_word(index + 1, word).then_some(index + 1)
            })
        };
        let word = |word: &str| words(last, &[word]);
        let is = |word: &str| first == last && self.is_word(first, word);
        let last = if is("double") {
            word("precision").unwrap_or(last)
        } else if ["character", "char", "nchar", "bit"].into_iter().any(is) {
            word("varying").unwrap_or(last)
        } else if is("national") {
            let last = word("character").or_else(|| word("char")).unwrap_or(last);
            words(last, &["varying"]).unwrap_or(last)
        } else if is("time") || is("timestamp") {
            let precision = if self.kind(last + 1) == Some(Kind::Ascii40) {
                (last + 1..self.tokens.len())
                    .find(|&index| self.kind(index) == Some(Kind::Ascii41))
                    .unwrap_or(last)
            } else {
                last
            };
            words(precision, &["with", "time", "zone"])
                .or_else(|| words(precision, &["without", "time", "zone"]))
                .unwrap_or(last)
        } else {
            last
        };
        self.end(last)
    }

    /// The end of `*`, or of a qualified `t.*` or `"s"."t".*`, starting at `start`.
    pub fn star_end(&self, start: usize) -> Option<usize> {
        let mut index = self.at(start)?;
        loop {
            match self.kind(index)? {
                Kind::Ascii42 => return self.end(index),
                Kind::Ascii46 => {}
                _ if self.is_name_part(index) => {}
                _ => return None,
            }
            index += 1;
        }
    }

    /// The span of the alias after the (possibly qualified) name that starts at `start`, with or
    /// without `AS`, like `u` in `users as u` or `"U"` in `public.users "U"`. `None` if no
    /// identifier follows.
    pub fn alias_span(&self, start: usize) -> Option<(usize, usize)> {
        let mut index = self.at(start)?;
        loop {
            if !self.is_identifier(index) {
                return None;
            }
            index += 1;
            if self.kind(index) != Some(Kind::Ascii46) {
                break;
            }
            index += 1;
        }
        if self.kind(index) == Some(Kind::As) {
            index += 1;
        }
        if !self.is_identifier(index) {
            return None;
        }
        let alias = self.tokens.get(index)?;
        Some((alias.start, alias.end))
    }

    /// The end of the token starting at `start`. `None` for parentheses, commas and semicolons.
    fn token_end(&self, start: usize) -> Option<usize> {
        let token = self.tokens.get(self.at(start)?)?;
        if matches!(
            token.kind,
            Kind::Ascii40 | Kind::Ascii41 | Kind::Ascii44 | Kind::Ascii59
        ) {
            return None;
        }
        Some(token.end)
    }

    /// The end of the first token of the expression starting at `start`, or of the whole name if
    /// it starts with one.
    pub fn expression_end(&self, start: usize) -> Option<usize> {
        self.name_end(start).or_else(|| self.token_end(start))
    }

    /// The end of the operator starting at `start`: a single token, or `OPERATOR(schema.op)`.
    pub fn operator_end(&self, start: usize) -> Option<usize> {
        let index = self.at(start)?;
        if self.kind(index) == Some(Kind::Operator) && self.kind(index + 1) == Some(Kind::Ascii40) {
            let close = (index + 2..self.tokens.len())
                .find(|&index| self.kind(index) == Some(Kind::Ascii41))?;
            return self.end(close);
        }
        self.token_end(start)
    }

    /// The end of the cast starting at `start`: `CAST(x AS t)`, `::t` with the modifiers of `t`,
    /// or a typed constant like `date 'today'`. The type name of `::t` and `t '...'` ends after
    /// its first word.
    pub fn cast_end(&self, start: usize) -> Option<usize> {
        let mut index = self.at(start)?;
        if self.kind(index) == Some(Kind::Cast) && self.kind(index + 1) == Some(Kind::Ascii40) {
            return self.end(self.closing_paren(index + 1)?);
        }
        if self.kind(index) == Some(Kind::Typecast) {
            index += 1;
        }
        let last = self.name_last(index)?;
        if matches!(self.kind(last + 1), Some(Kind::Sconst | Kind::Usconst)) {
            return self.end(last + 1);
        }
        let end = self.end(last)?;
        let modifiers = self
            .tokens
            .get(last + 1)
            .filter(|open| open.kind == Kind::Ascii40 && open.start == end);
        if modifiers.is_some() {
            return self.end(self.closing_paren(last + 1)?);
        }
        Some(end)
    }

    /// The end of the string constant starting at `start`: `'...'`, with an `E`, `B`, `X`, `N`,
    /// or `U&` prefix, or `$tag$...$tag$`. Constants continued on a new line (`'a'\n'b'`) end
    /// after their last part.
    pub fn literal_end(&self, start: usize) -> Option<usize> {
        let index = self.at(start)?;
        let token = self.tokens.get(index)?;
        match token.kind {
            Kind::Sconst | Kind::Usconst | Kind::Bconst | Kind::Xconst => Some(token.end),
            // The scanner returns `N'...'` as the keyword `NCHAR` for the `N`, then the string.
            Kind::Nchar if token.end - token.start == 1 => self
                .tokens
                .get(index + 1)
                .filter(|string| string.kind == Kind::Sconst && string.start == token.end)
                .map(|string| string.end),
            _ => None,
        }
    }
}

#[cfg(test)]
mod regress_tests;

#[cfg(test)]
mod tests {
    use super::Tokens;

    fn reference_end(sql: &str, start: usize) -> Option<usize> {
        Tokens::scan(sql)?.name_end(start)
    }

    fn type_name_end(sql: &str, start: usize) -> Option<usize> {
        Tokens::scan(sql)?.type_name_end(start)
    }

    fn star_end(sql: &str, start: usize) -> Option<usize> {
        Tokens::scan(sql)?.star_end(start)
    }

    fn alias_span(sql: &str, start: usize) -> Option<(usize, usize)> {
        Tokens::scan(sql)?.alias_span(start)
    }

    fn token_end(sql: &str, start: usize) -> Option<usize> {
        Tokens::scan(sql)?.token_end(start)
    }

    fn cast_end(sql: &str, start: usize) -> Option<usize> {
        Tokens::scan(sql)?.cast_end(start)
    }

    fn literal_end(sql: &str, start: usize) -> Option<usize> {
        Tokens::scan(sql)?.literal_end(start)
    }

    fn operator_end(sql: &str, start: usize) -> Option<usize> {
        Tokens::scan(sql)?.operator_end(start)
    }

    fn expression_end(sql: &str, start: usize) -> Option<usize> {
        Tokens::scan(sql)?.expression_end(start)
    }

    /// The text of the span that `end_fn` finds at the first `at` in `sql`.
    fn span_at<'a>(
        sql: &'a str,
        at: &str,
        end_fn: fn(&str, usize) -> Option<usize>,
    ) -> Option<&'a str> {
        let start = sql.find(at).expect("in sql");
        Some(&sql[start..end_fn(sql, start)?])
    }

    #[test]
    fn unicode_escaped_identifiers_are_one_token() {
        let sql = r#"select U&"d\0061t" from U&"s".t U&"x", U&"s" . *"#;
        assert_eq!(span_at(sql, "U&\"d", reference_end), Some(r#"U&"d\0061t""#));
        assert_eq!(span_at(sql, "U&\"s\".t", reference_end), Some(r#"U&"s".t"#));
        assert_eq!(
            alias_span(sql, sql.find("U&\"s\".t").unwrap()).map(|(start, end)| &sql[start..end]),
            Some(r#"U&"x""#)
        );
        assert_eq!(span_at(sql, "U&\"s\" .", star_end), Some(r#"U&"s" . *"#));
    }

    #[test]
    fn comments_are_skipped_like_whitespace() {
        let sql = "select public /* c */ . users, x:: -- c\n int from t";
        assert_eq!(
            span_at(sql, "public", reference_end),
            Some("public /* c */ . users")
        );
        assert_eq!(span_at(sql, "::", cast_end), Some(":: -- c\n int"));
    }

    #[test]
    fn whitespace_before_the_parenthesis_of_cast_and_operator() {
        let sql = "select CAST (x AS int), a OPERATOR (pg_catalog.+) b";
        assert_eq!(span_at(sql, "CAST", cast_end), Some("CAST (x AS int)"));
        assert_eq!(
            span_at(sql, "OPERATOR", operator_end),
            Some("OPERATOR (pg_catalog.+)")
        );
    }

    #[test]
    fn parentheses_in_string_constants() {
        let sql = "select CAST(')' AS text), x::varchar(10) || ')'";
        assert_eq!(span_at(sql, "CAST", cast_end), Some("CAST(')' AS text)"));
        assert_eq!(span_at(sql, "::", cast_end), Some("::varchar(10)"));
    }

    #[test]
    fn casts_at_the_end_of_the_statement() {
        assert_eq!(span_at("select x::t", "::", cast_end), Some("::t"));
        assert_eq!(span_at("select x::t;", "::", cast_end), Some("::t"));
    }

    #[test]
    fn typed_constants_with_escapes_and_dollar_quotes() {
        let sql = "select date E'today', date $$today$$";
        assert_eq!(span_at(sql, "date E", cast_end), Some("date E'today'"));
        assert_eq!(span_at(sql, "date $", cast_end), Some("date $$today$$"));
    }

    #[test]
    fn expressions_start_with_a_whole_token() {
        let sql = "select 'it''s', E'a', a='x', 1.5e-3, X'1F'";
        assert_eq!(span_at(sql, "'it", expression_end), Some("'it''s'"));
        assert_eq!(span_at(sql, "E'", expression_end), Some("E'a'"));
        assert_eq!(span_at(sql, "=", expression_end), Some("="));
        assert_eq!(span_at(sql, "=", operator_end), Some("="));
        assert_eq!(span_at(sql, "1.5", expression_end), Some("1.5e-3"));
        assert_eq!(span_at(sql, "1.5", reference_end), Some("1.5e-3"));
        assert_eq!(span_at(sql, "X'", expression_end), Some("X'1F'"));
        // String constants are not names, whatever their prefix.
        assert_eq!(span_at(sql, "E'", reference_end), None);
        assert_eq!(span_at(sql, "X'", reference_end), None);
        assert_eq!(span_at("select $$a$$", "$$", reference_end), None);
    }

    #[test]
    fn alias_spans() {
        fn alias(sql: &str) -> Option<&str> {
            alias_span(sql, 0).map(|(start, end)| &sql[start..end])
        }
        assert_eq!(alias("users u where"), Some("u"));
        assert_eq!(alias("users AS u"), Some("u"));
        assert_eq!(alias("public . users as\n  \"U s\"(a)"), Some("\"U s\""));
        assert_eq!(
            alias("users /* a /* nested */ c */ -- x\n asmodeus"),
            Some("asmodeus")
        );
        assert_eq!(alias("public /* c */ . users u"), Some("u"));
        assert_eq!(alias("users"), None);
        assert_eq!(alias("users, posts"), None);
    }

    fn text(sql: &str, start: usize, end: Option<usize>) -> &str {
        &sql[start..end.expect("span")]
    }

    #[test]
    fn type_finding_spans_cover_the_diagnostic_tokens() {
        let sql = "select a + b, OPERATOR(pg_catalog.+)(a,b), pg_catalog.f(a), x::bad, CAST(x AS bad), date 'today'";
        let start = sql.find('+').unwrap();
        assert_eq!(text(sql, start, operator_end(sql, start)), "+");
        let start = sql.find("OPERATOR").unwrap();
        assert_eq!(
            text(sql, start, operator_end(sql, start)),
            "OPERATOR(pg_catalog.+)"
        );
        let sql_lower = "select operator(pg_catalog.||)(a, b)";
        let start = sql_lower.find("operator").unwrap();
        assert_eq!(
            text(sql_lower, start, operator_end(sql_lower, start)),
            "operator(pg_catalog.||)"
        );
        let start = sql.find("pg_catalog.f").unwrap();
        assert_eq!(text(sql, start, reference_end(sql, start)), "pg_catalog.f");
        let start = sql.find("::bad").unwrap();
        assert_eq!(text(sql, start, cast_end(sql, start)), "::bad");
        let start = sql.find("CAST(").unwrap();
        assert_eq!(text(sql, start, cast_end(sql, start)), "CAST(x AS bad)");
        let start = sql.find("date '").unwrap();
        assert_eq!(text(sql, start, cast_end(sql, start)), "date 'today'");
    }

    #[test]
    fn type_name_spans_cover_every_word() {
        let cases = [
            ("int8[]", "int8"),
            ("pg_catalog.int4)", "pg_catalog.int4"),
            ("double precision, x", "double precision"),
            ("DOUBLE\n  PRECISION", "DOUBLE\n  PRECISION"),
            ("character varying(10)", "character varying"),
            ("character(10)", "character"),
            ("bit varying", "bit varying"),
            ("national character varying", "national character varying"),
            ("timestamp with time zone 'now'", "timestamp with time zone"),
            (
                "timestamp(3) without time zone",
                "timestamp(3) without time zone",
            ),
            ("timestamp(3)", "timestamp"),
            ("time with x", "time"),
            ("timestamptz", "timestamptz"),
            ("double precisionx", "double"),
            ("\"double\" precision", "\"double\""),
        ];
        for (sql, expected) in cases {
            assert_eq!(text(sql, 0, type_name_end(sql, 0)), expected, "{sql}");
        }
    }

    #[test]
    fn literal_spans_cover_the_whole_constant() {
        let cases = [
            ("'happy'", "'happy'"),
            ("'it''s' = x", "'it''s'"),
            ("E'a\\'b' || x", "E'a\\'b'"),
            ("U&'d\\0061t' x", "U&'d\\0061t'"),
            ("$$a'b$$ x", "$$a'b$$"),
            ("$t$a$$b$t$ x", "$t$a$$b$t$"),
            ("'a'\n  'b' x", "'a'\n  'b'"),
            ("'a' 'b'", "'a'"),
            ("N'abc' x", "N'abc'"),
            ("B'101' x", "B'101'"),
            ("x'1F' x", "x'1F'"),
            ("$ä$a$ä$ x", "$ä$a$ä$"),
        ];
        for (sql, expected) in cases {
            assert_eq!(text(sql, 0, literal_end(sql, 0)), expected, "{sql}");
        }
        assert_eq!(literal_end("x", 0), None);
        assert_eq!(literal_end("'open", 0), None);
    }

    #[test]
    fn star_spans_cover_the_qualifier() {
        let sql = r#"select *, t.*, "My ""T""" . *, s.t.* from t"#;
        let start = sql.find('*').unwrap();
        assert_eq!(text(sql, start, star_end(sql, start)), "*");
        let start = sql.find("t.*").unwrap();
        assert_eq!(text(sql, start, star_end(sql, start)), "t.*");
        let start = sql.find("\"My").unwrap();
        assert_eq!(text(sql, start, star_end(sql, start)), r#""My ""T""" . *"#);
        let start = sql.find("s.t").unwrap();
        assert_eq!(text(sql, start, star_end(sql, start)), "s.t.*");
    }

    #[test]
    fn non_ascii_identifiers() {
        let sql = "select größe, élan, café.öl, \"Ölstand\", x::straße from t";
        for name in ["größe", "élan", "café.öl", "\"Ölstand\""] {
            let start = sql.find(name).unwrap();
            assert_eq!(text(sql, start, reference_end(sql, start)), name);
            assert_eq!(
                text(sql, start, token_end(sql, start)),
                name.split('.').next().unwrap()
            );
        }
        let start = sql.find("::").unwrap();
        assert_eq!(text(sql, start, cast_end(sql, start)), "::straße");
    }
}
