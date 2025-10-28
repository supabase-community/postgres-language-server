# Auto-Completions

## What does this crate do?

The `pgls_completions` identifies and ranks autocompletion items that can be displayed in your code editor.
Its main export is the `complete` function. The function takes a PostgreSQL statement, a cursor position, and a datastructure representing the underlying databases schema. It returns a list of completion items.

Postgres's statement-parsing-engine, `libpg_query`, which is used in other parts of this LSP, is only capable of parsing _complete and valid_ statements. Since autocompletion should work for incomplete statements, we rely heavily on tree-sitter – an incremental parsing library.

### Working with TreeSitter

In the `pgls_test_utils` crate, there's a binary that parses an SQL file and prints out the matching tree-sitter tree.
This makes writing tree-sitter queries for this crate easy.

To print a tree, run `cargo run --bin tree_print -- -f <your_sql_file>`.
