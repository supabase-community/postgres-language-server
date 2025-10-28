use fs_extra::dir::CopyOptions;
use glob::glob;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

static LIB_NAME: &str = "pg_query";

struct Layout {
    include_dir: PathBuf,
    lib_dir: Option<PathBuf>, // Some => system/dynamic; None => vendored/static
    header: PathBuf,
    proto: Option<PathBuf>,
    c_src_roots: Vec<PathBuf>,
    extra_includes: Vec<PathBuf>,
    build_root: PathBuf, // base where vendor/protobuf live
}

fn system_layout(prefix: &Path) -> Result<Layout, String> {
    let include = prefix.join("include");
    let lib = prefix.join("lib");
    let header = include.join(format!("{LIB_NAME}.h"));
    if !header.exists() {
        return Err(format!(
            "LIBPG_QUERY_PATH set, but header not found: {}",
            header.display()
        ));
    }
    let sys_proto = prefix.join("protobuf").join(format!("{LIB_NAME}.proto"));
    Ok(Layout {
        include_dir: include,
        lib_dir: Some(lib),
        header,
        proto: sys_proto.exists().then_some(sys_proto),
        c_src_roots: vec![],
        extra_includes: vec![],
        build_root: prefix.to_path_buf(),
    })
}

fn vendored_layout(vendor_root: &Path, out_dir: &Path) -> Result<Layout, String> {
    // Ensure submodule content exists
    if !vendor_root.join("src").exists() {
        return Err(
            "libpg_query submodule not found. Run: git submodule update --init --recursive".into(),
        );
    }

    // Copy vendored tree into OUT_DIR
    let copy_opts = CopyOptions {
        overwrite: true,
        ..CopyOptions::default()
    };
    let items = vec![
        vendor_root.join(format!("{LIB_NAME}.h")),
        vendor_root.join("postgres_deparse.h"),
        vendor_root.join("Makefile"),
        vendor_root.join("src"),
        vendor_root.join("protobuf"),
        vendor_root.join("vendor"),
    ];
    fs_extra::copy_items(&items, out_dir, &copy_opts).map_err(|e| e.to_string())?;

    let root = out_dir.to_path_buf();
    let out_header = root.join(format!("{LIB_NAME}.h"));
    let out_proto = root.join("protobuf").join(format!("{LIB_NAME}.proto"));

    let extra_includes = vec![
        root.join("."),
        root.join("vendor"),
        root.join("src/postgres/include"),
        root.join("src/include"),
    ];

    Ok(Layout {
        include_dir: root.clone(),
        lib_dir: None,
        header: out_header,
        proto: out_proto.exists().then_some(out_proto),
        c_src_roots: vec![root.join("src"), root.join("src/postgres")],
        extra_includes,
        build_root: root,
    })
}

fn run_bindgen(
    header: &Path,
    include_dirs: &[PathBuf],
    is_emscripten: bool,
    out_bindings: &Path,
) -> Result<(), String> {
    let mut b = bindgen::Builder::default()
        .header(header.to_str().unwrap())
        // Allowlist only the functions we need
        .allowlist_function("pg_query_parse_protobuf")
        .allowlist_function("pg_query_scan")
        .allowlist_function("pg_query_deparse_protobuf")
        .allowlist_function("pg_query_normalize")
        .allowlist_function("pg_query_fingerprint")
        .allowlist_function("pg_query_split_with_parser")
        .allowlist_function("pg_query_split_with_scanner")
        .allowlist_function("pg_query_parse_plpgsql")
        .allowlist_function("pg_query_free_protobuf_parse_result")
        .allowlist_function("pg_query_free_scan_result")
        .allowlist_function("pg_query_free_deparse_result")
        .allowlist_function("pg_query_free_normalize_result")
        .allowlist_function("pg_query_free_fingerprint_result")
        .allowlist_function("pg_query_free_split_result")
        .allowlist_function("pg_query_free_plpgsql_parse_result")
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

    for inc in include_dirs {
        b = b.clang_arg(format!("-I{}", inc.display()));
    }

    if is_emscripten {
        b = b.clang_arg("--target=wasm32-unknown-emscripten");

        // Add emscripten sysroot includes
        if let Ok(emsdk) = env::var("EMSDK") {
            b = b.clang_arg(format!(
                "-I{emsdk}/upstream/emscripten/cache/sysroot/include"
            ));
        } else {
            b = b.clang_arg("-I/emsdk/upstream/emscripten/cache/sysroot/include");
        }

        b = b.clang_arg("-D__EMSCRIPTEN__");

        if let Ok(extra) = env::var("BINDGEN_EXTRA_CLANG_ARGS") {
            for arg in extra.split_whitespace() {
                b = b.clang_arg(arg);
            }
        }
    }

    b.generate()
        .map_err(|_| "bindgen failed".to_string())?
        .write_to_file(out_bindings)
        .map_err(|e| e.to_string())
}

