use std::borrow::Cow;

use pgt_text_size::TextSize;

use crate::CompletionParams;

pub(crate) struct SanitizedCompletionParams<'a> {
    pub position: TextSize,
    pub text: String,
    pub schema: &'a pgt_schema_cache::SchemaCache,
    pub tree: Cow<'a, tree_sitter::Tree>,
}

pub fn benchmark_sanitization(params: CompletionParams) -> String {
    let params: SanitizedCompletionParams = params.try_into().unwrap();
    params.text
}

impl<'larger, 'smaller> From<CompletionParams<'larger>> for SanitizedCompletionParams<'smaller>
where
    'larger: 'smaller,
{
    fn from(params: CompletionParams<'larger>) -> Self {
        if cursor_inbetween_nodes(params.tree, params.position)
            || cursor_prepared_to_write_token_after_last_node(params.tree, params.position)
            || cursor_before_semicolon(params.tree, params.position)
        {
            SanitizedCompletionParams::with_adjusted_sql(params)
        } else {
            SanitizedCompletionParams::unadjusted(params)
        }
    }
}

static SANITIZED_TOKEN: &str = "REPLACED_TOKEN";

impl<'larger, 'smaller> SanitizedCompletionParams<'smaller>
where
    'larger: 'smaller,
{
    fn with_adjusted_sql(params: CompletionParams<'larger>) -> Self {
        let cursor_pos: usize = params.position.into();
        let mut sql = String::new();

        for (idx, c) in params.text.chars().enumerate() {
            if idx == cursor_pos {
                sql.push_str(SANITIZED_TOKEN);
                sql.push(' ');
            }
            sql.push(c);
        }

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(tree_sitter_sql::language())
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

    pub fn is_sanitized_token(txt: &str) -> bool {
        txt == SANITIZED_TOKEN
    }
}

/// Checks if the cursor is positioned inbetween two SQL nodes.
///
/// ```sql
/// select| from users; -- cursor "touches" select node. returns false.
/// select |from users; -- cursor "touches" from node. returns false.
/// select | from users; -- cursor is between select and from nodes. returns true.
/// ```
fn cursor_inbetween_nodes(tree: &tree_sitter::Tree, position: TextSize) -> bool {
    let mut cursor = tree.walk();
    let mut leaf_node = tree.root_node();

    let byte = position.into();

    // if the cursor escapes the root node, it can't be between nodes.
    if byte < leaf_node.start_byte() || byte >= leaf_node.end_byte() {
        return false;
    }

    /*
     * Get closer and closer to the leaf node, until
     *  a) there is no more child *for the node* or
     *  b) there is no more child *under the cursor*.
     */
    loop {
        let child_idx = cursor.goto_first_child_for_byte(position.into());
        if child_idx.is_none() {
            break;
        }
        leaf_node = cursor.node();
    }

    let cursor_on_leafnode = byte >= leaf_node.start_byte() && leaf_node.end_byte() >= byte;

    /*
     * The cursor is inbetween nodes if it is not within the range
     * of a leaf node.
     */
    !cursor_on_leafnode
}

/// Checks if the cursor is positioned after the last node,
/// ready to write the next token:
///
/// ```sql
/// select * from |   -- ready to write!
/// select * from|    -- user still needs to type a space
/// select * from  |  -- too far off.
/// ```
fn cursor_prepared_to_write_token_after_last_node(
    tree: &tree_sitter::Tree,
    position: TextSize,
) -> bool {
    let cursor_pos: usize = position.into();
    cursor_pos == tree.root_node().end_byte() + 1
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

    // not okay to be on the semi.
    if byte == leaf_node.start_byte() {
        return false;
    }

    leaf_node
        .prev_named_sibling()
        .map(|n| n.end_byte() < byte)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use pgt_text_size::TextSize;

    use crate::sanitization::{
        cursor_before_semicolon, cursor_inbetween_nodes,
        cursor_prepared_to_write_token_after_last_node,
    };

    #[test]
    fn test_cursor_inbetween_nodes() {
        // note: two spaces between select and from.
        let input = "select  from users;";

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(tree_sitter_sql::language())
            .expect("Error loading sql language");

        let mut tree = parser.parse(input.to_string(), None).unwrap();

        // select | from users; <-- just right, one space after select token, one space before from
        assert!(cursor_inbetween_nodes(&mut tree, TextSize::new(7)));

        // select|  from users; <-- still on select token
        assert!(!cursor_inbetween_nodes(&mut tree, TextSize::new(6)));

        // select  |from users; <-- already on from token
        assert!(!cursor_inbetween_nodes(&mut tree, TextSize::new(8)));

        // select from users;|
        assert!(!cursor_inbetween_nodes(&mut tree, TextSize::new(19)));
    }

    #[test]
    fn test_cursor_after_nodes() {
        let input = "select * from ";

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(tree_sitter_sql::language())
            .expect("Error loading sql language");

        let mut tree = parser.parse(input.to_string(), None).unwrap();

        // select * from| <-- still on previous token
        assert!(!cursor_prepared_to_write_token_after_last_node(
            &mut tree,
            TextSize::new(13)
        ));

        // select * from  | <-- too far off, two spaces afterward
        assert!(!cursor_prepared_to_write_token_after_last_node(
            &mut tree,
            TextSize::new(15)
        ));

        // select * |from  <-- it's within
        assert!(!cursor_prepared_to_write_token_after_last_node(
            &mut tree,
            TextSize::new(9)
        ));

        // select * from | <-- just right
        assert!(cursor_prepared_to_write_token_after_last_node(
            &mut tree,
            TextSize::new(14)
        ));
    }

    #[test]
    fn test_cursor_before_semicolon() {
        // Idx "13" is the exlusive end of `select * from` (first space after from)
        // Idx "18" is right where the semi is
        let input = "select * from     ;";

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(tree_sitter_sql::language())
            .expect("Error loading sql language");

        let mut tree = parser.parse(input.to_string(), None).unwrap();

        // select * from     ;| <-- it's after the statement
        assert!(!cursor_before_semicolon(&mut tree, TextSize::new(19)));

        // select * from|    ; <-- still touches the from
        assert!(!cursor_before_semicolon(&mut tree, TextSize::new(13)));

        // not okay to be ON the semi.
        // select * from     |;
        assert!(!cursor_before_semicolon(&mut tree, TextSize::new(18)));

        // anything is fine here
        // select * from |   ;
        // select * from  |  ;
        // select * from   | ;
        // select * from    |;
        assert!(cursor_before_semicolon(&mut tree, TextSize::new(14)));
        assert!(cursor_before_semicolon(&mut tree, TextSize::new(15)));
        assert!(cursor_before_semicolon(&mut tree, TextSize::new(16)));
        assert!(cursor_before_semicolon(&mut tree, TextSize::new(17)));
    }
}
