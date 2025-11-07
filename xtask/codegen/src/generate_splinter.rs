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

    for (i, line) in lines.iter().enumerate() {
        let line = line.trim();

        // Look for pattern: 'rule_name' as "name!",
        if line.contains(" as \"name!\"") {
            if let Some(name) = extract_string_literal(line) {
                // Look ahead for categories and remediation URL
                let mut categories = None;
                let mut remediation_url = None;

                for next_line in lines[i..].iter().take(30) {
                    let next_line = next_line.trim();

                    // Extract categories from pattern: array['CATEGORY'] as "categories!",
                    if next_line.contains(" as \"categories!\"") {
                        categories = extract_categories(next_line);
                    }

                    // Extract remediation URL from pattern: 'url' as "remediation!",
                    if next_line.contains(" as \"remediation!\"") {
                        remediation_url = extract_string_literal(next_line);
                    }

                    // Stop once we have both
                    if categories.is_some() && remediation_url.is_some() {
                        break;
                    }
                }

                let cats = categories
                    .with_context(|| format!("Failed to find categories for rule '{name}'"))?;

                // Convert old database-linter URLs to database-advisors
                let updated_url = remediation_url
                    .map(|url| url.replace("/database-linter", "/database-advisors"))
                    .or(Some(
                        "https://supabase.com/docs/guides/database/database-advisors".to_string(),
                    ));

                rules.insert(
                    name.clone(),
                    RuleInfo {
                        snake_case: name.clone(),
                        camel_case: snake_to_camel_case(&name),
                        categories: cats,
                        url: updated_url,
                    },
                );
            }
        }
    }

    // Add the "unknown" fallback rule
    rules.insert(
        "unknown".to_string(),
        RuleInfo {
            snake_case: "unknown".to_string(),
            camel_case: "unknown".to_string(),
            categories: vec!["UNKNOWN".to_string()],
            url: Some("https://supabase.com/docs/guides/database/database-advisors".to_string()),
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

/// Check if a string is a valid URL (simple check for http/https)
fn is_valid_url(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://")
}

struct RuleInfo {
    #[allow(dead_code)]
    snake_case: String,
    camel_case: String,
    categories: Vec<String>,
    url: Option<String>,
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

                // Use extracted URL if it's a valid URL, otherwise fallback to default
                let url = rule
                    .url
                    .as_ref()
                    .filter(|u| is_valid_url(u))
                    .map(|u| u.as_str())
                    .unwrap_or("https://supabase.com/docs/guides/database/database-advisors");

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
