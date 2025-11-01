use clap::*;
use pgls_test_utils::print_ts_tree;

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

    let mut result = String::new();
    print_ts_tree(&tree.root_node(), &query, 0, &mut result);

    print!("{result}")
}
