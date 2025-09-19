fn main() {
    let grammar_file = std::path::Path::new("grammar.js");
    let src_dir = std::path::Path::new("src");
    let parser_path = src_dir.join("parser.c");

    // regenerate parser if grammar.js changes
    println!("cargo:rerun-if-changed={}", grammar_file.to_str().unwrap());

    // generate parser files if they don't exist or grammar changed
    if !parser_path.exists() || grammar_file.exists() {
        let output = std::process::Command::new("tree-sitter")
            .arg("generate")
            .output();

        match output {
            Ok(result) if result.status.success() => {
                println!("cargo:warning=Successfully generated parser from grammar.js");
            }
            Ok(result) => {
                panic!(
                    "Failed to generate parser: {}",
                    String::from_utf8_lossy(&result.stderr)
                );
            }
            Err(_) => {
                panic!("tree-sitter CLI not found. Please install it with: `just install`");
            }
        }
    }

    let mut c_config = cc::Build::new();
    c_config.std("c11").include(src_dir);

    #[cfg(target_env = "msvc")]
    c_config.flag("-utf-8");

    c_config.file(&parser_path);
    println!("cargo:rerun-if-changed={}", parser_path.to_str().unwrap());

    let scanner_path = src_dir.join("scanner.c");
    if scanner_path.exists() {
        c_config.file(&scanner_path);
        println!("cargo:rerun-if-changed={}", scanner_path.to_str().unwrap());
    }

    c_config.compile("tree-sitter-pgls");
}
