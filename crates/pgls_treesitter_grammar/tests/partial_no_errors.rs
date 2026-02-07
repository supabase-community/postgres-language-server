use std::sync::LazyLock;

use pgls_lexer::{SyntaxKind, lex};
use pgls_test_utils::print_ts_tree;

pub static SINGLE_TOKEN_RULES: &[&str] = &[
    "any_identifier",
    "column_identifier",
    "schema_identifier",
    "table_identifier",
    "function_identifier",
    "type_identifier",
    "type",
    "role_identifier",
    "policy_identifier",
    "object_reference",
    "table_reference",
    "column_reference",
    "function_reference",
    "type_reference",
    "literal",
    "term",
    "parameter",
    "direction",
    "field",
    "bang",
    "op_other",
    "op_unary_other",
    "comment",
    "marginalia",
];

pub static WITHOUT_END_RULES: &[&str] = &["program", "statement"];

// we're in /crates/pgls_treesitter_grammar/tests
// => ../../ is crates
static PG_REGRESSION_FILES: LazyLock<Vec<&'static str>> = std::sync::LazyLock::new(|| {
    vec![include_str!(
        "../../pgls_query/vendor/libpg_query/test/sql/postgres_regress/select.sql"
    )]
});

fn error_message(msg: &str, source: &str, tree: &tree_sitter::Tree) -> String {
    let root = tree.root_node();

    let mut printed_tree = String::new();
    print_ts_tree(&root, source, &mut printed_tree);

    format!("{msg}\n\n\n{printed_tree}")
}

fn verify_tree(tree: &tree_sitter::Tree, source: &str) -> Result<(), String> {
    let root = tree.root_node();

    if root.has_error() {
        return Err(error_message("Tree has errors!", source, tree));
    }

    verify_branches_have_ends(&root, true /* Root is always "last branch" */)
        .map_err(|e| error_message(e.as_str(), source, tree))
}

fn verify_branches_have_ends(
    branch_node: &tree_sitter::Node<'_>,
    is_last_of_parent: bool,
) -> Result<(), String> {
    if branch_node.child_count() == 0 {
        return Ok(());
    }

    if SINGLE_TOKEN_RULES.contains(&branch_node.kind()) {
        return Ok(());
    }

    if WITHOUT_END_RULES.contains(&branch_node.kind()) {
        return Ok(());
    }

    if branch_node.child_by_field_name("end").is_none() && !is_last_of_parent {
        return Err(format!(
            "Branch_node {} not at the end of the tree, yet has no @end tag.",
            branch_node.kind()
        ));
    }

    let mut cursor = branch_node.walk();
    let last = branch_node.children(&mut cursor).last().unwrap();

    for child in branch_node.children(&mut cursor) {
        verify_branches_have_ends(&child, is_last_of_parent && last == child)?;
    }

    Ok(())
}

#[test]
// #[ignore = "wip"]
fn test_grammar() {
    let mut parser = tree_sitter::Parser::new();
    if let Err(e) = parser.set_language(&pgls_treesitter_grammar::LANGUAGE.into()) {
        panic!("Language is invalid! {}", e)
    }

    let files = PG_REGRESSION_FILES.iter();

    for file in files {
        for state in generate_typing_states(file) {
            let tree = parser.parse(state.as_str(), None);

            if let Some(t) = tree {
                if let Err(msg) = verify_tree(&t, state.as_str()) {
                    panic!("\n\nGot error for statement: {}\n\n\nError: {}", state, msg)
                }
            } else {
                panic!("Unable to get tree for statement: {}", file);
            }
        }
    }
}

fn is_whitespace(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::SPACE | SyntaxKind::TAB | SyntaxKind::LINE_ENDING
    )
}

