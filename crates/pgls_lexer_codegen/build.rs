use std::env;
use std::path::PathBuf;

/// Points the macro at `kwlist.h` in the libpg_query submodule that `pgls_query` builds, so the
/// keywords always match the parser.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let kwlist_path = manifest_dir
        .join("../pgls_query/vendor/libpg_query/src/postgres/include/parser/kwlist.h")
        .canonicalize()
        .map_err(|error| {
            format!("kwlist.h not found ({error}). Run: git submodule update --init --recursive")
        })?;

    println!(
        "cargo:rustc-env=PG_QUERY_KWLIST_PATH={}",
        kwlist_path.display()
    );
    println!("cargo:rerun-if-changed={}", kwlist_path.display());

    Ok(())
}
