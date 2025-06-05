use pgt_text_size::{TextRange, TextSize};

use crate::context::base_parser::{
    CompletionStatementParser, TokenNavigator, WordWithIndex, schema_and_table_name,
};

#[derive(Default, Debug, PartialEq, Eq)]
pub(crate) struct RevokeContext {
    pub table_name: Option<String>,
    pub schema_name: Option<String>,
    pub node_text: String,
    pub node_range: TextRange,
    pub node_kind: String,
}

/// Simple parser that'll turn a policy-related statement into a context object required for
/// completions.
/// The parser will only work if the (trimmed) sql starts with `create policy`, `drop policy`, or `alter policy`.
/// It can only parse policy statements.
pub(crate) struct RevokeParser {
    navigator: TokenNavigator,
    context: RevokeContext,
    cursor_position: usize,
    in_roles_list: bool,
    is_revoking_role: bool,
}

impl CompletionStatementParser for RevokeParser {
    type Context = RevokeContext;
    const NAME: &'static str = "GrantParser";

    fn looks_like_matching_stmt(sql: &str) -> bool {
        let lowercased = sql.to_ascii_lowercase();
        let trimmed = lowercased.trim();
        trimmed.starts_with("revoke")
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
            context: RevokeContext::default(),
            cursor_position,
            in_roles_list: false,
            is_revoking_role: false,
        }
    }
}

impl RevokeParser {
    fn handle_token_under_cursor(&mut self, token: WordWithIndex) {
        if self.navigator.previous_token.is_none() {
            return;
        }

        let previous = self.navigator.previous_token.take().unwrap();
        let current = self
            .navigator
            .current_token
            .as_ref()
            .map(|w| w.get_word_without_quotes());

        match previous
            .get_word_without_quotes()
            .to_ascii_lowercase()
            .as_str()
        {
            "on" if !matches!(current.as_ref().map(|c| c.as_str()), Some("table")) => {
                self.handle_table(&token)
            }

            "table" => {
                self.handle_table(&token);
            }

            "from" | "revoke" => {
                self.context.node_range = token.get_range();
                self.context.node_kind = "revoke_role".into();
                self.context.node_text = token.get_word();
            }

            "for" if self.is_revoking_role => {
                self.context.node_range = token.get_range();
                self.context.node_kind = "revoke_role".into();
                self.context.node_text = token.get_word();
            }

            t => {
                if self.in_roles_list && t.ends_with(',') {
                    self.context.node_kind = "grant_role".into();
                }

                self.context.node_range = token.get_range();
                self.context.node_text = token.get_word();
            }
        }
    }

    fn handle_table(&mut self, token: &WordWithIndex) {
        if token.get_word_without_quotes().contains('.') {
            let (schema_name, table_name) = schema_and_table_name(token);

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
            self.context.node_kind = "revoke_table".into();

            // In practice, we should always have a table name.
            // The completion sanitization will add a word after a `.` if nothing follows it;
            // the token_text will then look like `schema.REPLACED_TOKEN`.
            self.context.node_text = table_name.unwrap_or_default();
        } else {
            self.context.node_range = token.get_range();
            self.context.node_text = token.get_word();
            self.context.node_kind = "revoke_table".into();
        }
    }

    fn handle_token(&mut self, token: WordWithIndex) {
        match token.get_word_without_quotes().as_str() {
            "on" if !self.navigator.next_matches(&["table"]) => self.table_with_schema(),

            // This is the only case where there is no "GRANT" before the option:
            // REVOKE [ { ADMIN | INHERIT | SET } OPTION FOR ] role_name
            "option" if !self.navigator.prev_matches(&["grant"]) => {
                self.is_revoking_role = true;
            }

            "table" => self.table_with_schema(),

            "from" => {
                self.in_roles_list = true;
            }

            t => {
                if self.in_roles_list && !t.ends_with(',') {
                    self.in_roles_list = false;
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