/// Generates incremental "typing states" from SQL, simulating word-by-word typing.
/// Used for testing the tree-sitter grammar with partial input.
///
/// Emit points:
/// - After each whitespace-separated "word"
/// - Immediately after `(` or `[` (with balanced close)
/// - Before `,` when inside parens/brackets
pub fn generate_typing_states(sql: &str) -> Vec<String> {
    let lexed = lex(sql);
    let mut states: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut paren_depth = 0;
    let mut bracket_depth = 0;

    let emit_state =
        |current: &str, paren_depth: i32, bracket_depth: i32, states: &mut Vec<String>| {
            let trimmed = current.trim_end().trim_end_matches(',').trim_end();
            if trimmed.is_empty() {
                return;
            }

            let mut state = trimmed.to_string();
            for _ in 0..bracket_depth {
                state.push(']');
            }
            for _ in 0..paren_depth {
                state.push(')');
            }

            if states.last() != Some(&state) {
                states.push(state);
            }
        };

    for (idx, kind) in lexed.tokens().enumerate() {
        if kind == SyntaxKind::EOF {
            break;
        }

        let text = lexed.text(idx);

        match kind {
            SyntaxKind::L_PAREN => {
                current.push_str(text);
                paren_depth += 1;
                emit_state(&current, paren_depth, bracket_depth, &mut states);
            }
            SyntaxKind::R_PAREN => {
                paren_depth -= 1;
                current.push_str(text);
            }
            SyntaxKind::L_BRACK => {
                current.push_str(text);
                bracket_depth += 1;
                emit_state(&current, paren_depth, bracket_depth, &mut states);
            }
            SyntaxKind::R_BRACK => {
                bracket_depth -= 1;
                current.push_str(text);
            }
            SyntaxKind::COMMA if paren_depth > 0 || bracket_depth > 0 => {
                emit_state(&current, paren_depth, bracket_depth, &mut states);
                current.push_str(text);
            }
            _ if is_whitespace(kind) => {
                emit_state(&current, paren_depth, bracket_depth, &mut states);
                current.push_str(text);
            }
            _ => {
                current.push_str(text);
            }
        }
    }

    emit_state(&current, paren_depth, bracket_depth, &mut states);
    states
}

#[cfg(test)]
mod generate_typing_states_tests {
    use super::*;

    #[test]
    fn simple_select() {
        let states = generate_typing_states("SELECT * FROM users");
        assert_eq!(
            states,
            vec!["SELECT", "SELECT *", "SELECT * FROM", "SELECT * FROM users"]
        );
    }

    #[test]
    fn function_with_args() {
        let states = generate_typing_states("decode(a, NULL, b)");
        assert_eq!(
            states,
            vec![
                "decode()",
                "decode(a)",
                "decode(a, NULL)",
                "decode(a, NULL, b)"
            ]
        );
    }

    #[test]
    fn string_literals() {
        let states = generate_typing_states("SELECT 'Hello World' FROM t");
        assert_eq!(
            states,
            vec![
                "SELECT",
                "SELECT 'Hello World'",
                "SELECT 'Hello World' FROM",
                "SELECT 'Hello World' FROM t"
            ]
        );
    }

    #[test]
    fn nested_parens() {
        let states = generate_typing_states("(sum(a) + 1)");
        assert_eq!(
            states,
            vec!["()", "(sum())", "(sum(a))", "(sum(a) +)", "(sum(a) + 1)"]
        );
    }

    #[test]
    fn array_brackets() {
        let states = generate_typing_states("a[1, 2]");
        assert_eq!(states, vec!["a[]", "a[1]", "a[1, 2]"]);
    }

    #[test]
    fn emtpy_sub_query_is_error() {
        let mut parser = tree_sitter::Parser::new();
        if let Err(e) = parser.set_language(&pgls_treesitter_grammar::LANGUAGE.into()) {
            panic!("Language is invalid! {}", e)
        }

        let sql = "select foo from ()";

        let tree = parser.parse(sql, None).unwrap();
        let root = tree.root_node();

        let mut printed = String::new();
        print_ts_tree(&root, sql, &mut printed);
        println!("{}", printed);

        assert!(!root.has_error())
    }
}
