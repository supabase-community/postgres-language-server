use std::fs;
use std::io::Write;
use std::path::Path;

const EXPECTED_COMMIT: &str = "main";
const REPO: &str = "pmpetit/pglinter";

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
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
        fs::create_dir_all(&sql_dir).expect("Failed to create vendor/sql directory");

        // Download rules.sql using ureq (blocking HTTP client)
        let url =
            format!("https://raw.githubusercontent.com/{REPO}/{EXPECTED_COMMIT}/sql/rules.sql");

        let response = ureq::get(&url)
            .call()
            .expect("Failed to download rules.sql");

        let content = response.into_string().expect("Failed to read response");

        let mut file = fs::File::create(&rules_file).expect("Failed to create rules.sql");
        file.write_all(content.as_bytes())
            .expect("Failed to write rules.sql");

        // Write commit SHA
        fs::write(&sha_file, EXPECTED_COMMIT).expect("Failed to write COMMIT_SHA.txt");

        println!("cargo:warning=Downloaded pglinter vendor files successfully");
    }

    // Tell cargo to rerun if SHA changes
    println!("cargo:rerun-if-changed=vendor/COMMIT_SHA.txt");
}
