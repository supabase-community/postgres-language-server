use pgls_text_size::{TextRange, TextSize};
use std::iter::Peekable;

pub(crate) struct TokenNavigator {
    tokens: Peekable<std::vec::IntoIter<WordWithIndex>>,
    pub previous_token: Option<WordWithIndex>,
    pub current_token: Option<WordWithIndex>,
}

impl TokenNavigator {
    pub(crate) fn next_matches(&mut self, options: &[&str]) -> bool {
        self.tokens
            .peek()
            .is_some_and(|c| options.contains(&c.get_word_without_quotes().as_str()))
    }

    pub(crate) fn advance(&mut self) -> Option<WordWithIndex> {
        // we can't peek back n an iterator, so we'll have to keep track manually.
        self.previous_token = self.current_token.take();
        self.current_token = self.tokens.next();
        self.current_token.clone()
    }
}

impl From<Vec<WordWithIndex>> for TokenNavigator {
    fn from(tokens: Vec<WordWithIndex>) -> Self {
        TokenNavigator {
            tokens: tokens.into_iter().peekable(),
            previous_token: None,
            current_token: None,
        }
    }
}

pub(crate) trait CompletionStatementParser: Sized {
    type Context: Default;
    const NAME: &'static str;

    fn looks_like_matching_stmt(sql: &str) -> bool;
    fn parse(self) -> Self::Context;
    fn make_parser(tokens: Vec<WordWithIndex>, cursor_position: usize) -> Self;

    fn get_context(sql: &str, cursor_position: usize) -> Self::Context {
        assert!(
            Self::looks_like_matching_stmt(sql),
            "Using {} for a wrong statement! Developer Error!",
            Self::NAME
        );

        match sql_to_words(sql) {
            Ok(tokens) => {
                let parser = Self::make_parser(tokens, cursor_position);
                parser.parse()
            }
            Err(_) => Self::Context::default(),
        }
    }
}

