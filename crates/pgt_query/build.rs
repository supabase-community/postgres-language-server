#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]

use fs_extra::dir::CopyOptions;
use glob::glob;
use std::env;
use std::path::PathBuf;
use std::process::Command;

static LIBRARY_NAME: &str = "pg_query";
static LIBPG_QUERY_REPO: &str = "https://github.com/pganalyze/libpg_query.git";
fn get_libpg_query_tag() -> &'static str {
    #[cfg(feature = "postgres-15")]
    return "15-5.3.0";
    #[cfg(feature = "postgres-16")]
    return "16-6.1.0";
    #[cfg(feature = "postgres-17")]
    return "17-6.1.0";
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let libpg_query_tag = get_libpg_query_tag();
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let vendor_dir = out_dir.join("vendor");
    let libpg_query_dir = vendor_dir.join("libpg_query").join(libpg_query_tag);
    let stamp_file = libpg_query_dir.join(".stamp");

    let src_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?).join("src");
    let target = env::var("TARGET").unwrap();
    let is_emscripten = target.contains("emscripten");

    // Configure cargo through stdout
    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static={LIBRARY_NAME}");

    // Clone libpg_query if not already present
    if !stamp_file.exists() {
        println!("cargo:warning=Cloning libpg_query {}", libpg_query_tag);

        // Create vendor directory
        std::fs::create_dir_all(&vendor_dir)?;

        // Clone the repository with partial clone for faster download
        let status = Command::new("git")
            .args([
                "clone",
                "--filter=blob:none",
                "--depth",
                "1",
                "--branch",
                libpg_query_tag,
                LIBPG_QUERY_REPO,
                libpg_query_dir.to_str().unwrap(),
            ])
            .status()?;

        if !status.success() {
            return Err("Failed to clone libpg_query".into());
        }

        // Create stamp file
        std::fs::File::create(&stamp_file)?;
    }

    // Tell cargo to rerun if the stamp file is deleted
    println!("cargo:rerun-if-changed={}", stamp_file.display());

    // Copy necessary files to OUT_DIR for compilation
    let out_header_path = out_dir.join(LIBRARY_NAME).with_extension("h");
    let out_protobuf_path = out_dir.join("protobuf");

    let source_paths = vec![
        libpg_query_dir.join(LIBRARY_NAME).with_extension("h"),
        libpg_query_dir.join("Makefile"),
        libpg_query_dir.join("src"),
        libpg_query_dir.join("protobuf"),
        libpg_query_dir.join("vendor"),
    ];

    let copy_options = CopyOptions {
        overwrite: true,
        ..CopyOptions::default()
    };

    fs_extra::copy_items(&source_paths, &out_dir, &copy_options)?;

    // Compile the C library.
    let mut build = cc::Build::new();

    // Configure for Emscripten if needed
    if is_emscripten {
        // Use emcc as the compiler instead of gcc/clang
        build.compiler("emcc");
        // Use emar as the archiver instead of ar
        build.archiver("emar");
        // Note: We don't add WASM-specific flags here as this creates a static library
        // The final linking flags should be added when building the final WASM module
    }

    build
        .files(
            glob(out_dir.join("src/*.c").to_str().unwrap())
                .unwrap()
                .map(|p| p.unwrap()),
        )
        .files(
            glob(out_dir.join("src/postgres/*.c").to_str().unwrap())
                .unwrap()
                .map(|p| p.unwrap()),
        )
        .file(out_dir.join("vendor/protobuf-c/protobuf-c.c"))
        .file(out_dir.join("vendor/xxhash/xxhash.c"))
        .file(out_dir.join("protobuf/pg_query.pb-c.c"))
        .include(out_dir.join("."))
        .include(out_dir.join("./vendor"))
        .include(out_dir.join("./src/postgres/include"))
        .include(out_dir.join("./src/include"))
        .warnings(false); // Avoid unnecessary warnings, as they are already considered as part of libpg_query development
    if env::var("PROFILE").unwrap() == "debug" || env::var("DEBUG").unwrap() == "1" {
        build.define("USE_ASSERT_CHECKING", None);
    }
    if target.contains("windows") && !is_emscripten {
        build.include(out_dir.join("./src/postgres/include/port/win32"));
        if target.contains("msvc") {
            build.include(out_dir.join("./src/postgres/include/port/win32_msvc"));
        }
    }
    build.compile(LIBRARY_NAME);

    // Generate bindings for Rust
    let mut bindgen_builder = bindgen::Builder::default()
        .header(out_header_path.to_str().ok_or("Invalid header path")?)
        // Allowlist only the functions we need
        .allowlist_function("pg_query_parse_protobuf")
        .allowlist_function("pg_query_scan")
        .allowlist_function("pg_query_deparse_protobuf")
        .allowlist_function("pg_query_normalize")
        .allowlist_function("pg_query_fingerprint")
        .allowlist_function("pg_query_split_with_parser")
        .allowlist_function("pg_query_split_with_scanner")
        .allowlist_function("pg_query_free_protobuf_parse_result")
        .allowlist_function("pg_query_free_scan_result")
        .allowlist_function("pg_query_free_deparse_result")
        .allowlist_function("pg_query_free_normalize_result")
        .allowlist_function("pg_query_free_fingerprint_result")
        .allowlist_function("pg_query_free_split_result")
        // Allowlist the types used by these functions
        .allowlist_type("PgQueryProtobufParseResult")
        .allowlist_type("PgQueryScanResult")
        .allowlist_type("PgQueryError")
        .allowlist_type("PgQueryProtobuf")
        .allowlist_type("PgQueryDeparseResult")
        .allowlist_type("PgQueryNormalizeResult")
        .allowlist_type("PgQueryFingerprintResult")
        .allowlist_type("PgQuerySplitResult")
        .allowlist_type("PgQuerySplitStmt")
        // Also generate bindings for size_t since it's used in PgQueryProtobuf
        .allowlist_type("size_t")
        .allowlist_var("PG_VERSION_NUM");

    // Configure bindgen for Emscripten target
    if is_emscripten {
        // Tell bindgen to generate bindings for the wasm32 target
        bindgen_builder = bindgen_builder.clang_arg("--target=wasm32-unknown-emscripten");

        // Add emscripten sysroot includes
        // First try to use EMSDK environment variable (set in CI and when sourcing emsdk_env.sh)
        if let Ok(emsdk) = env::var("EMSDK") {
            bindgen_builder = bindgen_builder.clang_arg(format!(
                "-I{}/upstream/emscripten/cache/sysroot/include",
                emsdk
            ));
        } else {
            // Fallback to the default path if EMSDK is not set
            bindgen_builder =
                bindgen_builder.clang_arg("-I/emsdk/upstream/emscripten/cache/sysroot/include");
        }

        // Ensure we have the basic C standard library headers
        bindgen_builder = bindgen_builder.clang_arg("-D__EMSCRIPTEN__");

        // Use environment variable if set (from our justfile)
        if let Ok(extra_args) = env::var("BINDGEN_EXTRA_CLANG_ARGS") {
            for arg in extra_args.split_whitespace() {
                bindgen_builder = bindgen_builder.clang_arg(arg);
            }
        }
    }

    let bindings = bindgen_builder
        .generate()
        .map_err(|_| "Unable to generate bindings")?;

    let bindings_path = src_dir.join("bindings.rs");
    bindings.write_to_file(&bindings_path)?;

    // For WASM/emscripten builds, manually add the function declarations
    // since bindgen sometimes misses them due to preprocessor conditions
    if is_emscripten {
        let mut bindings_content = std::fs::read_to_string(&bindings_path)?;

        // Check if we need to add the extern "C" block
        if !bindings_content.contains("extern \"C\"") {
            bindings_content.push_str("\nextern \"C\" {\n");
            bindings_content.push_str("    pub fn pg_query_scan(input: *const ::std::os::raw::c_char) -> PgQueryScanResult;\n");
            bindings_content.push_str("    pub fn pg_query_parse_protobuf(input: *const ::std::os::raw::c_char) -> PgQueryProtobufParseResult;\n");
            bindings_content.push_str("    pub fn pg_query_deparse_protobuf(protobuf: PgQueryProtobuf) -> PgQueryDeparseResult;\n");
            bindings_content.push_str("    pub fn pg_query_normalize(input: *const ::std::os::raw::c_char) -> PgQueryNormalizeResult;\n");
            bindings_content.push_str("    pub fn pg_query_fingerprint(input: *const ::std::os::raw::c_char) -> PgQueryFingerprintResult;\n");
            bindings_content.push_str("    pub fn pg_query_split_with_parser(input: *const ::std::os::raw::c_char) -> PgQuerySplitResult;\n");
            bindings_content.push_str("    pub fn pg_query_split_with_scanner(input: *const ::std::os::raw::c_char) -> PgQuerySplitResult;\n");
            bindings_content
                .push_str("    pub fn pg_query_free_scan_result(result: PgQueryScanResult);\n");
            bindings_content.push_str("    pub fn pg_query_free_protobuf_parse_result(result: PgQueryProtobufParseResult);\n");
            bindings_content.push_str(
                "    pub fn pg_query_free_deparse_result(result: PgQueryDeparseResult);\n",
            );
            bindings_content.push_str(
                "    pub fn pg_query_free_normalize_result(result: PgQueryNormalizeResult);\n",
            );
            bindings_content.push_str(
                "    pub fn pg_query_free_fingerprint_result(result: PgQueryFingerprintResult);\n",
            );
            bindings_content
                .push_str("    pub fn pg_query_free_split_result(result: PgQuerySplitResult);\n");
            bindings_content.push_str("}\n");

            std::fs::write(&bindings_path, bindings_content)?;
        }
    }

    let protoc_exists = Command::new("protoc").arg("--version").status().is_ok();
    if protoc_exists {
        println!("generating protobuf bindings");
        // HACK: Set OUT_DIR to src/ so that the generated protobuf file is copied to src/protobuf.rs
        unsafe {
            env::set_var("OUT_DIR", &src_dir);
        }

        prost_build::compile_protos(
            &[&out_protobuf_path.join(LIBRARY_NAME).with_extension("proto")],
            &[&out_protobuf_path],
        )?;

        std::fs::rename(src_dir.join("pg_query.rs"), src_dir.join("protobuf.rs"))?;

        // Reset OUT_DIR to the original value
        unsafe {
            env::set_var("OUT_DIR", &out_dir);
        }
    } else {
        println!("skipping protobuf generation");
    }

    Ok(())
}
