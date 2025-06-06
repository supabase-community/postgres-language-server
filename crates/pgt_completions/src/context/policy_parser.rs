use pgt_text_size::{TextRange, TextSize};

use crate::context::base_parser::{
    CompletionStatementParser, TokenNavigator, WordWithIndex, schema_and_table_name,
};

#[derive(Default, Debug, PartialEq, Eq)]
pub(crate) enum PolicyStmtKind {
    #[default]
    Create,

    Alter,
    Drop,
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
    pub in_check_or_using_clause: bool,
}

/// Simple parser that'll turn a policy-related statement into a context object required for
/// completions.
/// The parser will only work if the (trimmed) sql starts with `create policy`, `drop policy`, or `alter policy`.
/// It can only parse policy statements.
pub(crate) struct PolicyParser {
    navigator: TokenNavigator,
    context: PolicyContext,
    cursor_position: usize,
    in_check_or_using_clause: bool,
}

impl CompletionStatementParser for PolicyParser {
    type Context = PolicyContext;
    const NAME: &'static str = "PolicyParser";

    fn looks_like_matching_stmt(sql: &str) -> bool {
        let lowercased = sql.to_ascii_lowercase();
        let trimmed = lowercased.trim();
        trimmed.starts_with("create policy")
            || trimmed.starts_with("drop policy")
            || trimmed.starts_with("alter policy")
    }

    fn parse(mut self) -> Self::Context {
        while let Some(token) = self.navigator.advance() {
            if token.is_under_cursor(self.cursor_position) {
                self.handle_token_under_cursor(token);
            } else {
                self.handle_token(token);
            }
        }

        self.context
    }

    fn make_parser(tokens: Vec<WordWithIndex>, cursor_position: usize) -> Self {
        Self {
            navigator: tokens.into(),
            context: PolicyContext::default(),
            cursor_position,
            in_check_or_using_clause: false,
        }
    }
}

impl PolicyParser {
    fn handle_token_under_cursor(&mut self, token: WordWithIndex) {
        if self.navigator.previous_token.is_none() {
            return;
        }

        self.context.in_check_or_using_clause = self.in_check_or_using_clause;

        let previous = self.navigator.previous_token.take().unwrap();

        match previous
            .get_word_without_quotes()
            .to_ascii_lowercase()
            .as_str()
        {
            "policy" => {
                self.context.node_range = token.get_range();
                self.context.node_kind = "policy_name".into();
                self.context.node_text = token.get_word();
            }
            "on" => {
                if token.get_word_without_quotes().contains('.') {
                    let (schema_name, table_name) = schema_and_table_name(&token);

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
                    self.context.node_text = token.get_word();
                    self.context.node_kind = "policy_table".into();
                }
            }
            "to" => {
                self.context.node_range = token.get_range();
                self.context.node_kind = "policy_role".into();
                self.context.node_text = token.get_word();
            }
            _ => {
                self.context.node_range = token.get_range();
                self.context.node_text = token.get_word();
            }
        }
    }

    fn handle_token(&mut self, token: WordWithIndex) {
        match token
            .get_word_without_quotes()
            .to_ascii_lowercase()
            .as_str()
        {
            "create" if self.navigator.next_matches(&["policy"]) => {
                self.context.statement_kind = PolicyStmtKind::Create;
            }
            "alter" if self.navigator.next_matches(&["policy"]) => {
                self.context.statement_kind = PolicyStmtKind::Alter;
            }
            "drop" if self.navigator.next_matches(&["policy"]) => {
                self.context.statement_kind = PolicyStmtKind::Drop;
            }
            "on" => self.table_with_schema(),

            "(" if self.navigator.prev_matches(&["using", "check"]) => {
                self.in_check_or_using_clause = true;
            }
            ")" => {
                self.in_check_or_using_clause = false;
            }

            // skip the "to" so we don't parse it as the TO rolename when it's under the cursor
            "rename" if self.navigator.next_matches(&["to"]) => {
                self.navigator.advance();
            }

            _ => {
                if self.navigator.prev_matches(&["policy"]) {
                    self.context.policy_name = Some(token.get_word());
                }
            }
        }
    }