pub(crate) fn schema_and_table_name(token: &WordWithIndex) -> (String, Option<String>) {
    let word = token.get_word_without_quotes();
    let mut parts = word.split('.');

    (
        parts.next().unwrap().into(),
        parts.next().map(|tb| tb.into()),
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WordWithIndex {
    word: String,
    start: usize,
    end: usize,
}

impl WordWithIndex {
    pub(crate) fn is_under_cursor(&self, cursor_pos: usize) -> bool {
        self.start <= cursor_pos && self.end > cursor_pos
    }

    pub(crate) fn get_range(&self) -> TextRange {
        let start: u32 = self.start.try_into().expect("Text too long");
        let end: u32 = self.end.try_into().expect("Text too long");
        TextRange::new(TextSize::from(start), TextSize::from(end))
    }

    pub(crate) fn get_word_without_quotes(&self) -> String {
        self.word.replace('"', "")
    }

    pub(crate) fn get_word(&self) -> String {
        self.word.clone()
    }
}

pub(crate) struct SubStatementParser {
    start_of_word: Option<usize>,
    current_word: String,
    in_quotation_marks: bool,
    is_fn_call: bool,
    words: Vec<WordWithIndex>,
}

impl SubStatementParser {
    pub(crate) fn parse(sql: &str) -> Result<Vec<WordWithIndex>, String> {
        let mut parser = SubStatementParser {
            start_of_word: None,
            current_word: String::new(),
            in_quotation_marks: false,
            is_fn_call: false,
            words: vec![],
        };

        parser.collect_words(sql);

        if parser.in_quotation_marks {
            Err("String was not closed properly.".into())
        } else {
            Ok(parser.words)
        }
    }

    pub fn collect_words(&mut self, sql: &str) {
        for (pos, c) in sql.char_indices() {
            match c {
                '"' => {
                    if !self.has_started_word() {
                        self.in_quotation_marks = true;
                        self.add_char(c);
                        self.start_word(pos);
                    } else {
                        self.in_quotation_marks = false;
                        self.add_char(c);
                    }
                }

                '(' => {
                    if !self.has_started_word() {
                        self.push_char_as_word(c, pos);
                    } else {
                        self.add_char(c);
                        self.is_fn_call = true;
                    }
                }

                ')' => {
                    if self.is_fn_call {
                        self.add_char(c);
                        self.is_fn_call = false;
                    } else {
                        if self.has_started_word() {
                            self.push_word(pos);
                        }
                        self.push_char_as_word(c, pos);
                    }
                }

                _ => {
                    if c.is_ascii_whitespace() || c == ';' {
                        if self.in_quotation_marks {
                            self.add_char(c);
                        } else if !self.is_empty() && self.has_started_word() {
                            self.push_word(pos);
                        }
                    } else if self.has_started_word() {
                        self.add_char(c);
                    } else {
                        self.start_word(pos);
                        self.add_char(c)
                    }
                }
            }
        }

        if self.has_started_word() && !self.is_empty() {
            self.push_word(sql.len())
        }
    }

    fn is_empty(&self) -> bool {
        self.current_word.is_empty()
    }

    fn add_char(&mut self, c: char) {
        self.current_word.push(c)
    }

    fn start_word(&mut self, pos: usize) {
        self.start_of_word = Some(pos);
    }

    fn has_started_word(&self) -> bool {
        self.start_of_word.is_some()
    }

    fn push_char_as_word(&mut self, c: char, pos: usize) {
        self.words.push(WordWithIndex {
            word: String::from(c),
            start: pos,
            end: pos + 1,
        });
    }

    fn push_word(&mut self, current_position: usize) {
        self.words.push(WordWithIndex {
            word: self.current_word.clone(),
            start: self.start_of_word.unwrap(),
            end: current_position,
        });
        self.current_word = String::new();
        self.start_of_word = None;
    }
}

/// Note: A policy name within quotation marks will be considered a single word.
pub(crate) fn sql_to_words(sql: &str) -> Result<Vec<WordWithIndex>, String> {
    SubStatementParser::parse(sql)
}

#[cfg(test)]
mod tests {
    use crate::context::base_parser::{SubStatementParser, WordWithIndex, sql_to_words};

    #[test]
    fn determines_positions_correctly() {
        let query = "\ncreate policy \"my cool pol\"\n\ton auth.users\n\tas permissive\n\tfor select\n\t\tto   public\n\t\tusing (auth.uid());".to_string();

        let words = SubStatementParser::parse(query.as_str()).unwrap();

        assert_eq!(words[0], to_word("create", 1, 7));
        assert_eq!(words[1], to_word("policy", 8, 14));
        assert_eq!(words[2], to_word("\"my cool pol\"", 15, 28));
        assert_eq!(words[3], to_word("on", 30, 32));
        assert_eq!(words[4], to_word("auth.users", 33, 43));
        assert_eq!(words[5], to_word("as", 45, 47));
        assert_eq!(words[6], to_word("permissive", 48, 58));
        assert_eq!(words[7], to_word("for", 60, 63));
        assert_eq!(words[8], to_word("select", 64, 70));
        assert_eq!(words[9], to_word("to", 73, 75));
        assert_eq!(words[10], to_word("public", 78, 84));
        assert_eq!(words[11], to_word("using", 87, 92));
        assert_eq!(words[12], to_word("(", 93, 94));
        assert_eq!(words[13], to_word("auth.uid()", 94, 104));
        assert_eq!(words[14], to_word(")", 104, 105));
    }

    #[test]
    fn handles_schemas_in_quotation_marks() {
        let query = r#"grant select on "public"."users""#.to_string();

        let words = sql_to_words(query.as_str()).unwrap();

        assert_eq!(words[0], to_word("grant", 0, 5));
        assert_eq!(words[1], to_word("select", 6, 12));
        assert_eq!(words[2], to_word("on", 13, 15));
        assert_eq!(words[3], to_word(r#""public"."users""#, 16, 32));
    }

    fn to_word(word: &str, start: usize, end: usize) -> WordWithIndex {
        WordWithIndex {
            word: word.into(),
            start,
            end,
        }
    }
}
