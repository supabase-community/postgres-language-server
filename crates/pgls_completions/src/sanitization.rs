use std::{borrow::Cow, cmp::max};

use pgls_text_size::TextSize;

use crate::CompletionParams;

static SANITIZED_TOKEN: &str = "REPLACED_TOKEN";
static SANITIZED_TOKEN_WITH_QUOTE: &str = r#"REPLACED_TOKEN_WITH_QUOTE""#;

#[derive(Debug)]
pub(crate) struct SanitizedCompletionParams<'a> {
    pub position: TextSize,
    pub text: String,
    pub schema: &'a pgls_schema_cache::SchemaCache,
    pub tree: Cow<'a, tree_sitter::Tree>,
}

pub fn benchmark_sanitization(params: CompletionParams) -> String {
    let params: SanitizedCompletionParams = params.into();
    params.text
}

pub(crate) fn remove_sanitized_token(it: &str) -> String {
    it.replace(SANITIZED_TOKEN_WITH_QUOTE, "")
        .replace(SANITIZED_TOKEN, "")
}

pub(crate) fn is_sanitized_token(node_under_cursor_txt: &str) -> bool {
    node_under_cursor_txt == SANITIZED_TOKEN || is_sanitized_token_with_quote(node_under_cursor_txt)
}

pub(crate) fn is_sanitized_token_with_quote(node_under_cursor_txt: &str) -> bool {
    if node_under_cursor_txt.len() <= 1 {
        return false;
    }

    // Node under cursor text will be "REPLACED_TOKEN_WITH_QUOTE".
    // The SANITIZED_TOKEN_WITH_QUOTE does not have the leading ".
    // We need to omit it from the txt.
    &node_under_cursor_txt[1..] == SANITIZED_TOKEN_WITH_QUOTE
}

impl<'larger, 'smaller> From<CompletionParams<'larger>> for SanitizedCompletionParams<'smaller>
where
    'larger: 'smaller,
{
    fn from(mut params: CompletionParams<'larger>) -> Self {
        params.text = params.text.to_ascii_lowercase();
        if cursor_inbetween_nodes(&params.text, params.position)
            || cursor_prepared_to_write_token_after_last_node(&params.text, params.position)
            || cursor_before_semicolon(params.tree, params.position)
            || cursor_on_a_dot(&params.text, params.position)
            || cursor_between_parentheses(&params.text, params.position)
            || cursor_after_opened_quote(&params.text, params.position)
        {
            SanitizedCompletionParams::with_adjusted_sql(params)
        } else {
            SanitizedCompletionParams::unadjusted(params)
        }
    }
}

impl<'larger, 'smaller> SanitizedCompletionParams<'smaller>
where
    'larger: 'smaller,
{
    fn with_adjusted_sql(params: CompletionParams<'larger>) -> Self {
        let cursor_pos: usize = params.position.into();
        let mut sql = String::new();

        let mut sql_iter = params.text.chars();

        let max = max(cursor_pos + 1, params.text.len());

        let has_uneven_quotes = params.text.chars().filter(|c| *c == '"').count() % 2 != 0;
        let mut opened_quote = false;

        for idx in 0..max {
            match sql_iter.next() {
                Some(c) => {
                    if c == '"' {
                        opened_quote = !opened_quote;
                    }

                    if idx == cursor_pos {
                        if opened_quote && has_uneven_quotes {
                            sql.push_str(SANITIZED_TOKEN_WITH_QUOTE);
                            opened_quote = false;
                        } else {
                            sql.push_str(SANITIZED_TOKEN);
                        }
                    }

                    sql.push(c);
                }
                None => {
                    // the cursor is outside the statement,
                    // we want to push spaces until we arrive at the cursor position.
                    // we'll then add the SANITIZED_TOKEN
                    if idx == cursor_pos {
                        if opened_quote && has_uneven_quotes {
                            sql.push_str(SANITIZED_TOKEN_WITH_QUOTE);
                        } else {
                            sql.push_str(SANITIZED_TOKEN);
                        }
                    } else {
                        sql.push(' ');
                    }
                }
            }
        }

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
            .expect("Error loading sql language");
        let tree = parser.parse(sql.clone(), None).unwrap();

        Self {
            position: params.position,
            text: sql,
            schema: params.schema,
            tree: Cow::Owned(tree),
        }
    }
    fn unadjusted(params: CompletionParams<'larger>) -> Self {
        Self {
            position: params.position,
            text: params.text.clone(),
            schema: params.schema,
            tree: Cow::Borrowed(params.tree),
        }
    }
}

