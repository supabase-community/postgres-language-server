use std::env;
use std::fs;
use std::path::Path;

// Update this commit SHA to pull in a new version of splinter.sql
const SPLINTER_COMMIT_SHA: &str = "27ea2ece65464213e466cd969cc61b6940d16219";

// Rules that work on any PostgreSQL database
const GENERIC_RULES: &[&str] = &[
    "unindexed_foreign_keys",
    "no_primary_key",
    "unused_index",
    "multiple_permissive_policies",
    "policy_exists_rls_disabled",
    "rls_enabled_no_policy",
    "duplicate_index",
    "extension_in_public",
    "table_bloat",
    "extension_versions_outdated",
    "function_search_path_mutable",
    "unsupported_reg_types",
];

// Rules that require Supabase-specific infrastructure (auth schema, anon/authenticated roles, pgrst.db_schemas)
const SUPABASE_ONLY_RULES: &[&str] = &[
    "auth_users_exposed",
    "auth_rls_initplan",
    "rls_disabled_in_public",
    "security_definer_view",
    "rls_references_user_metadata",
    "materialized_view_in_api",
    "foreign_table_in_api",
    "insecure_queue_exposed_in_api",
    "fkey_to_auth_unique",
];

fn main() {
    let out_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let vendor_dir = Path::new(&out_dir).join("vendor");
    let generic_sql_file = vendor_dir.join("splinter_generic.sql");
    let supabase_sql_file = vendor_dir.join("splinter_supabase.sql");
    let sha_file = vendor_dir.join("COMMIT_SHA.txt");

    // Create vendor directory if it doesn't exist
    if !vendor_dir.exists() {
        fs::create_dir_all(&vendor_dir).expect("Failed to create vendor directory");
    }

    // Check if we need to download
    let needs_download =
        if !generic_sql_file.exists() || !supabase_sql_file.exists() || !sha_file.exists() {
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
        download_and_process_sql(&generic_sql_file, &supabase_sql_file);
        fs::write(&sha_file, SPLINTER_COMMIT_SHA).expect("Failed to write COMMIT_SHA.txt");
    }

    // Tell cargo to rerun if build.rs or SHA file changes
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=vendor/COMMIT_SHA.txt");
}

fn download_and_process_sql(generic_dest: &Path, supabase_dest: &Path) {
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

    // Split into generic and Supabase-specific queries (validates categorization)
    let (generic_queries, supabase_queries) = split_queries(&processed_content);

    // Write to destination files
    fs::write(generic_dest, generic_queries).expect("Failed to write splinter_generic.sql");
    fs::write(supabase_dest, supabase_queries).expect("Failed to write splinter_supabase.sql");

    println!(
        "cargo:warning=Successfully downloaded and processed splinter.sql into generic and Supabase-specific files"
    );
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

/// Extract rule name from a query fragment
fn extract_rule_name_from_query(query: &str) -> String {
    // Look for pattern 'rule_name' as "name!"
    for line in query.lines() {
        if line.contains(" as \"name!\"") {
            if let Some(start) = line.rfind('\'') {
                if let Some(prev_quote) = line[..start].rfind('\'') {
                    return line[prev_quote + 1..start].to_string();
                }
            }
        }
    }
    "unknown".to_string()
}

fn split_queries(content: &str) -> (String, String) {
    // Split the union all queries based on rule names
    let queries: Vec<&str> = content.split("union all").collect();

    let mut generic_queries = Vec::new();
    let mut supabase_queries = Vec::new();

    for query in queries {
        // Extract the rule name from the query (it's the first 'name' field)
        let is_supabase = SUPABASE_ONLY_RULES
            .iter()
            .any(|rule| query.contains(&format!("'{rule}' as \"name!\"")));

        let is_generic = GENERIC_RULES
            .iter()
            .any(|rule| query.contains(&format!("'{rule}' as \"name!\"")));

        if is_supabase {
            supabase_queries.push(query);
        } else if is_generic {
            generic_queries.push(query);
        } else {
            // Extract rule name for better error message
            let rule_name = extract_rule_name_from_query(query);
            panic!(
                "Found unknown Splinter rule that is not categorized: {rule_name:?}\n\
                Please add this rule to either GENERIC_RULES or SUPABASE_ONLY_RULES in build.rs.\n\
                \n\
                Guidelines:\n\
                - GENERIC_RULES: Rules that work on any PostgreSQL database\n\
                - SUPABASE_ONLY_RULES: Rules that require Supabase infrastructure (auth schema, roles, pgrst.db_schemas)\n\
                \n\
                This prevents new Supabase-specific rules from breaking linting on non-Supabase databases."
            );
        }
    }

    // Join queries with "union all" and wrap in parentheses
    let generic_sql = if generic_queries.is_empty() {
        String::new()
    } else {
        generic_queries.join("union all\n")
    };

    let supabase_sql = if supabase_queries.is_empty() {
        String::new()
    } else {
        supabase_queries.join("union all\n")
    };

    (generic_sql, supabase_sql)
}
