use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

// TODO make this selectable via feature flags
static LIBPG_QUERY_TAG: &str = "17-6.1.0";

/// Downloads the `kwlist.h` file from the specified version of `libpg_query`
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let version = LIBPG_QUERY_TAG.to_string();

    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let vendor_dir = out_dir.join("vendor");
    let libpg_query_dir = vendor_dir.join("libpg_query").join(&version);
    let kwlist_path = libpg_query_dir.join("kwlist.h");
    let stamp_file = libpg_query_dir.join(".stamp");

    if !stamp_file.exists() {
        println!(
            "cargo:warning=Downloading kwlist.h for libpg_query {}",
            version
        );

        fs::create_dir_all(&libpg_query_dir)?;

        let proto_url = format!(
            "https://raw.githubusercontent.com/pganalyze/libpg_query/{}/src/postgres/include/parser/kwlist.h",
            version
        );

        let response = ureq::get(&proto_url).call()?;
        let content = response.into_string()?;

        let mut file = fs::File::create(&kwlist_path)?;
        file.write_all(content.as_bytes())?;

        fs::File::create(&stamp_file)?;

        println!("cargo:warning=Successfully downloaded kwlist.h");
    }

    println!(
        "cargo:rustc-env=PG_QUERY_KWLIST_PATH={}",
        kwlist_path.display()
    );

    println!("cargo:rerun-if-changed={}", stamp_file.display());

    Ok(())
}