/// Checks if the cursor is positioned inbetween two SQL nodes.
///
/// ```sql
/// select| from users; -- cursor "touches" select node. returns false.
/// select |from users; -- cursor "touches" from node. returns false.
/// select | from users; -- cursor is between select and from nodes. returns true.
/// ```
fn cursor_inbetween_nodes(sql: &str, position: TextSize) -> bool {
    let position: usize = position.into();
    let mut chars = sql.chars();

    let previous_whitespace = chars
        .nth(position - 1)
        .is_some_and(|c| c.is_ascii_whitespace());

    let current_whitespace = chars.next().is_some_and(|c| c.is_ascii_whitespace());

    previous_whitespace && current_whitespace
}

/// Checks if the cursor is positioned after the last node,
/// ready to write the next token:
///
/// ```sql
/// select * from |   -- ready to write!
/// select * from|    -- user still needs to type a space
/// select * from  |  -- too far off.
/// ```
fn cursor_prepared_to_write_token_after_last_node(sql: &str, position: TextSize) -> bool {
    let cursor_pos: usize = position.into();
    cursor_pos == sql.len() + 1
}

fn cursor_on_a_dot(sql: &str, position: TextSize) -> bool {
    let position: usize = position.into();
    sql.chars().nth(position - 1).is_some_and(|c| c == '.')
}

fn cursor_before_semicolon(tree: &tree_sitter::Tree, position: TextSize) -> bool {
    let mut cursor = tree.walk();
    let mut leaf_node = tree.root_node();

    let byte: usize = position.into();

    // if the cursor escapes the root node, it can't be between nodes.
    if byte < leaf_node.start_byte() || byte >= leaf_node.end_byte() {
        return false;
    }

    loop {
        let child_idx = cursor.goto_first_child_for_byte(position.into());
        if child_idx.is_none() {
            break;
        }
        leaf_node = cursor.node();
    }

    // The semicolon node is on the same level as the statement:
    //
    // program [0..26]
    //   statement [0..19]
    //   ; [25..26]
    //
    // However, if we search for position 21, we'll still land on the semi node.
    // We must manually verify that the cursor is between the statement and the semi nodes.

    // if the last node is not a semi, the statement is not completed.
    if leaf_node.kind() != ";" {
        return false;
    }

    leaf_node
        .prev_named_sibling()
        .map(|n| n.end_byte() < byte)
        .unwrap_or(false)
}