fn maybe_generate_prost(proto_candidates: &[PathBuf], out_dir_src: &Path, out_dir_real: &Path) {
    let protoc_ok = Command::new("protoc")
        .arg("--version")
        .status()
        .ok()
        .map(|s| s.success())
        .unwrap_or(false);
    if !protoc_ok {
        println!("skipping protobuf generation (no protoc)");
        return;
    }
    let proto = proto_candidates.iter().find(|p| p.exists());
    if let Some(p) = proto {
        println!("generating protobuf from {}", p.display());
        unsafe {
            env::set_var("OUT_DIR", out_dir_src);
        }
        let inc = p.parent().unwrap();
        prost_build::compile_protos(&[p], &[inc]).expect("prost_build failed");
        std::fs::rename(
            out_dir_src.join("pg_query.rs"),
            out_dir_src.join("protobuf.rs"),
        )
        .ok();
        unsafe {
            env::set_var("OUT_DIR", out_dir_real);
        }
    } else {
        println!("skipping protobuf generation (no .proto found)");
    }
}

fn compile_c_if_needed(layout: &Layout, is_emscripten: bool, target: &str) {
    if layout.lib_dir.is_some() {
        return;
    } // System lib, nothing to compile.

    let mut cc = cc::Build::new();
    if is_emscripten {
        cc.compiler("emcc").archiver("emar");
    }

    for root in &layout.c_src_roots {
        let pattern = root.join("*.c");
        for p in glob(pattern.to_str().unwrap()).unwrap().flatten() {
            cc.file(p);
        }
    }

    // Add vendor files from copied tree
    cc.file(layout.build_root.join("vendor/protobuf-c/protobuf-c.c"));
    cc.file(layout.build_root.join("vendor/xxhash/xxhash.c"));
    cc.file(layout.build_root.join("protobuf/pg_query.pb-c.c"));

    for inc in &layout.extra_includes {
        cc.include(inc);
    }
    cc.warnings(false);

    let is_debug = env::var("PROFILE").ok().as_deref() == Some("debug")
        || env::var("DEBUG").ok().as_deref() == Some("1");
    if is_debug {
        cc.define("USE_ASSERT_CHECKING", None);
    }
    if target.contains("windows") && !is_emscripten {
        cc.include(layout.include_dir.join("src/postgres/include/port/win32"));
        if target.contains("msvc") {
            cc.include(
                layout
                    .include_dir
                    .join("src/postgres/include/port/win32_msvc"),
            );
        }
    }

    println!("cargo:rustc-link-lib=static={LIB_NAME}");
    cc.compile(LIB_NAME);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let src_dir = manifest_dir.join("src");
    let target = env::var("TARGET").unwrap();
    let is_emscripten = target.contains("emscripten");

    println!("cargo:rustc-link-search=native={}", out_dir.display());

    let layout = if let Ok(p) = env::var("LIBPG_QUERY_PATH") {
        println!("using system libpg_query at {p}");
        system_layout(Path::new(&p))?
    } else {
        println!("using vendored libpg_query (submodule)");
        let vendor_root = manifest_dir.join("vendor").join("libpg_query");
        vendored_layout(&vendor_root, &out_dir)?
    };

    if let Some(lib_dir) = &layout.lib_dir {
        println!("cargo:rustc-link-search=native={}", lib_dir.display());
        println!("cargo:rustc-link-lib={LIB_NAME}");
    }

    compile_c_if_needed(&layout, is_emscripten, &target);

    let mut include_dirs = vec![layout.include_dir.clone()];
    include_dirs.extend(layout.extra_includes.clone());
    let bindings_path = out_dir.join("bindings.rs");
    run_bindgen(&layout.header, &include_dirs, is_emscripten, &bindings_path)?;

    // Emscripten-specific post-processing
    if is_emscripten {
        let mut bindings_content = std::fs::read_to_string(&bindings_path)?;
        if !bindings_content.contains("extern \"C\"") {
            bindings_content.push_str("\nextern \"C\" {\n");
            bindings_content.push_str("    pub fn pg_query_scan(input: *const ::std::os::raw::c_char) -> PgQueryScanResult;\n");
            bindings_content.push_str("    pub fn pg_query_parse_protobuf(input: *const ::std::os::raw::c_char) -> PgQueryProtobufParseResult;\n");
            bindings_content.push_str("    pub fn pg_query_parse_plpgsql(input: *const ::std::os::raw::c_char) -> PgQueryPlpgsqlParseResult;\n");
            bindings_content.push_str("    pub fn pg_query_deparse_protobuf(protobuf: PgQueryProtobuf) -> PgQueryDeparseResult;\n");
            bindings_content.push_str("    pub fn pg_query_normalize(input: *const ::std::os::raw::c_char) -> PgQueryNormalizeResult;\n");
            bindings_content.push_str("    pub fn pg_query_fingerprint(input: *const ::std::os::raw::c_char) -> PgQueryFingerprintResult;\n");
            bindings_content.push_str("    pub fn pg_query_split_with_parser(input: *const ::std::os::raw::c_char) -> PgQuerySplitResult;\n");
            bindings_content.push_str("    pub fn pg_query_split_with_scanner(input: *const ::std::os::raw::c_char) -> PgQuerySplitResult;\n");
            bindings_content
                .push_str("    pub fn pg_query_free_scan_result(result: PgQueryScanResult);\n");
            bindings_content.push_str("    pub fn pg_query_free_protobuf_parse_result(result: PgQueryProtobufParseResult);\n");
            bindings_content.push_str("    pub fn pg_query_free_plpgsql_parse_result(result: PgQueryPlpgsqlParseResult);\n");
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

    // Protobuf generation (optional, uses pre-generated file as fallback)
    let candidates = layout.proto.into_iter().collect::<Vec<_>>();
    maybe_generate_prost(&candidates, &src_dir, &out_dir);

    Ok(())
}
