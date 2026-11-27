use std::env;
use std::path::PathBuf;

/// Points the macros at `pg_query.proto` in the libpg_query submodule that `pgls_query` builds,
/// so the nodes always match the parser.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let file = "protobuf/pg_query.proto";
    let path = manifest_dir
        .join("../pgls_query/vendor/libpg_query")
        .join(file)
        .canonicalize()
        .map_err(|error| {
            format!("{file} not found ({error}). Run: git submodule update --init --recursive")
        })?;
    println!("cargo:rustc-env=PG_QUERY_PROTO_PATH={}", path.display());
    println!("cargo:rerun-if-changed={}", path.display());

    Ok(())
}