fn cursor_between_parentheses(sql: &str, position: TextSize) -> bool {
    let position: usize = position.into();

    let mut level = 0;
    let mut tracking_open_idx = None;

    let mut matching_open_idx = None;
    let mut matching_close_idx = None;

    for (idx, char) in sql.chars().enumerate() {
        if char == '(' {
            tracking_open_idx = Some(idx);
            level += 1;
        }

        if char == ')' {
            level -= 1;

            if tracking_open_idx.is_some_and(|it| it < position) && idx >= position {
                matching_open_idx = tracking_open_idx;
                matching_close_idx = Some(idx)
            }
        }
    }

    // invalid statement
    if level != 0 {
        return false;
    }

    // early check: '(|)'
    // however, we want to check this after the level nesting.
    let mut chars = sql.chars();
    if chars.nth(position - 1).is_some_and(|c| c == '(') && chars.next().is_some_and(|c| c == ')') {
        return true;
    }

    // not *within* parentheses
    if matching_open_idx.is_none() || matching_close_idx.is_none() {
        return false;
    }

    // use string indexing, because we can't `.rev()` after `.take()`
    let before = sql[..position]
        .to_string()
        .chars()
        .rev()
        .find(|c| !c.is_whitespace())
        .unwrap_or_default();

    let after = sql
        .chars()
        .skip(position)
        .find(|c| !c.is_whitespace())
        .unwrap_or_default();

    // (.. and |)
    let after_and_keyword = &sql[position.saturating_sub(4)..position] == "and " && after == ')';
    let after_eq_sign = before == '=' && after == ')';

    let head_of_list = before == '(' && after == ',';
    let end_of_list = before == ',' && after == ')';
    let between_list_items = before == ',' && after == ',';

    head_of_list || end_of_list || between_list_items || after_and_keyword || after_eq_sign
}

fn cursor_after_opened_quote(sql: &str, position: TextSize) -> bool {
    let position: usize = position.into();
    let mut opened_quote = false;
    let mut preceding_quote = false;

    for (idx, c) in sql.char_indices() {
        if idx == position && opened_quote && preceding_quote {
            return true;
        }

        if c == '"' {
            preceding_quote = true;
            opened_quote = !opened_quote;
        } else {
            preceding_quote = false;
        }
    }

    opened_quote && preceding_quote
}

#[cfg(test)]
mod tests {
    use pgls_schema_cache::SchemaCache;
    use pgls_text_size::TextSize;

    use crate::{
        CompletionParams, SanitizedCompletionParams,
        sanitization::{
            cursor_after_opened_quote, cursor_before_semicolon, cursor_between_parentheses,
            cursor_inbetween_nodes, cursor_on_a_dot,
            cursor_prepared_to_write_token_after_last_node,
        },
    };

    fn get_test_params(input: &str, position: TextSize) -> CompletionParams {
        let mut ts = tree_sitter::Parser::new();
        ts.set_language(&pgls_treesitter_grammar::LANGUAGE.into())
            .unwrap();

        let tree = Box::new(ts.parse(input, None).unwrap());
        let cache = Box::new(SchemaCache::default());

        let leaked_tree = Box::leak(tree);
        let leaked_cache = Box::leak(cache);

        CompletionParams {
            position,
            schema: leaked_cache,
            text: input.into(),
            tree: leaked_tree,
        }
    }

    #[test]
    fn should_lowercase_everything_except_replaced_token() {
        let input = "SELECT  FROM users WHERE ts = NOW();";
        let position = TextSize::new(7);

        let params = get_test_params(input, position);
        let sanitized = SanitizedCompletionParams::from(params);

        assert_eq!(
            sanitized.text,
            "select REPLACED_TOKEN from users where ts = now();"
        );
    }

    #[test]
    fn should_sanitize_with_opened_quotes() {
        {
            // select "email", "| from "auth"."users";
            let input = r#"select "email", " from "auth"."users";"#;
            let position = TextSize::new(17);

            let params = get_test_params(input, position);

            let sanitized = SanitizedCompletionParams::from(params);

            assert_eq!(
                sanitized.text,
                r#"select "email", "REPLACED_TOKEN_WITH_QUOTE" from "auth"."users";"#
            );
        }

        {
            // select * from "auth"."|; <-- with semi
            let input = r#"select * from "auth".";"#;
            let position = TextSize::new(22);

            let params = get_test_params(input, position);

            let sanitized = SanitizedCompletionParams::from(params);

            assert_eq!(
                sanitized.text,
                r#"select * from "auth"."REPLACED_TOKEN_WITH_QUOTE";"#
            );
        }

        {
            // select * from "auth"."| <-- without semi
            let input = r#"select * from "auth".""#;
            let position = TextSize::new(22);

            let params = get_test_params(input, position);

            let sanitized = SanitizedCompletionParams::from(params);

            assert_eq!(
                sanitized.text,
                r#"select * from "auth"."REPLACED_TOKEN_WITH_QUOTE""#
            );
        }
    }

