use anyhow::{Context, Result};
use biome_string_case::Case;
use std::collections::BTreeMap;
use std::fs;
use xtask::{glue::fs2, project_root};

/// Generate splinter categories from the SQL files (both generic and Supabase-specific)
pub fn generate_splinter() -> Result<()> {
    let mut all_rules = BTreeMap::new();

    // Process generic rules
    let generic_sql_path = project_root().join("crates/pgls_splinter/vendor/splinter_generic.sql");
    if generic_sql_path.exists() {
        let sql_content = fs::read_to_string(&generic_sql_path)
            .with_context(|| format!("Failed to read SQL file at {generic_sql_path:?}"))?;
        let rules = extract_rules_from_sql(&sql_content)?;
        all_rules.extend(rules);
    }

    // Process Supabase-specific rules
    let supabase_sql_path =
        project_root().join("crates/pgls_splinter/vendor/splinter_supabase.sql");
    if supabase_sql_path.exists() {
        let sql_content = fs::read_to_string(&supabase_sql_path)
            .with_context(|| format!("Failed to read SQL file at {supabase_sql_path:?}"))?;
        let rules = extract_rules_from_sql(&sql_content)?;
        all_rules.extend(rules);
    }

    update_categories_file(all_rules)?;

    Ok(())
}

/// Extract rule information from the SQL file
fn extract_rules_from_sql(content: &str) -> Result<BTreeMap<String, RuleInfo>> {
    let mut rules = BTreeMap::new();

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Look for pattern: 'rule_name' as "name!",
        if line.contains(" as \"name!\"") {
            if let Some(name) = extract_string_literal(line) {
                // Look ahead for categories
                let mut categories = None;

                for j in i..std::cmp::min(i + 30, lines.len()) {
                    let next_line = lines[j].trim();

                    // Extract categories from pattern: array['CATEGORY'] as "categories!",
                    if next_line.contains(" as \"categories!\"") {
                        categories = extract_categories(next_line);
                        break; // Stop once we have categories
                    }
                }

                let cats = categories
                    .with_context(|| format!("Failed to find categories for rule '{name}'"))?;

                rules.insert(
                    name.clone(),
                    RuleInfo {
                        snake_case: name.clone(),
                        camel_case: snake_to_camel_case(&name),
                        categories: cats,
                    },
                );
            }
        }

        i += 1;
    }

    // Add the "unknown" fallback rule
    rules.insert(
        "unknown".to_string(),
        RuleInfo {
            snake_case: "unknown".to_string(),
            camel_case: "unknown".to_string(),
            categories: vec!["UNKNOWN".to_string()],
        },
    );

    Ok(rules)
}

/// Extract a string literal from a line like "'some_string' as ..."
fn extract_string_literal(line: &str) -> Option<String> {
    let trimmed = line.trim();

    if let Some(start_single) = trimmed.find('\'') {
        if let Some(end) = trimmed[start_single + 1..].find('\'') {
            return Some(trimmed[start_single + 1..start_single + 1 + end].to_string());
        }
    }

    None
}

/// Extract categories from a line like "array['CATEGORY'] as "categories!","
fn extract_categories(line: &str) -> Option<Vec<String>> {
    let trimmed = line.trim();

    // Look for array['...']
    if let Some(start) = trimmed.find("array[") {
        if let Some(end) = trimmed[start..].find(']') {
            let array_content = &trimmed[start + 6..start + end];

            // Extract all string literals within the array
            let categories: Vec<String> = array_content
                .split(',')
                .filter_map(|s| {
                    let s = s.trim();
                    if let Some(start_quote) = s.find('\'') {
                        if let Some(end_quote) = s[start_quote + 1..].find('\'') {
                            return Some(
                                s[start_quote + 1..start_quote + 1 + end_quote].to_string(),
                            );
                        }
                    }
                    None
                })
                .collect();

            if !categories.is_empty() {
                return Some(categories);
            }
        }
    }

    None
}

/// Convert snake_case to camelCase
fn snake_to_camel_case(s: &str) -> String {
    Case::Camel.convert(s)
}

struct RuleInfo {
    #[allow(dead_code)]
    snake_case: String,
    camel_case: String,
    categories: Vec<String>,
}

