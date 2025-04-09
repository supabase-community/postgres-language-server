use pgt_text_size::TextSize;

use crate::{
    builder::CompletionBuilder,
    context::CompletionContext,
    item::CompletionItem,
    providers::{complete_columns, complete_functions, complete_tables},
};

pub const LIMIT: usize = 50;

#[derive(Debug)]
pub struct CompletionParams<'a> {
    pub position: TextSize,
    pub schema: &'a pgt_schema_cache::SchemaCache,
    pub text: String,
    pub tree: Option<&'a tree_sitter::Tree>,
}

#[tracing::instrument(level = "debug", skip_all, fields(
    text = params.text,
    position = params.position.to_string()
))]
pub fn complete(mut params: CompletionParams) -> Vec<CompletionItem> {
    let should_adjust_params = params.tree.is_some()
        && (cursor_inbetween_nodes(params.tree.unwrap(), params.position)
            || cursor_prepared_to_write_token_after_last_node(
                params.tree.unwrap(),
                params.position,
            ));

    let usable_sql = if should_adjust_params {
        let pos: usize = params.position.into();

        let mut mutated_sql = String::new();

        for (idx, c) in params.text.chars().enumerate() {
            if idx == pos {
                mutated_sql.push_str("REPLACED_TOKEN ");
            }
            mutated_sql.push(c);
        }

        mutated_sql
    } else {
        params.text
    };

    let usable_tree = if should_adjust_params {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(tree_sitter_sql::language())
            .expect("Error loading sql language");
        parser.parse(usable_sql.clone(), None)
    } else {
        tracing::info!("We're reusing the previous tree.");
        None
    };

    params.text = usable_sql;

    let ctx = CompletionContext::new(&params, usable_tree.as_ref().or(params.tree));

    let mut builder = CompletionBuilder::new();

    complete_tables(&ctx, &mut builder);
    complete_functions(&ctx, &mut builder);
    complete_columns(&ctx, &mut builder);

    builder.finish()
}

fn cursor_inbetween_nodes(tree: &tree_sitter::Tree, position: TextSize) -> bool {
    let mut cursor = tree.walk();
    let mut node = tree.root_node();

    loop {
        let child_dx = cursor.goto_first_child_for_byte(position.into());
        if child_dx.is_none() {
            break;
        }
        node = cursor.node();
    }

    let byte = position.into();

    // Return true if the cursor is NOT within the node's bounds, INCLUSIVE
    !(node.start_byte() <= byte && node.end_byte() >= byte)
}

fn cursor_prepared_to_write_token_after_last_node(
    tree: &tree_sitter::Tree,
    position: TextSize,
) -> bool {
    let cursor_pos: usize = position.into();
    cursor_pos == tree.root_node().end_byte() + 1
}

#[cfg(test)]
mod tests {
    use pgt_text_size::TextSize;

    use crate::complete::{cursor_inbetween_nodes, cursor_prepared_to_write_token_after_last_node};

    #[test]
    fn test_cursor_inbetween_nodes() {
        let input = "select  from users;";

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(tree_sitter_sql::language())
            .expect("Error loading sql language");

        let mut tree = parser.parse(input.to_string(), None).unwrap();

        // select | from users;
        assert!(cursor_inbetween_nodes(&mut tree, TextSize::new(7)));

        // select  |from users;
        assert!(!cursor_inbetween_nodes(&mut tree, TextSize::new(8)));

        // select|  from users;
        assert!(!cursor_inbetween_nodes(&mut tree, TextSize::new(6)));
    }

    #[test]
    fn test_cursor_after_nodes() {
        let input = "select * from ";

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(tree_sitter_sql::language())
            .expect("Error loading sql language");

        let mut tree = parser.parse(input.to_string(), None).unwrap();

        // select * from|; <-- still on previous token
        assert!(!cursor_prepared_to_write_token_after_last_node(
            &mut tree,
            TextSize::new(14)
        ));

        // select * from  |; <-- too far off
        assert!(!cursor_prepared_to_write_token_after_last_node(
            &mut tree,
            TextSize::new(16)
        ));

        // select * from |; <-- just right
        assert!(cursor_prepared_to_write_token_after_last_node(
            &mut tree,
            TextSize::new(15)
        ));
    }
}
