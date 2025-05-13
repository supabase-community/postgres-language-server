use std::iter::Peekable;

use pgt_text_size::{TextRange, TextSize};

#[derive(Default, Debug, PartialEq, Eq)]
pub(crate) enum PolicyStmtKind {
    #[default]
    Create,

    Alter,
    Drop,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct WordWithIndex {
    word: String,
    start: usize,
    end: usize,
}

impl WordWithIndex {
    fn is_under_cursor(&self, cursor_pos: usize) -> bool {
        self.start <= cursor_pos && self.end > cursor_pos
    }

    fn get_range(&self) -> TextRange {
        let start: u32 = self.start.try_into().expect("Text too long");
        let end: u32 = self.end.try_into().expect("Text too long");
        TextRange::new(TextSize::from(start), TextSize::from(end))
    }
}

/// Note: A policy name within quotation marks will be considered a single word.
fn sql_to_words(sql: &str) -> Result<Vec<WordWithIndex>, String> {
    let mut words = vec![];

    let mut start_of_word: Option<usize> = None;
    let mut current_word = String::new();
    let mut in_quotation_marks = false;

    for (current_position, current_char) in sql.char_indices() {
        if (current_char.is_ascii_whitespace() || current_char == ';')
            && !current_word.is_empty()
            && start_of_word.is_some()
            && !in_quotation_marks
        {
            words.push(WordWithIndex {
                word: current_word,
                start: start_of_word.unwrap(),
                end: current_position,
            });

            current_word = String::new();
            start_of_word = None;
        } else if (current_char.is_ascii_whitespace() || current_char == ';')
            && current_word.is_empty()
        {
            // do nothing
        } else if current_char == '"' && start_of_word.is_none() {
            in_quotation_marks = true;
            current_word.push(current_char);
            start_of_word = Some(current_position);
        } else if current_char == '"' && start_of_word.is_some() {
            current_word.push(current_char);
            words.push(WordWithIndex {
                word: current_word,
                start: start_of_word.unwrap(),
                end: current_position + 1,
            });
            in_quotation_marks = false;
            start_of_word = None;
            current_word = String::new()
        } else if start_of_word.is_some() {
            current_word.push(current_char)
        } else {
            start_of_word = Some(current_position);
            current_word.push(current_char);
        }
    }

    if let Some(start_of_word) = start_of_word {
        if !current_word.is_empty() {
            words.push(WordWithIndex {
                word: current_word,
                start: start_of_word,
                end: sql.len(),
            });
        }
    }

    if in_quotation_marks {
        Err("String was not closed properly.".into())
    } else {
        Ok(words)
    }
}

#[derive(Default, Debug, PartialEq, Eq)]
pub(crate) struct PolicyContext {
    pub policy_name: Option<String>,
    pub table_name: Option<String>,
    pub schema_name: Option<String>,
    pub statement_kind: PolicyStmtKind,
    pub node_text: String,
    pub node_range: TextRange,
    pub node_kind: String,
}

/// Simple parser that'll turn a policy-related statement into a context object required for
/// completions.
/// The parser will only work if the (trimmed) sql starts with `create policy`, `drop policy`, or `alter policy`.
/// It can only parse policy statements.
pub(crate) struct PolicyParser {
    tokens: Peekable<std::vec::IntoIter<WordWithIndex>>,
    previous_token: Option<WordWithIndex>,
    current_token: Option<WordWithIndex>,
    context: PolicyContext,
    cursor_position: usize,
}

impl PolicyParser {
    pub(crate) fn looks_like_policy_stmt(sql: &str) -> bool {
        let lowercased = sql.to_ascii_lowercase();
        let trimmed = lowercased.trim();
        trimmed.starts_with("create policy")
            || trimmed.starts_with("drop policy")
            || trimmed.starts_with("alter policy")
    }

    pub(crate) fn get_context(sql: &str, cursor_position: usize) -> PolicyContext {
        assert!(
            Self::looks_like_policy_stmt(sql),
            "PolicyParser should only be used for policy statements. Developer error!"
        );

        match sql_to_words(sql) {
            Ok(tokens) => {
                let parser = PolicyParser {
                    tokens: tokens.into_iter().peekable(),
                    context: PolicyContext::default(),
                    previous_token: None,
                    current_token: None,
                    cursor_position,
                };

                parser.parse()
            }
            Err(_) => PolicyContext::default(),
        }
    }