/// Build remediation URL from rule name
/// Must match the logic in crates/pgls_splinter/src/convert.rs
fn build_remediation_url(name: &str) -> String {
    let lint_id = match name {
        "unindexed_foreign_keys" => "0001_unindexed_foreign_keys",
        "auth_users_exposed" => "0002_auth_users_exposed",
        "auth_rls_initplan" => "0003_auth_rls_initplan",
        "no_primary_key" => "0004_no_primary_key",
        "unused_index" => "0005_unused_index",
        "multiple_permissive_policies" => "0006_multiple_permissive_policies",
        "policy_exists_rls_disabled" => "0007_policy_exists_rls_disabled",
        "rls_enabled_no_policy" => "0008_rls_enabled_no_policy",
        "duplicate_index" => "0009_duplicate_index",
        "security_definer_view" => "0010_security_definer_view",
        "function_search_path_mutable" => "0011_function_search_path_mutable",
        "rls_disabled_in_public" => "0013_rls_disabled_in_public",
        "extension_in_public" => "0014_extension_in_public",
        "rls_references_user_metadata" => "0015_rls_references_user_metadata",
        "materialized_view_in_api" => "0016_materialized_view_in_api",
        "foreign_table_in_api" => "0017_foreign_table_in_api",
        "unsupported_reg_types" => "unsupported_reg_types",
        "insecure_queue_exposed_in_api" => "0019_insecure_queue_exposed_in_api",
        "table_bloat" => "0020_table_bloat",
        "fkey_to_auth_unique" => "0021_fkey_to_auth_unique",
        "extension_versions_outdated" => "0022_extension_versions_outdated",
        _ => return "https://supabase.com/docs/guides/database/database-linter".to_string(),
    };

    format!("https://supabase.com/docs/guides/database/database-linter?lint={lint_id}")
}

/// Update the categories.rs file with splinter rules
fn update_categories_file(rules: BTreeMap<String, RuleInfo>) -> Result<()> {
    let categories_path =
        project_root().join("crates/pgls_diagnostics_categories/src/categories.rs");

    let mut content = fs2::read_to_string(&categories_path)?;

    // Generate splinter rule entries grouped by category
    let mut splinter_rules: Vec<(String, String)> = rules
        .values()
        .flat_map(|rule| {
            // For each rule, create entries for all its categories
            // In practice, splinter rules have only one category
            rule.categories.iter().map(|category| {
                let group = category.to_lowercase();
                let url = build_remediation_url(&rule.snake_case);
                (
                    group.clone(),
                    format!(
                        "    \"splinter/{}/{}\": \"{}\",",
                        group, rule.camel_case, url
                    ),
                )
            })
        })
        .collect::<Vec<_>>();

    // Sort by group, then by entry
    splinter_rules.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

    // Extract just the formatted strings
    let splinter_entries: String = splinter_rules
        .iter()
        .map(|(_, entry)| entry.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    // Replace content between splinter rules markers
    let rules_start = "// splinter rules start";
    let rules_end = "// splinter rules end";

    content = replace_between_markers(
        &content,
        rules_start,
        rules_end,
        &format!("\n{splinter_entries}\n    "),
    )?;

    // Generate splinter group entries
    let mut groups: Vec<String> = splinter_rules
        .iter()
        .map(|(group, _)| group.clone())
        .collect();
    groups.sort();
    groups.dedup();

    let mut group_entries = vec!["    \"splinter\",".to_string()];
    for group in groups {
        group_entries.push(format!("    \"splinter/{group}\","));
    }
    let groups_content = group_entries.join("\n");

    // Replace content between splinter groups markers
    let groups_start = "// Splinter groups start";
    let groups_end = "// Splinter groups end";

    content = replace_between_markers(
        &content,
        groups_start,
        groups_end,
        &format!("\n{groups_content}\n    "),
    )?;

    fs2::write(categories_path, content)?;

    Ok(())
}

/// Replace content between two markers
fn replace_between_markers(
    content: &str,
    start_marker: &str,
    end_marker: &str,
    new_content: &str,
) -> Result<String> {
    let start_pos = content
        .find(start_marker)
        .with_context(|| format!("Could not find '{start_marker}' marker"))?;

    let end_pos = content
        .find(end_marker)
        .with_context(|| format!("Could not find '{end_marker}' marker"))?;

    let mut result = String::new();
    result.push_str(&content[..start_pos + start_marker.len()]);
    result.push_str(new_content);
    result.push_str(&content[end_pos..]);

    Ok(result)
}
