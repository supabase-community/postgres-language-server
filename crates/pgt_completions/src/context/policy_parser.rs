use std::{iter::Peekable, str::SplitAsciiWhitespace};

#[derive(Default)]
pub enum PolicyStmtKind {
    #[default]
    Create,

    Alter,
    Drop,
}

#[derive(Default)]
pub struct PolicyContext {
    table_name: String,
    schema_name: Option<String>,
    statement_kind: PolicyStmtKind,
}

pub struct PolicyParser<'a> {
    tokens: Peekable<SplitAsciiWhitespace<'a>>,
    sql: &'a str,
    context: PolicyContext,
}

impl<'a> PolicyParser<'a> {
    pub(crate) fn get_context(sql: &'a str, cursor_position: usize) -> PolicyContext {
        let lower_cased = sql.to_ascii_lowercase();

        let parser = PolicyParser {
            tokens: lower_cased.split_ascii_whitespace().peekable(),
            sql,
            context: PolicyContext::default(),
        };

        parser.parse()
    }

    fn parse(mut self) -> PolicyContext {
        while let Some(token) = self.tokens.next() {
            self.handle_token(token);
        }

        self.context
    }

    fn handle_token(&mut self, token: &'a str) {
        match token {
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

            _ => {}
        }
    }

    fn next_matches(&mut self, it: &str) -> bool {
        self.tokens.peek().is_some_and(|c| *c == it)
    }

    fn table_with_schema(&mut self) {
        let token = self.tokens.next();
    }
}
