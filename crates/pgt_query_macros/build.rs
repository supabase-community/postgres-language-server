use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

// This should match the version used by pgt_query crate
// You can configure this via environment variable PG_QUERY_VERSION if needed
static LIBPG_QUERY_TAG: &str = "17-6.1.0";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Allow version override via environment variable
    let version = env::var("PG_QUERY_VERSION").unwrap_or_else(|_| LIBPG_QUERY_TAG.to_string());

    // Get the manifest directory (source directory)
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let postgres_dir = manifest_dir.join("postgres");
    let proto_filename = format!("{}.proto", version);
    let proto_path = postgres_dir.join(&proto_filename);

    // Download proto file if not already present in source directory
    if !proto_path.exists() {
        println!(
            "cargo:warning=Downloading pg_query.proto for libpg_query {} to source directory",
            version
        );

        // Create postgres directory if it doesn't exist
        fs::create_dir_all(&postgres_dir)?;

        // Download the proto file
        let proto_url = format!(
            "https://raw.githubusercontent.com/pganalyze/libpg_query/{}/protobuf/pg_query.proto",
            version
        );

        let response = ureq::get(&proto_url).call()?;
        let proto_content = response.into_string()?;

        // Write proto file to source directory
        let mut file = fs::File::create(&proto_path)?;
        file.write_all(proto_content.as_bytes())?;

        println!(
            "cargo:warning=Successfully downloaded pg_query.proto to {}",
            proto_path.display()
        );
    }

    // Set environment variable for the proc macro
    println!(
        "cargo:rustc-env=PG_QUERY_PROTO_PATH={}",
        proto_path.display()
    );

    // Tell cargo to rerun if the proto file changes
    println!("cargo:rerun-if-changed={}", proto_path.display());

    Ok(())
}