    fn parse(mut self) -> PolicyContext {
        while let Some(token) = self.advance() {
            if token.is_under_cursor(self.cursor_position) {
                self.handle_token_under_cursor(token);
            } else {
                self.handle_token(token);
            }
        }

        self.context
    }

    fn handle_token_under_cursor(&mut self, token: WordWithIndex) {
        if self.previous_token.is_none() {
            return;
        }

        let previous = self.previous_token.take().unwrap();

        match previous.word.to_ascii_lowercase().as_str() {
            "policy" => {
                self.context.node_range = token.get_range();
                self.context.node_kind = "policy_name".into();
                self.context.node_text = token.word;
            }
            "on" => {
                if token.word.contains('.') {
                    let (schema_name, table_name) = self.schema_and_table_name(&token);

                    let schema_name_len = schema_name.len();
                    self.context.schema_name = Some(schema_name);

                    let offset: u32 = schema_name_len.try_into().expect("Text too long");
                    let range_without_schema = token
                        .get_range()
                        .checked_expand_start(
                            TextSize::new(offset + 1), // kill the dot as well
                        )
                        .expect("Text too long");

                    self.context.node_range = range_without_schema;
                    self.context.node_kind = "policy_table".into();

                    // In practice, we should always have a table name.
                    // The completion sanitization will add a word after a `.` if nothing follows it;
                    // the token_text will then look like `schema.REPLACED_TOKEN`.
                    self.context.node_text = table_name.unwrap_or_default();
                } else {
                    self.context.node_range = token.get_range();
                    self.context.node_text = token.word;
                    self.context.node_kind = "policy_table".into();
                }
            }
            "to" => {
                self.context.node_range = token.get_range();
                self.context.node_kind = "policy_role".into();
                self.context.node_text = token.word;
            }
            _ => {
                self.context.node_range = token.get_range();
                self.context.node_text = token.word;
            }
        }
    }

    fn handle_token(&mut self, token: WordWithIndex) {
        match token.word.to_ascii_lowercase().as_str() {
            "create" if self.next_matches("policy") => {
                self.context.statement_kind = PolicyStmtKind::Create;
            }
            "alter" if self.next_matches("policy") => {
                self.context.statement_kind = PolicyStmtKind::Alter;
            }
            "drop" if self.next_matches("policy") => {
                self.context.statement_kind = PolicyStmtKind::Drop;
            }
            "on" => self.table_with_schema(),

            // skip the "to" so we don't parse it as the TO rolename when it's under the cursor
            "rename" if self.next_matches("to") => {
                self.advance();
            }

            _ => {
                if self.prev_matches("policy") {
                    self.context.policy_name = Some(token.word);
                }
            }
        }
    }

    fn next_matches(&mut self, it: &str) -> bool {
        self.tokens.peek().is_some_and(|c| c.word.as_str() == it)
    }

    fn prev_matches(&self, it: &str) -> bool {
        self.previous_token.as_ref().is_some_and(|t| t.word == it)
    }

    fn advance(&mut self) -> Option<WordWithIndex> {
        // we can't peek back n an iterator, so we'll have to keep track manually.
        self.previous_token = self.current_token.take();
        self.current_token = self.tokens.next();
        self.current_token.clone()
    }

    fn table_with_schema(&mut self) {
        if let Some(token) = self.advance() {
            if token.is_under_cursor(self.cursor_position) {
                self.handle_token_under_cursor(token);
            } else if token.word.contains('.') {
                let (schema, maybe_table) = self.schema_and_table_name(&token);
                self.context.schema_name = Some(schema);
                self.context.table_name = maybe_table;
            } else {
                self.context.table_name = Some(token.word);
            }
        };
    }

    fn schema_and_table_name(&self, token: &WordWithIndex) -> (String, Option<String>) {
        let mut parts = token.word.split('.');

        (
            parts.next().unwrap().into(),
            parts.next().map(|tb| tb.into()),
        )
    }
}

#[cfg(test)]
mod tests {
    use pgt_text_size::{TextRange, TextSize};

    use crate::{
        context::policy_parser::{PolicyContext, PolicyStmtKind, WordWithIndex},
        test_helper::CURSOR_POS,
    };

