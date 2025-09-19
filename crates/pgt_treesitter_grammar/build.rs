fn main() {
    let grammar_file = std::path::Path::new("grammar.js");
    let src_dir = std::path::Path::new("src");
    let parser_path = src_dir.join("parser.c");

    // regenerate parser if grammar.js changes
    println!("cargo:rerun-if-changed={}", grammar_file.to_str().unwrap());

    // generate parser if it does not exist.
    if !parser_path.exists() || is_file_newer(grammar_file, parser_path.as_path()) {
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

    c_config.compile("tree_sitter_pgls");
}

fn is_file_newer(file1: &std::path::Path, file2: &std::path::Path) -> bool {
    if !file1.exists() || !file2.exists() {
        return true;
    }

    let modified1 = file1.metadata().unwrap().modified().unwrap();
    let modified2 = file2.metadata().unwrap().modified().unwrap();

    modified1 > modified2
}
