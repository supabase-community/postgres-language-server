use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

// TODO make this selectable via feature flags
static LIBPG_QUERY_TAG: &str = "17-6.1.0";

/// Downloads the `kwlist.h` file from the specified version of `libpg_query`
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let version = LIBPG_QUERY_TAG.to_string();

    // Check for the postgres header file in the source tree first
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let headers_dir = manifest_dir.join("postgres").join(&version);
    let kwlist_path = headers_dir.join("kwlist.h");

    // Only download if the file doesn't exist
    if !kwlist_path.exists() {
        println!(
            "cargo:warning=Downloading kwlist.h for libpg_query {}",
            version
        );

        fs::create_dir_all(&headers_dir)?;

        let proto_url = format!(
            "https://raw.githubusercontent.com/pganalyze/libpg_query/{}/src/postgres/include/parser/kwlist.h",
            version
        );

        let response = ureq::get(&proto_url).call()?;
        let content = response.into_string()?;

        let mut file = fs::File::create(&kwlist_path)?;
        file.write_all(content.as_bytes())?;

        println!("cargo:warning=Successfully downloaded kwlist.h");
    }

    println!(
        "cargo:rustc-env=PG_QUERY_KWLIST_PATH={}",
        kwlist_path.display()
    );

    println!("cargo:rerun-if-changed={}", kwlist_path.display());

    Ok(())
}
