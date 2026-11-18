use std::fs;
use std::path::Path;

/// The pglinter commit that `vendor/sql/rules.sql`, and the rules generated from it in `src/rules`,
/// come from. Pinned, so that builds are reproducible.
const EXPECTED_COMMIT: &str = "73fc2aefec54d1d2411769e7d21dd9172defab17";
const REPO: &str = "pmpetit/pglinter";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let vendor_dir = Path::new(&manifest_dir).join("vendor");
    let sql_dir = vendor_dir.join("sql");
    let sha_file = vendor_dir.join("COMMIT_SHA.txt");
    let rules_file = sql_dir.join("rules.sql");

    // Check if vendor files exist and SHA matches
    let needs_download = if sha_file.exists() && rules_file.exists() {
        let current_sha = fs::read_to_string(&sha_file).unwrap_or_default();
        current_sha.trim() != EXPECTED_COMMIT
    } else {
        true
    };

    if needs_download {
        println!("cargo:warning=Downloading pglinter vendor files...");

        // Create directories
        fs::create_dir_all(&sql_dir)?;

        // Download rules.sql using ureq (blocking HTTP client)
        let url =
            format!("https://raw.githubusercontent.com/{REPO}/{EXPECTED_COMMIT}/sql/rules.sql");

        let content = ureq::get(&url)
            .call()
            .map_err(|error| format!("Failed to download rules.sql: {error}"))?
            .into_string()?;

        fs::write(&rules_file, content)?;

        // Write commit SHA
        fs::write(&sha_file, EXPECTED_COMMIT)?;

        println!("cargo:warning=Downloaded pglinter vendor files successfully");
    }

    // Tell cargo to rerun if SHA changes
    println!("cargo:rerun-if-changed=vendor/COMMIT_SHA.txt");
    Ok(())
}
