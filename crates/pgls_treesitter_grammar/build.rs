use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("Missing CARGO_MANIFEST_DIR"));
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("Missing OUT_DIR"));
    let grammar_file = manifest_dir.join("grammar.js");
    let config_file = manifest_dir.join("tree-sitter.json");
    let src_dir = manifest_dir.join("src");
    let scanner_path = src_dir.join("scanner.c");
    let generated_dir = out_dir.join("generated");
    let parser_path = generated_dir.join("parser.c");
    let node_types_path = generated_dir.join("node-types.json");
    let stamp_path = generated_dir.join(".stamp");

    println!("cargo:rerun-if-changed={}", grammar_file.display());
    println!("cargo:rerun-if-changed={}", config_file.display());
    println!("cargo:rerun-if-changed={}", scanner_path.display());
    println!(
        "cargo:rerun-if-changed={}",
        manifest_dir.join("build.rs").display()
    );

    // Detect Emscripten target for WASM builds.
    let target = std::env::var("TARGET").unwrap_or_default();
    let is_emscripten = target.contains("emscripten");

    std::fs::create_dir_all(&generated_dir).expect("Failed to create generated output directory");

    let stamp = compute_stamp([&grammar_file, &config_file]);
    let should_regenerate =
        !parser_path.exists() || !node_types_path.exists() || read_stamp(&stamp_path) != stamp;

    if should_regenerate {
        generate_grammar(&grammar_file, &config_file, &generated_dir);
        std::fs::write(&stamp_path, &stamp).expect("Failed to write grammar stamp");
    }

    let mut c_config = cc::Build::new();

    // Use Emscripten compiler for WASM builds.
    if is_emscripten {
        c_config.compiler("emcc").archiver("emar");
    }

    // Generated parser.c includes tree_sitter headers from generated_dir/tree_sitter.
    // scanner.c still lives in src/, so both include roots are required.
    c_config
        .std("c11")
        .include(&generated_dir)
        .include(&src_dir);

    #[cfg(target_env = "msvc")]
    c_config.flag("-utf-8");

    c_config.file(&parser_path);
    if scanner_path.exists() {
        c_config.file(&scanner_path);
    }

    c_config.compile("tree_sitter_pgls");
}

fn compute_stamp(files: [&Path; 2]) -> String {
    let mut hasher = DefaultHasher::new();

    for file in files {
        file.as_os_str().hash(&mut hasher);
        let contents = std::fs::read(file).unwrap_or_else(|error| {
            panic!("Failed to read {}: {error}", file.display());
        });
        contents.hash(&mut hasher);
    }

    format!("{:016x}", hasher.finish())
}

fn read_stamp(stamp_path: &Path) -> String {
    std::fs::read_to_string(stamp_path)
        .map(|value| value.trim().to_owned())
        .unwrap_or_default()
}

fn generate_grammar(grammar_file: &Path, config_file: &Path, generated_dir: &Path) {
    // tree-sitter generate updates tree-sitter.json in its working directory.
    // Use an isolated temp workdir under OUT_DIR to avoid mutating repository files.
    let generator_workdir = generated_dir.join("tree-sitter-workdir");
    let work_grammar = generator_workdir.join("grammar.js");
    let work_config = generator_workdir.join("tree-sitter.json");

    let _ = std::fs::remove_dir_all(&generator_workdir);
    std::fs::create_dir_all(&generator_workdir)
        .expect("Failed to create temporary tree-sitter generator workdir");
    std::fs::copy(grammar_file, &work_grammar).unwrap_or_else(|error| {
        panic!(
            "Failed to copy {} into generator workdir: {error}",
            grammar_file.display()
        );
    });
    std::fs::copy(config_file, &work_config).unwrap_or_else(|error| {
        panic!(
            "Failed to copy {} into generator workdir: {error}",
            config_file.display()
        );
    });

    let output = Command::new("tree-sitter")
        .arg("generate")
        .arg("grammar.js")
        .arg("--output")
        .arg(generated_dir)
        .current_dir(&generator_workdir)
        .output();

    let _ = std::fs::remove_dir_all(&generator_workdir);

    match output {
        Ok(result) if result.status.success() => {}
        Ok(result) => {
            panic!(
                "Failed to generate tree-sitter grammar.\nstdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&result.stdout),
                String::from_utf8_lossy(&result.stderr)
            );
        }
        Err(error) => {
            panic!(
                "tree-sitter CLI not found ({error}). Please install it with: `just install-tools`"
            );
        }
    }
}
