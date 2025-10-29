use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

static LIBPG_QUERY_TAG: &str = "17-latest";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let vendor_dir = manifest_dir.join("vendor").join(LIBPG_QUERY_TAG);
    let kwlist_path = vendor_dir.join("kwlist.h");

    // Download kwlist.h if not already present in source directory
    if !kwlist_path.exists() {
        println!(
            "cargo:warning=Downloading kwlist.h for libpg_query {LIBPG_QUERY_TAG} to source directory"
        );

        fs::create_dir_all(&vendor_dir)?;

        let kwlist_url = format!(
            "https://raw.githubusercontent.com/pganalyze/libpg_query/{LIBPG_QUERY_TAG}/src/postgres/include/parser/kwlist.h"
        );

        let response = ureq::get(&kwlist_url).call()?;
        let content = response.into_string()?;

        let mut file = fs::File::create(&kwlist_path)?;
        file.write_all(content.as_bytes())?;

        println!("cargo:warning=Successfully downloaded kwlist.h to source");
    }

    println!(
        "cargo:rustc-env=PG_QUERY_KWLIST_PATH={}",
        kwlist_path.display()
    );

    println!("cargo:rerun-if-changed={}", kwlist_path.display());

    Ok(())
}
