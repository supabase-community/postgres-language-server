use pgt_text_size::{TextRange, TextSize};

use crate::context::base_parser::{
    CompletionStatementParser, TokenNavigator, WordWithIndex, schema_and_table_name,
};

#[derive(Default, Debug, PartialEq, Eq)]
pub(crate) struct GrantContext {
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
pub(crate) struct GrantParser {
    navigator: TokenNavigator,
    context: GrantContext,
    cursor_position: usize,
    in_roles_list: bool,
}

impl CompletionStatementParser for GrantParser {
    type Context = GrantContext;
    const NAME: &'static str = "GrantParser";

    fn looks_like_matching_stmt(sql: &str) -> bool {
        let lowercased = sql.to_ascii_lowercase();
        let trimmed = lowercased.trim();
        trimmed.starts_with("grant")
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
            context: GrantContext::default(),
            cursor_position,
            in_roles_list: false,
        }
    }
}

impl GrantParser {
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
            "grant" => {
                self.context.node_range = token.get_range();
                self.context.node_kind = "grant_role".into();
                self.context.node_text = token.get_word();
            }
            "on" if !matches!(current.as_deref(), Some("table")) => {
                self.handle_table(&token)
            }

            "table" => {
                self.handle_table(&token);
            }
            "to" => {
                self.context.node_range = token.get_range();
                self.context.node_kind = "grant_role".into();
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
            self.context.node_kind = "grant_table".into();

            // In practice, we should always have a table name.
            // The completion sanitization will add a word after a `.` if nothing follows it;
            // the token_text will then look like `schema.REPLACED_TOKEN`.
            self.context.node_text = table_name.unwrap_or_default();
        } else {
            self.context.node_range = token.get_range();
            self.context.node_text = token.get_word();
            self.context.node_kind = "grant_table".into();
        }
    }

    fn handle_token(&mut self, token: WordWithIndex) {
        match token.get_word_without_quotes().as_str() {
            "on" if !self.navigator.next_matches(&[
                "table",
                "schema",
                "foreign",
                "domain",
                "sequence",
                "database",
                "function",
                "procedure",
                "routine",
                "language",
                "large",
                "parameter",
                "schema",
                "tablespace",
                "type",
            ]) =>
            {
                self.table_with_schema()
            }
            "table" => self.table_with_schema(),

            "to" => {
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

#[cfg(test)]
mod tests {
    use pgt_text_size::{TextRange, TextSize};

    use crate::{
        context::base_parser::CompletionStatementParser,
        context::grant_parser::{GrantContext, GrantParser},
        test_helper::CURSOR_POS,
    };

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
    fn infers_grant_keyword() {
        let (pos, query) = with_pos(format!(
            r#"
            grant {}
        "#,
            CURSOR_POS
        ));

        let context = GrantParser::get_context(query.as_str(), pos);

        assert_eq!(
            context,
            GrantContext {
                table_name: None,
                schema_name: None,
                node_text: "REPLACED_TOKEN".into(),
                node_range: TextRange::new(TextSize::new(19), TextSize::new(33)),
                node_kind: "grant_role".into(),
            }
        );
    }

    #[test]
    fn infers_table_name() {
        let (pos, query) = with_pos(format!(
            r#"
            grant select on {} 
        "#,
            CURSOR_POS
        ));

        let context = GrantParser::get_context(query.as_str(), pos);

        assert_eq!(
            context,
            GrantContext {
                table_name: None,
                schema_name: None,
                node_text: "REPLACED_TOKEN".into(),
                node_range: TextRange::new(TextSize::new(29), TextSize::new(43)),
                node_kind: "grant_table".into(),
            }
        );
    }

    #[test]
    fn infers_table_name_with_keyword() {
        let (pos, query) = with_pos(format!(
            r#"
            grant select on table {} 
        "#,
            CURSOR_POS
        ));

        let context = GrantParser::get_context(query.as_str(), pos);

        assert_eq!(
            context,
            GrantContext {
                table_name: None,
                schema_name: None,
                node_text: "REPLACED_TOKEN".into(),
                node_range: TextRange::new(TextSize::new(35), TextSize::new(49)),
                node_kind: "grant_table".into(),
            }
        );
    }

    #[test]
    fn infers_schema_and_table_name() {
        let (pos, query) = with_pos(format!(
            r#"
            grant select on public.{} 
        "#,
            CURSOR_POS
        ));

        let context = GrantParser::get_context(query.as_str(), pos);

        assert_eq!(
            context,
            GrantContext {
                table_name: None,
                schema_name: Some("public".into()),
                node_text: "REPLACED_TOKEN".into(),
                node_range: TextRange::new(TextSize::new(36), TextSize::new(50)),
                node_kind: "grant_table".into(),
            }
        );
    }

    #[test]
    fn infers_schema_and_table_name_with_keyword() {
        let (pos, query) = with_pos(format!(
            r#"
            grant select on table public.{} 
        "#,
            CURSOR_POS
        ));

        let context = GrantParser::get_context(query.as_str(), pos);

        assert_eq!(
            context,
            GrantContext {
                table_name: None,
                schema_name: Some("public".into()),
                node_text: "REPLACED_TOKEN".into(),
                node_range: TextRange::new(TextSize::new(42), TextSize::new(56)),
                node_kind: "grant_table".into(),
            }
        );
    }

    #[test]
    fn infers_role_name() {
        let (pos, query) = with_pos(format!(
            r#"
            grant select on public.users to {} 
        "#,
            CURSOR_POS
        ));

        let context = GrantParser::get_context(query.as_str(), pos);

        assert_eq!(
            context,
            GrantContext {
                table_name: Some("users".into()),
                schema_name: Some("public".into()),
                node_text: "REPLACED_TOKEN".into(),
                node_range: TextRange::new(TextSize::new(45), TextSize::new(59)),
                node_kind: "grant_role".into(),
            }
        );
    }

    #[test]
    fn determines_table_name_after_schema() {
        let (pos, query) = with_pos(format!(
            r#"
            grant select on public.{} to test_role
        "#,
            CURSOR_POS
        ));

        let context = GrantParser::get_context(query.as_str(), pos);

        assert_eq!(
            context,
            GrantContext {
                table_name: None,
                schema_name: Some("public".into()),
                node_text: "REPLACED_TOKEN".into(),
                node_range: TextRange::new(TextSize::new(36), TextSize::new(50)),
                node_kind: "grant_table".into(),
            }
        );
    }

    #[test]
    fn infers_quoted_schema_and_table() {
        let (pos, query) = with_pos(format!(
            r#"
            grant select on "MySchema"."MyTable" to {}
        "#,
            CURSOR_POS
        ));

        let context = GrantParser::get_context(query.as_str(), pos);

        assert_eq!(
            context,
            GrantContext {
                table_name: Some("MyTable".into()),
                schema_name: Some("MySchema".into()),
                node_text: "REPLACED_TOKEN".into(),
                node_range: TextRange::new(TextSize::new(53), TextSize::new(67)),
                node_kind: "grant_role".into(),
            }
        );
    }

    #[test]
    fn infers_multiple_roles() {
        let (pos, query) = with_pos(format!(
            r#"
            grant select on public.users to alice, {}
        "#,
            CURSOR_POS
        ));

        let context = GrantParser::get_context(query.as_str(), pos);

        assert_eq!(
            context,
            GrantContext {
                table_name: Some("users".into()),
                schema_name: Some("public".into()),
                node_text: "REPLACED_TOKEN".into(),
                node_range: TextRange::new(TextSize::new(52), TextSize::new(66)),
                node_kind: "grant_role".into(),
            }
        );
    }
}