    use super::{PolicyParser, sql_to_words};

    fn with_pos(query: String) -> (usize, String) {
        let mut pos: Option<usize> = None;

        for (p, c) in query.char_indices() {
            if c == CURSOR_POS {
                pos = Some(p);
                break;
            }
        }

        (
            pos.expect("Please add cursor position!"),
            query.replace(CURSOR_POS, "REPLACED_TOKEN").to_string(),
        )
    }

    #[test]
    fn infers_progressively() {
        let (pos, query) = with_pos(format!(
            r#"
          create policy {}
        "#,
            CURSOR_POS
        ));

        let context = PolicyParser::get_context(query.as_str(), pos);

        assert_eq!(
            context,
            PolicyContext {
                policy_name: None,
                table_name: None,
                schema_name: None,
                statement_kind: PolicyStmtKind::Create,
                node_text: "REPLACED_TOKEN".into(),
                node_range: TextRange::new(TextSize::new(25), TextSize::new(39)),
                node_kind: "policy_name".into()
            }
        );

        let (pos, query) = with_pos(format!(
            r#"
          create policy "my cool policy" {}
        "#,
            CURSOR_POS
        ));

        let context = PolicyParser::get_context(query.as_str(), pos);

        assert_eq!(
            context,
            PolicyContext {
                policy_name: Some("\"my cool policy\"".into()),
                table_name: None,
                schema_name: None,
                statement_kind: PolicyStmtKind::Create,
                node_text: "REPLACED_TOKEN".into(),
                node_kind: "".into(),
                node_range: TextRange::new(TextSize::new(42), TextSize::new(56)),
            }
        );

        let (pos, query) = with_pos(format!(
            r#"
          create policy "my cool policy" on {}
        "#,
            CURSOR_POS
        ));

        let context = PolicyParser::get_context(query.as_str(), pos);

        assert_eq!(
            context,
            PolicyContext {
                policy_name: Some("\"my cool policy\"".into()),
                table_name: None,
                schema_name: None,
                statement_kind: PolicyStmtKind::Create,
                node_text: "REPLACED_TOKEN".into(),
                node_kind: "policy_table".into(),
                node_range: TextRange::new(TextSize::new(45), TextSize::new(59)),
            }
        );

        let (pos, query) = with_pos(format!(
            r#"
          create policy "my cool policy" on auth.{}
        "#,
            CURSOR_POS
        ));

        let context = PolicyParser::get_context(query.as_str(), pos);

        assert_eq!(
            context,
            PolicyContext {
                policy_name: Some("\"my cool policy\"".into()),
                table_name: None,
                schema_name: Some("auth".into()),
                statement_kind: PolicyStmtKind::Create,
                node_text: "REPLACED_TOKEN".into(),
                node_kind: "policy_table".into(),
                node_range: TextRange::new(TextSize::new(50), TextSize::new(64)),
            }
        );

        let (pos, query) = with_pos(format!(
            r#"
          create policy "my cool policy" on auth.users 
            as {}
        "#,
            CURSOR_POS
        ));

        let context = PolicyParser::get_context(query.as_str(), pos);

        assert_eq!(
            context,
            PolicyContext {
                policy_name: Some("\"my cool policy\"".into()),
                table_name: Some("users".into()),
                schema_name: Some("auth".into()),
                statement_kind: PolicyStmtKind::Create,
                node_text: "REPLACED_TOKEN".into(),
                node_kind: "".into(),
                node_range: TextRange::new(TextSize::new(72), TextSize::new(86)),
            }
        );

        let (pos, query) = with_pos(format!(
            r#"
          create policy "my cool policy" on auth.users 
            as permissive
            {} 
        "#,
            CURSOR_POS
        ));

        let context = PolicyParser::get_context(query.as_str(), pos);

        assert_eq!(
            context,
            PolicyContext {
                policy_name: Some("\"my cool policy\"".into()),
                table_name: Some("users".into()),
                schema_name: Some("auth".into()),
                statement_kind: PolicyStmtKind::Create,
                node_text: "REPLACED_TOKEN".into(),
                node_kind: "".into(),
                node_range: TextRange::new(TextSize::new(95), TextSize::new(109)),
            }
        );

        let (pos, query) = with_pos(format!(
            r#"
          create policy "my cool policy" on auth.users 
            as permissive
            to {} 
        "#,
            CURSOR_POS
        ));

        let context = PolicyParser::get_context(query.as_str(), pos);

        assert_eq!(
            context,
            PolicyContext {
                policy_name: Some("\"my cool policy\"".into()),
                table_name: Some("users".into()),
                schema_name: Some("auth".into()),
                statement_kind: PolicyStmtKind::Create,
                node_text: "REPLACED_TOKEN".into(),
                node_kind: "policy_role".into(),
                node_range: TextRange::new(TextSize::new(98), TextSize::new(112)),
            }
        );
    }

