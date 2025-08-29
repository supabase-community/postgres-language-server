use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

// TODO make this selectable via feature flags
static LIBPG_QUERY_TAG: &str = "17-6.1.0";

/// Downloads the `kwlist.h` file from the specified version of `libpg_query`
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let version = LIBPG_QUERY_TAG.to_string();

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let postgres_dir = manifest_dir.join("postgres").join(&version);
    let kwlist_path = postgres_dir.join("kwlist.h");
    let proto_path = postgres_dir.join("pg_query.proto");

    if !postgres_dir.exists() {
        fs::create_dir_all(&postgres_dir)?;
    }

    if !kwlist_path.exists() {
        println!(
            "cargo:warning=Downloading kwlist.h for libpg_query {}",
            version
        );

        let kwlist_url = format!(
            "https://raw.githubusercontent.com/pganalyze/libpg_query/{}/src/postgres/include/parser/kwlist.h",
            version
        );

        let response = ureq::get(&kwlist_url).call()?;
        let content = response.into_string()?;

        let mut file = fs::File::create(&kwlist_path)?;
        file.write_all(content.as_bytes())?;

        println!("cargo:warning=Successfully downloaded kwlist.h");
    }

    if !proto_path.exists() {
        println!(
            "cargo:warning=Downloading pg_query.proto for libpg_query {}",
            version
        );

        let proto_url = format!(
            "https://raw.githubusercontent.com/pganalyze/libpg_query/{}/protobuf/pg_query.proto",
            version
        );

        let response = ureq::get(&proto_url).call()?;
        let proto_content = response.into_string()?;

        let mut file = fs::File::create(&proto_path)?;
        file.write_all(proto_content.as_bytes())?;

        println!(
            "cargo:warning=Successfully downloaded pg_query.proto to {}",
            proto_path.display()
        );
    }

    println!(
        "cargo:rustc-env=PG_QUERY_KWLIST_PATH={}",
        kwlist_path.display()
    );

    println!(
        "cargo:rustc-env=PG_QUERY_PROTO_PATH={}",
        proto_path.display()
    );

    println!("cargo:rerun-if-changed={}", kwlist_path.display());

    println!("cargo:rerun-if-changed={}", proto_path.display());

    Ok(())
}
