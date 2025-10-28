use clap::*;

#[derive(Parser)]
#[command(
    name = "tree-printer",
    about = "Prints the TreeSitter tree of the given file."
)]
struct Args {
    #[arg(long = "file", short = 'f')]
    file: String,
}

fn main() {
    let args = Args::parse();

    let query = std::fs::read_to_string(&args.file).expect("Failed to read file.");

    let mut parser = tree_sitter::Parser::new();

    parser
        .set_language(&pgls_treesitter_grammar::LANGUAGE.into())
        .expect("Setting Language failed.");

    let tree = parser
        .parse(query.clone(), None)
        .expect("Failed to parse query.");

    print_tree(&tree.root_node(), &query, 0);
}

fn print_tree(node: &tree_sitter::Node, source: &str, level: usize) {
    let indent = "  ".repeat(level);

    let node_text = node
        .utf8_text(source.as_bytes())
        .unwrap_or("NO_NAME")
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ");

    println!(
        "{}{} [{}..{}] '{}'",
        indent,
        node.kind(),
        node.start_position().column,
        node.end_position().column,
        node_text
    );

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        print_tree(&child, source, level + 1);
    }
}
