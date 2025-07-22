use std::fmt::Display;

pub static MIGRATIONS: sqlx::migrate::Migrator = sqlx::migrate!("./testdb_migrations");

static CURSOR_POS: char = '€';

#[derive(Clone)]
pub struct QueryWithCursorPosition {
    sql: String,
    position: usize,
}

impl QueryWithCursorPosition {
    pub fn cursor_marker() -> char {
        CURSOR_POS
    }

    pub fn get_text_and_position(&self) -> (usize, String) {
        (self.position, self.sql.clone())
    }
}

impl From<String> for QueryWithCursorPosition {
    fn from(value: String) -> Self {
        value.as_str().into()
    }
}

impl From<&str> for QueryWithCursorPosition {
    fn from(value: &str) -> Self {
        let position = value
            .find(CURSOR_POS)
            .expect("Use `QueryWithCursorPosition::cursor_marker()` to insert cursor position into your Query.");

        QueryWithCursorPosition {
            sql: value.replace(CURSOR_POS, "").trim().to_string(),
            position,
        }
    }
}

impl Display for QueryWithCursorPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.sql)
    }
}

#[cfg(test)]
mod tests {

    use super::QueryWithCursorPosition;

    #[test]
    fn input_query_should_extract_correct_position() {
        struct TestCase {
            query: String,
            expected_pos: usize,
            expected_sql_len: usize,
        }

        let cases = vec![
            TestCase {
                query: format!("select * from{}", QueryWithCursorPosition::cursor_marker()),
                expected_pos: 13,
                expected_sql_len: 13,
            },
            TestCase {
                query: format!("{}select * from", QueryWithCursorPosition::cursor_marker()),
                expected_pos: 0,
                expected_sql_len: 13,
            },
            TestCase {
                query: format!("select {} from", QueryWithCursorPosition::cursor_marker()),
                expected_pos: 7,
                expected_sql_len: 12,
            },
        ];

        for case in cases {
            let query = QueryWithCursorPosition::from(case.query.as_str());
            assert_eq!(query.position, case.expected_pos);
            assert_eq!(query.sql.len(), case.expected_sql_len);
        }
    }
}