    #[test]
    fn should_not_complete_quote_if_we_are_inside_pair() {
        // select "email", "|  " from "auth"."users";
        // we have opened a quote, but it is already closed a couple of characters later
        let input = r#"select "email", "  " from "auth"."users";"#;
        let position = TextSize::new(17);

        let params = get_test_params(input, position);

        let sanitized = SanitizedCompletionParams::from(params);

        assert_eq!(
            sanitized.text,
            r#"select "email", "REPLACED_TOKEN  " from "auth"."users";"#
        );
    }

    #[test]
    fn should_not_use_quote_token_if_we_are_not_within_opened_quote() {
        // select "users".| from "users" join "public"."
        // we have an opened quote at the end, but the cursor is not within an opened quote
        let input = r#"select "users". from "users" join "public"."   "#;
        let position = TextSize::new(15);

        let params = get_test_params(input, position);

        let sanitized = SanitizedCompletionParams::from(params);

        assert_eq!(
            sanitized.text,
            r#"select "users".REPLACED_TOKEN from "users" join "public"."   "#
        );
    }

    #[test]
    fn test_cursor_inbetween_nodes() {
        // note: two spaces between select and from.
        let input = "select  from users;";

        // select | from users; <-- just right, one space after select token, one space before from
        assert!(cursor_inbetween_nodes(input, TextSize::new(7)));

        // select|  from users; <-- still on select token
        assert!(!cursor_inbetween_nodes(input, TextSize::new(6)));

        // select  |from users; <-- already on from token
        assert!(!cursor_inbetween_nodes(input, TextSize::new(8)));

        // select from users;|
        assert!(!cursor_inbetween_nodes(input, TextSize::new(19)));
    }

    #[test]
    fn test_cursor_after_nodes() {
        let input = "select * from";

        // select * from| <-- still on previous token
        assert!(!cursor_prepared_to_write_token_after_last_node(
            input,
            TextSize::new(13)
        ));

        // select * from  | <-- too far off, two spaces afterward
        assert!(!cursor_prepared_to_write_token_after_last_node(
            input,
            TextSize::new(15)
        ));

        // select * |from  <-- it's within
        assert!(!cursor_prepared_to_write_token_after_last_node(
            input,
            TextSize::new(9)
        ));

        // select * from | <-- just right
        assert!(cursor_prepared_to_write_token_after_last_node(
            input,
            TextSize::new(14)
        ));
    }

    #[test]
    fn on_a_dot() {
        let input = "select * from private.";

        // select * from private.| <-- on a dot
        assert!(cursor_on_a_dot(input, TextSize::new(22)));

        // select * from private|. <-- before the dot
        assert!(!cursor_on_a_dot(input, TextSize::new(21)));

        // select * from private. | <-- too far off the dot
        assert!(!cursor_on_a_dot(input, TextSize::new(23)));
    }

    #[test]
    fn test_cursor_before_semicolon() {
        // Idx "13" is the exlusive end of `select * from` (first space after from)
        // Idx "18" is right where the semi is
        let input = "select * from     ;";

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
            .expect("Error loading sql language");

        let tree = parser.parse(input, None).unwrap();

        // select * from     ;| <-- it's after the statement
        assert!(!cursor_before_semicolon(&tree, TextSize::new(19)));

        // select * from|    ; <-- still touches the from
        assert!(!cursor_before_semicolon(&tree, TextSize::new(13)));

        // anything is fine here
        // select * from |    ;
        // select * from  |   ;
        // select * from   |  ;
        // select * from    | ;
        // select * from     |;
        assert!(cursor_before_semicolon(&tree, TextSize::new(14)));
        assert!(cursor_before_semicolon(&tree, TextSize::new(15)));
        assert!(cursor_before_semicolon(&tree, TextSize::new(16)));
        assert!(cursor_before_semicolon(&tree, TextSize::new(17)));
        assert!(cursor_before_semicolon(&tree, TextSize::new(18)));
    }

