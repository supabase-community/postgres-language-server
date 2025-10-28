use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

static LIBPG_QUERY_TAG: &str = "17-latest";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let vendor_dir = manifest_dir.join("vendor").join(LIBPG_QUERY_TAG);
    let proto_path = vendor_dir.join("pg_query.proto");

    // Download proto file if not already present in source directory
    if !proto_path.exists() {
        println!(
            "cargo:warning=Downloading pg_query.proto for libpg_query {LIBPG_QUERY_TAG} to source directory"
        );

        fs::create_dir_all(&vendor_dir)?;

        let proto_url = format!(
            "https://raw.githubusercontent.com/pganalyze/libpg_query/{LIBPG_QUERY_TAG}/protobuf/pg_query.proto"
        );

        let response = ureq::get(&proto_url).call()?;
        let proto_content = response.into_string()?;

        let mut file = fs::File::create(&proto_path)?;
        file.write_all(proto_content.as_bytes())?;

        println!("cargo:warning=Successfully downloaded pg_query.proto to source");
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