    #[test]
    fn determines_on_table_node() {
        let (pos, query) = with_pos(format!(
            r#"
          create policy "my cool policy"
            on {}
            to all 
            using (true);
        "#,
            CURSOR_POS
        ));

        let context = PolicyParser::get_context(query.as_str(), pos);

        assert_eq!(
            context,
            PolicyContext {
                policy_name: Some(r#""my cool policy""#.into()),
                table_name: None,
                schema_name: None,
                statement_kind: PolicyStmtKind::Create,
                node_text: "REPLACED_TOKEN".into(),
                node_range: TextRange::new(TextSize::new(57), TextSize::new(71)),
                node_kind: "policy_table".into()
            }
        )
    }

    #[test]
    fn determines_on_table_node_after_schema() {
        let (pos, query) = with_pos(format!(
            r#"
          create policy "my cool policy"
            on auth.{}
            to all 
            using (true);
        "#,
            CURSOR_POS
        ));

        let context = PolicyParser::get_context(query.as_str(), pos);

        assert_eq!(
            context,
            PolicyContext {
                policy_name: Some(r#""my cool policy""#.into()),
                table_name: None,
                schema_name: Some("auth".into()),
                statement_kind: PolicyStmtKind::Create,
                node_text: "REPLACED_TOKEN".into(),
                node_range: TextRange::new(TextSize::new(62), TextSize::new(76)),
                node_kind: "policy_table".into()
            }
        )
    }

    #[test]
    fn determines_we_are_on_column_name() {
        let (pos, query) = with_pos(format!(
            r#"
          drop policy {} on auth.users;
        "#,
            CURSOR_POS
        ));

        let context = PolicyParser::get_context(query.as_str(), pos);

        assert_eq!(
            context,
            PolicyContext {
                policy_name: None,
                table_name: Some("users".into()),
                schema_name: Some("auth".into()),
                statement_kind: PolicyStmtKind::Drop,
                node_text: "REPLACED_TOKEN".into(),
                node_range: TextRange::new(TextSize::new(23), TextSize::new(37)),
                node_kind: "policy_name".into()
            }
        );

        // cursor within quotation marks.
        let (pos, query) = with_pos(format!(
            r#"
          drop policy "{}" on auth.users;
        "#,
            CURSOR_POS
        ));

        let context = PolicyParser::get_context(query.as_str(), pos);

        assert_eq!(
            context,
            PolicyContext {
                policy_name: None,
                table_name: Some("users".into()),
                schema_name: Some("auth".into()),
                statement_kind: PolicyStmtKind::Drop,
                node_text: "\"REPLACED_TOKEN\"".into(),
                node_range: TextRange::new(TextSize::new(23), TextSize::new(39)),
                node_kind: "policy_name".into()
            }
        );
    }

    #[test]
    fn single_quotation_mark_does_not_fail() {
        let (pos, query) = with_pos(format!(
            r#"
          drop policy "{} on auth.users;
        "#,
            CURSOR_POS
        ));

        let context = PolicyParser::get_context(query.as_str(), pos);

        assert_eq!(context, PolicyContext::default());
    }

    fn to_word(word: &str, start: usize, end: usize) -> WordWithIndex {
        WordWithIndex {
            word: word.into(),
            start,
            end,
        }
    }

    #[test]
    fn determines_positions_correctly() {
        let query = "\ncreate policy \"my cool pol\"\n\ton auth.users\n\tas permissive\n\tfor select\n\t\tto   public\n\t\tusing (true);".to_string();

        let words = sql_to_words(query.as_str()).unwrap();

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
        assert_eq!(words[12], to_word("(true)", 93, 99));
    }
}