    #[test]
    fn between_parentheses() {
        let input = "insert into instruments ()";

        // insert into (|) <- right in the parentheses
        assert!(cursor_between_parentheses(input, TextSize::new(25)));

        // insert into ()| <- too late
        assert!(!cursor_between_parentheses(input, TextSize::new(26)));

        // insert into |() <- too early
        assert!(!cursor_between_parentheses(input, TextSize::new(24)));

        let input = "insert into instruments (name, id, )";

        // insert into instruments (name, id, |) <-- we should sanitize the next column
        assert!(cursor_between_parentheses(input, TextSize::new(35)));

        // insert into instruments (name, id|, ) <-- we are still on the previous token.
        assert!(!cursor_between_parentheses(input, TextSize::new(33)));

        let input = "insert into instruments (name, , id)";

        // insert into instruments (name, |, id) <-- we can sanitize!
        assert!(cursor_between_parentheses(input, TextSize::new(31)));

        // insert into instruments (name, ,| id) <-- we are already on the next token
        assert!(!cursor_between_parentheses(input, TextSize::new(32)));

        let input = "insert into instruments (, name, id)";

        // insert into instruments (|, name, id) <-- we can sanitize!
        assert!(cursor_between_parentheses(input, TextSize::new(25)));

        // insert into instruments (,| name, id) <-- already on next token
        assert!(!cursor_between_parentheses(input, TextSize::new(26)));

        // bails on invalidly nested statements
        assert!(!cursor_between_parentheses(
            "insert into (instruments ()",
            TextSize::new(26)
        ));

        // can find its position in nested statements
        // "insert into instruments (name) values (a_function(name, |))",
        assert!(cursor_between_parentheses(
            "insert into instruments (name) values (a_function(name, ))",
            TextSize::new(56)
        ));

        // will sanitize after =
        assert!(cursor_between_parentheses(
            // create policy my_pol on users using (id = |),
            "create policy my_pol on users using (id = )",
            TextSize::new(42)
        ));

        // will sanitize after and
        assert!(cursor_between_parentheses(
            // create policy my_pol on users using (id = 1 and |),
            "create policy my_pol on users using (id = 1 and )",
            TextSize::new(48)
        ));

        // does not break if sql is really short
        assert!(!cursor_between_parentheses("(a)", TextSize::new(2)));
    }

    #[test]
    fn after_single_quote() {
        // select "|    <-- right after single quote
        assert!(cursor_after_opened_quote(r#"select ""#, TextSize::new(8)));
        // select "| from something;    <-- right after opening quote
        assert!(cursor_after_opened_quote(
            r#"select " from something;"#,
            TextSize::new(8)
        ));

        // select "user_id", "|    <-- right after opening quote
        assert!(cursor_after_opened_quote(
            r#"select "user_id", ""#,
            TextSize::new(19)
        ));
        // select "user_id, "| from something;    <-- right after opening quote
        assert!(cursor_after_opened_quote(
            r#"select "user_id", " from something;"#,
            TextSize::new(19)
        ));

        // select "user_id"| from something;    <-- after closing quote
        assert!(!cursor_after_opened_quote(
            r#"select "user_id" from something;"#,
            TextSize::new(16)
        ));

        // select ""| from something;    <-- after closing quote
        assert!(!cursor_after_opened_quote(
            r#"select "" from something;"#,
            TextSize::new(9)
        ));

        // select "user_id, " |from something;    <-- one off after opening quote
        assert!(!cursor_after_opened_quote(
            r#"select "user_id", " from something;"#,
            TextSize::new(20)
        ));
    }
}