    fn table_with_schema(&mut self) {
        if let Some(token) = self.navigator.advance() {
            if token.is_under_cursor(self.cursor_position) {
                self.handle_token_under_cursor(token);
            } else if token.get_word_without_quotes().contains('.') {
                let (schema, maybe_table) = schema_and_table_name(&token);
                self.context.schema_name = Some(schema);
                self.context.table_name = maybe_table;
            } else {
                self.context.table_name = Some(token.get_word());
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use pgt_text_size::{TextRange, TextSize};

    use crate::{
        context::base_parser::CompletionStatementParser,
        context::policy_parser::{PolicyContext, PolicyStmtKind},
        test_helper::CURSOR_POS,
    };

    use super::PolicyParser;

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
                node_kind: "policy_name".into(),
                in_check_or_using_clause: false
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
                in_check_or_using_clause: false
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
                in_check_or_using_clause: false
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
                in_check_or_using_clause: false
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
                in_check_or_using_clause: false
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
                in_check_or_using_clause: false
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
                in_check_or_using_clause: false
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
                node_kind: "policy_table".into(),
                in_check_or_using_clause: false
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
                node_kind: "policy_table".into(),
                in_check_or_using_clause: false
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
                node_kind: "policy_name".into(),
                in_check_or_using_clause: false
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
                node_kind: "policy_name".into(),
                in_check_or_using_clause: false
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

    #[test]
    fn correctly_determines_we_are_inside_checks() {
        {
            let (pos, query) = with_pos(format!(
                r#"
          create policy "my cool policy"
            on auth.users
            to all 
            using (id = {})
        "#,
                CURSOR_POS
            ));

            let context = PolicyParser::get_context(query.as_str(), pos);

            assert_eq!(
                context,
                PolicyContext {
                    policy_name: Some(r#""my cool policy""#.into()),
                    table_name: Some("users".into()),
                    schema_name: Some("auth".into()),
                    statement_kind: PolicyStmtKind::Create,
                    node_text: "REPLACED_TOKEN".into(),
                    node_range: TextRange::new(TextSize::new(112), TextSize::new(127)),
                    node_kind: "".into(),
                    in_check_or_using_clause: true
                }
            );
        }

        // {
        //     let (pos, query) = with_pos(format!(
        //         r#"
        //   create policy "my cool policy"
        //     on auth.users
        //     to all
        //     using ({}
        // "#,
        //         CURSOR_POS
        //     ));

        //     let context = PolicyParser::get_context(query.as_str(), pos);

        //     assert_eq!(
        //         context,
        //         PolicyContext {
        //             policy_name: Some(r#""my cool policy""#.into()),
        //             table_name: None,
        //             schema_name: None,
        //             statement_kind: PolicyStmtKind::Create,
        //             node_text: "REPLACED_TOKEN".into(),
        //             node_range: TextRange::new(TextSize::new(57), TextSize::new(71)),
        //             node_kind: "policy_table".into(),
        //             in_check_or_using_clause: true
        //         }
        //     )
        // }

        // {
        //     let (pos, query) = with_pos(format!(
        //         r#"
        //   create policy "my cool policy"
        //     on auth.users
        //     to all
        //     with check ({}
        // "#,
        //         CURSOR_POS
        //     ));

        //     let context = PolicyParser::get_context(query.as_str(), pos);

        //     assert_eq!(
        //         context,
        //         PolicyContext {
        //             policy_name: Some(r#""my cool policy""#.into()),
        //             table_name: None,
        //             schema_name: None,
        //             statement_kind: PolicyStmtKind::Create,
        //             node_text: "REPLACED_TOKEN".into(),
        //             node_range: TextRange::new(TextSize::new(57), TextSize::new(71)),
        //             node_kind: "policy_table".into(),
        //             in_check_or_using_clause: true
        //         }
        //     )
        // }
    }
}
