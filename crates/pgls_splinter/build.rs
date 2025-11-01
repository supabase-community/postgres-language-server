use std::env;
use std::fs;
use std::path::Path;

// Update this commit SHA to pull in a new version of splinter.sql
const SPLINTER_COMMIT_SHA: &str = "27ea2ece65464213e466cd969cc61b6940d16219";

fn main() {
    let out_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let vendor_dir = Path::new(&out_dir).join("vendor");
    let sql_file = vendor_dir.join("splinter.sql");
    let sha_file = vendor_dir.join("COMMIT_SHA.txt");

    // Create vendor directory if it doesn't exist
    if !vendor_dir.exists() {
        fs::create_dir_all(&vendor_dir).expect("Failed to create vendor directory");
    }

    // Check if we need to download
    let needs_download = if !sql_file.exists() || !sha_file.exists() {
        true
    } else {
        // Check if stored SHA matches current constant
        let stored_sha = fs::read_to_string(&sha_file)
            .expect("Failed to read COMMIT_SHA.txt")
            .trim()
            .to_string();
        stored_sha != SPLINTER_COMMIT_SHA
    };

    if needs_download {
        println!(
            "cargo:warning=Downloading splinter.sql from GitHub (commit: {SPLINTER_COMMIT_SHA})"
        );
        download_and_process_sql(&sql_file);
        fs::write(&sha_file, SPLINTER_COMMIT_SHA).expect("Failed to write COMMIT_SHA.txt");
    }

    // Tell cargo to rerun if build.rs or SHA file changes
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=vendor/COMMIT_SHA.txt");
}

fn download_and_process_sql(dest_path: &Path) {
    let url = format!(
        "https://raw.githubusercontent.com/supabase/splinter/{SPLINTER_COMMIT_SHA}/splinter.sql"
    );

    // Download the file
    let response = ureq::get(&url)
        .call()
        .expect("Failed to download splinter.sql");

    let content = response
        .into_string()
        .expect("Failed to read response body");

    // Remove the SET LOCAL search_path section
    let mut processed_content = remove_set_search_path(&content);

    // Add "!" suffix to column aliases for sqlx non-null checking
    processed_content = add_not_null_markers(&processed_content);

    // Write to destination
    fs::write(dest_path, processed_content).expect("Failed to write splinter.sql");

    println!("cargo:warning=Successfully downloaded and processed splinter.sql");
}

fn remove_set_search_path(content: &str) -> String {
    content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.to_lowercase().starts_with("set local search_path")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn add_not_null_markers(content: &str) -> String {
    // Add "!" suffix to all column aliases to mark them as non-null for sqlx
    // This transforms patterns like: 'value' as name
    // Into: 'value' as "name!"

    let columns_to_mark = [
        "name",
        "title",
        "level",
        "facing",
        "categories",
        "description",
        "detail",
        "remediation",
        "metadata",
        "cache_key",
    ];

    let mut result = content.to_string();

    for column in &columns_to_mark {
        // Match patterns like: as name, as name)
        let pattern_comma = format!(" as {column}");
        let replacement_comma = format!(" as \"{column}!\"");
        result = result.replace(&pattern_comma, &replacement_comma);
    }

    result
}
