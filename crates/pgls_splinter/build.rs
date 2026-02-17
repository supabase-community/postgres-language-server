use std::fs;
use std::io::Write;
use std::path::Path;

const EXPECTED_COMMIT: &str = "b9de3a3001cbdf01dc1da327acae0700c07f0110";
const REPO: &str = "supabase/splinter";

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let vendor_dir = Path::new(&manifest_dir).join("vendor");
    let sha_file = vendor_dir.join("COMMIT_SHA.txt");

    // Check if vendor files exist and SHA matches
    let needs_download = if sha_file.exists() {
        let current_sha = fs::read_to_string(&sha_file).unwrap_or_default();
        current_sha.trim() != EXPECTED_COMMIT
    } else {
        true
    };

    if needs_download {
        println!("cargo:warning=Downloading splinter vendor files...");
        fs::create_dir_all(&vendor_dir).expect("Failed to create vendor directory");

        // Discover categories by listing lints/ directory
        let categories = list_directories(REPO, EXPECTED_COMMIT, "lints");

        for category in &categories {
            let category_dir = vendor_dir.join(category);
            fs::create_dir_all(&category_dir)
                .unwrap_or_else(|_| panic!("Failed to create vendor/{category}"));

            download_sql_files(
                REPO,
                EXPECTED_COMMIT,
                &format!("lints/{category}"),
                &category_dir,
            );
        }

        // Write commit SHA
        fs::write(&sha_file, EXPECTED_COMMIT).expect("Failed to write COMMIT_SHA.txt");

        println!("cargo:warning=Downloaded splinter vendor files successfully");
    }

    println!("cargo:rerun-if-changed=vendor/COMMIT_SHA.txt");
}

/// List subdirectories in a GitHub path
fn list_directories(repo: &str, commit: &str, path: &str) -> Vec<String> {
    let api_url = format!("https://api.github.com/repos/{repo}/contents/{path}?ref={commit}");

    let response = ureq::get(&api_url)
        .set("User-Agent", "pgls-build")
        .call()
        .unwrap_or_else(|e| panic!("Failed to list {path}: {e}"));

    let json: serde_json::Value = response
        .into_json()
        .expect("Failed to parse GitHub API response");

    json.as_array()
        .expect("Expected array from GitHub API")
        .iter()
        .filter(|item| item["type"].as_str() == Some("dir"))
        .filter_map(|item| item["name"].as_str().map(String::from))
        .collect()
}

/// Download all .sql files from a GitHub directory
fn download_sql_files(repo: &str, commit: &str, path: &str, dest_dir: &Path) {
    let api_url = format!("https://api.github.com/repos/{repo}/contents/{path}?ref={commit}");

    let response = ureq::get(&api_url)
        .set("User-Agent", "pgls-build")
        .call()
        .unwrap_or_else(|e| panic!("Failed to list {path}: {e}"));

    let json: serde_json::Value = response
        .into_json()
        .expect("Failed to parse GitHub API response");

    for item in json.as_array().expect("Expected array") {
        let name = item["name"].as_str().expect("Missing name");
        if !name.ends_with(".sql") {
            continue;
        }

        let download_url = item["download_url"].as_str().expect("Missing download_url");

        let content = ureq::get(download_url)
            .call()
            .unwrap_or_else(|_| panic!("Failed to download {name}"))
            .into_string()
            .unwrap_or_else(|_| panic!("Failed to read {name}"));

        let dest = dest_dir.join(name);
        let mut file =
            fs::File::create(&dest).unwrap_or_else(|_| panic!("Failed to create {name}"));
        file.write_all(content.as_bytes())
            .unwrap_or_else(|_| panic!("Failed to write {name}"));
    }
}
