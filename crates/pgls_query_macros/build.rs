use std::env;
use std::path::PathBuf;

/// Points the macros at `pg_query.proto` in the libpg_query submodule that `pgls_query` builds, so
/// the generated nodes always match the parser.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let proto_path = manifest_dir
        .join("../pgls_query/vendor/libpg_query/protobuf/pg_query.proto")
        .canonicalize()
        .map_err(|error| {
            format!(
                "pg_query.proto not found ({error}). Run: git submodule update --init --recursive"
            )
        })?;

    println!(
        "cargo:rustc-env=PG_QUERY_PROTO_PATH={}",
        proto_path.display()
    );
    println!("cargo:rerun-if-changed={}", proto_path.display());

    Ok(())
}
