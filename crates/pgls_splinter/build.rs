use std::fs;
use std::path::Path;

const EXPECTED_COMMIT: &str = "b9de3a3001cbdf01dc1da327acae0700c07f0110";
const REPO: &str = "supabase/splinter";

type BuildResult<T> = Result<T, Box<dyn std::error::Error>>;

fn main() -> BuildResult<()> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
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
        fs::create_dir_all(&vendor_dir)?;

        // Discover categories by listing lints/ directory
        let categories = list_directories(REPO, EXPECTED_COMMIT, "lints")?;

        for category in &categories {
            let category_dir = vendor_dir.join(category);
            fs::create_dir_all(&category_dir)?;

            download_sql_files(
                REPO,
                EXPECTED_COMMIT,
                &format!("lints/{category}"),
                &category_dir,
            )?;
        }

        // Write commit SHA
        fs::write(&sha_file, EXPECTED_COMMIT)?;

        println!("cargo:warning=Downloaded splinter vendor files successfully");
    }

    println!("cargo:rerun-if-changed=vendor/COMMIT_SHA.txt");
    Ok(())
}

/// List subdirectories in a GitHub path
fn list_directories(repo: &str, commit: &str, path: &str) -> BuildResult<Vec<String>> {
    let api_url = format!("https://api.github.com/repos/{repo}/contents/{path}?ref={commit}");

    let json: serde_json::Value = ureq::get(&api_url)
        .set("User-Agent", "pgls-build")
        .call()
        .map_err(|error| format!("Failed to list {path}: {error}"))?
        .into_json()?;

    Ok(json
        .as_array()
        .ok_or("Expected array from GitHub API")?
        .iter()
        .filter(|item| item["type"].as_str() == Some("dir"))
        .filter_map(|item| item["name"].as_str().map(String::from))
        .collect())
}

/// Download all .sql files from a GitHub directory
fn download_sql_files(repo: &str, commit: &str, path: &str, dest_dir: &Path) -> BuildResult<()> {
    let api_url = format!("https://api.github.com/repos/{repo}/contents/{path}?ref={commit}");

    let json: serde_json::Value = ureq::get(&api_url)
        .set("User-Agent", "pgls-build")
        .call()
        .map_err(|error| format!("Failed to list {path}: {error}"))?
        .into_json()?;

    for item in json.as_array().ok_or("Expected array")? {
        let name = item["name"].as_str().ok_or("Missing name")?;
        if !name.ends_with(".sql") {
            continue;
        }

        let download_url = item["download_url"]
            .as_str()
            .ok_or("Missing download_url")?;

        let content = ureq::get(download_url)
            .call()
            .map_err(|error| format!("Failed to download {name}: {error}"))?
            .into_string()?;

        fs::write(dest_dir.join(name), content)?;
    }
    Ok(())
}
