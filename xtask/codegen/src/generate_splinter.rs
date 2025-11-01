use anyhow::{Context, Result};
use biome_string_case::Case;
use std::collections::BTreeMap;
use std::fs;
use xtask::{glue::fs2, project_root};

/// Generate splinter categories from the SQL file
pub fn generate_splinter() -> Result<()> {
    let sql_path = project_root().join("crates/pgls_splinter/vendor/splinter.sql");
    let sql_content = fs::read_to_string(&sql_path)
        .with_context(|| format!("Failed to read SQL file at {:?}", sql_path))?;

    let rules = extract_rules_from_sql(&sql_content)?;

    update_categories_file(rules)?;

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
                // Look ahead for remediation URL
                let mut remediation_url = None;
                for j in i..std::cmp::min(i + 30, lines.len()) {
                    let next_line = lines[j].trim();
                    if next_line.contains(" as \"remediation!\"") {
                        remediation_url = extract_string_literal(next_line);
                        break;
                    }
                }

                let url = remediation_url.with_context(|| {
                    format!("Failed to find remediation URL for rule '{}'", name)
                })?;

                rules.insert(
                    name.clone(),
                    RuleInfo {
                        snake_case: name.clone(),
                        camel_case: snake_to_camel_case(&name),
                        url,
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
            url: "https://pg-language-server.com/latest".to_string(),
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

/// Convert snake_case to camelCase
fn snake_to_camel_case(s: &str) -> String {
    Case::Camel.convert(s)
}

struct RuleInfo {
    #[allow(dead_code)]
    snake_case: String,
    camel_case: String,
    url: String,
}

/// Update the categories.rs file with splinter rules
fn update_categories_file(rules: BTreeMap<String, RuleInfo>) -> Result<()> {
    let categories_path =
        project_root().join("crates/pgls_diagnostics_categories/src/categories.rs");

    let content = fs2::read_to_string(&categories_path)?;

    // Generate splinter rule entries
    let mut splinter_rules: Vec<String> = rules
        .values()
        .map(|rule| {
            format!(
                "    \"dblint/splinter/{}\": \"{}\",",
                rule.camel_case, rule.url
            )
        })
        .collect();

    splinter_rules.sort();
    let splinter_entries = splinter_rules.join("\n");

    // Replace content between splinter rules markers
    let rules_start = "// splinter rules start";
    let rules_end = "// splinter rules end";

    let new_content = replace_between_markers(
        &content,
        rules_start,
        rules_end,
        &format!("\n{}\n    ", splinter_entries),
    )?;

    // Replace content between splinter groups markers
    let groups_start = "// splinter groups start";
    let groups_end = "// splinter groups end";

    let groups_content = "\n    \"dblint\",\n    \"dblint/splinter\",\n    ";

    let new_content =
        replace_between_markers(&new_content, groups_start, groups_end, groups_content)?;

    fs2::write(categories_path, new_content)?;

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
        .with_context(|| format!("Could not find '{}' marker", start_marker))?;

    let end_pos = content
        .find(end_marker)
        .with_context(|| format!("Could not find '{}' marker", end_marker))?;

    let mut result = String::new();
    result.push_str(&content[..start_pos + start_marker.len()]);
    result.push_str(new_content);
    result.push_str(&content[end_pos..]);

    Ok(result)
}
