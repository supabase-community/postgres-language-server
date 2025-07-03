use pgt_lexer_new::{SyntaxKind, lex};

fn main() {
    let sql = "SELECT id, name FROM users WHERE active = true;";
    let lexed = lex(sql);

    println!("Total tokens: {}", lexed.len());
    println!("\nToken details:");

    // Iterate over tokens
    for (idx, kind) in lexed.tokens().enumerate() {
        // Skip whitespace for cleaner output
        if matches!(
            kind,
            SyntaxKind::SPACE | SyntaxKind::TAB | SyntaxKind::NEWLINE
        ) {
            continue;
        }

        let range = lexed.range(idx);
        let text = lexed.text(idx);

        println!("  [{:3}] {:?} @ {:?} = {:?}", idx, kind, range, text);
    }

    // Check for errors
    let errors = lexed.errors();
    if !errors.is_empty() {
        println!("\nLexing errors:");
        for error in errors {
            println!("  Error at {:?}: {}", error.span, error.message);
        }
    } else {
        println!("\nNo lexing errors found.");
    }

    // Example: Find all identifiers
    println!("\nIdentifiers found:");
    for (idx, kind) in lexed.tokens().enumerate() {
        if kind == SyntaxKind::IDENT {
            println!("  - {} at {:?}", lexed.text(idx), lexed.range(idx));
        }
    }
}
