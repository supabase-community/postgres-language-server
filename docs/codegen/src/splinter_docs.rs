use anyhow::Result;
use biome_string_case::Case;
use std::{fs, io::Write as _, path::Path};

use crate::utils::SplinterRuleMetadata;

/// Extract remediation URL from SQL metadata comments
fn extract_remediation_from_sql(sql: &str) -> Option<String> {
    for line in sql.lines() {
        if let Some(url) = line.strip_prefix("-- meta: remediation = ") {
            return Some(url.trim().to_string());
        }
    }
    None
}

/// Strip metadata comments from SQL content
/// Removes all lines starting with "-- meta:"
fn strip_metadata_from_sql(sql: &str) -> String {
    sql.lines()
        .filter(|line| !line.trim().starts_with("-- meta:"))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

/// Generates the documentation page for each splinter rule.
///
/// * `docs_dir`: Path to the docs directory.
pub fn generate_splinter_docs(docs_dir: &Path) -> anyhow::Result<()> {
    let rules_dir = docs_dir.join("reference/rules");

    // Ensure rules directory exists (created by linter docs generation)
    if !rules_dir.exists() {
        fs::create_dir_all(&rules_dir)?;
    }

    let mut visitor = crate::utils::SplinterRulesVisitor::default();
    pgls_splinter::registry::visit_registry(&mut visitor);

    let crate::utils::SplinterRulesVisitor { groups } = visitor;

    for (group, rules) in groups {
        for (rule, metadata) in rules {
            let content = generate_splinter_rule_doc(group, rule, metadata)?;
            let dashed_rule = Case::Kebab.convert(rule);
            fs::write(rules_dir.join(format!("{dashed_rule}.md")), content)?;
        }
    }

    Ok(())
}

fn generate_splinter_rule_doc(
    group: &'static str,
    rule: &'static str,
    splinter_meta: SplinterRuleMetadata,
) -> Result<String> {
    let meta = splinter_meta.metadata;
    let mut content = Vec::new();

    writeln!(content, "# {rule}")?;
    writeln!(content)?;

    writeln!(
        content,
        "**Diagnostic Category: `splinter/{group}/{rule}`**"
    )?;
    writeln!(content)?;

    // Add severity
    let severity_str = match meta.severity {
        pgls_diagnostics::Severity::Information => "Info",
        pgls_diagnostics::Severity::Warning => "Warning",
        pgls_diagnostics::Severity::Error => "Error",
        _ => "Info",
    };
    writeln!(content, "**Severity**: {severity_str}")?;
    writeln!(content)?;

    // Add Supabase requirement notice
    if splinter_meta.requires_supabase {
        writeln!(content, "> [!NOTE]")?;
        writeln!(
            content,
            "> This rule requires a Supabase database/project and will be automatically skipped if not detected."
        )?;
        writeln!(content)?;
    }

    writeln!(content, "## Description")?;
    writeln!(content)?;

    // Use description from SQL metadata
    writeln!(content, "{}", splinter_meta.description)?;
    writeln!(content)?;

    // Add "Learn More" link with remediation URL
    if let Some(remediation) = extract_remediation_from_sql(splinter_meta.sql_content) {
        writeln!(content, "[Learn More]({remediation})")?;
        writeln!(content)?;
    }

    // Add SQL query section (with metadata stripped)
    writeln!(content, "## SQL Query")?;
    writeln!(content)?;
    writeln!(content, "```sql")?;
    let sql_without_metadata = strip_metadata_from_sql(splinter_meta.sql_content);
    writeln!(content, "{sql_without_metadata}")?;
    writeln!(content, "```")?;
    writeln!(content)?;

    // Add configuration section
    write_how_to_configure(group, rule, &mut content)?;

    Ok(String::from_utf8(content)?)
}

fn write_how_to_configure(
    group: &'static str,
    rule: &'static str,
    content: &mut Vec<u8>,
) -> std::io::Result<()> {
    writeln!(content, "## How to configure")?;
    writeln!(content)?;

    let json = format!(
        r#"{{
  "splinter": {{
    "rules": {{
      "{group}": {{
        "{rule}": "error"
      }}
    }}
  }}
}}"#
    );

    writeln!(content, "```json")?;
    writeln!(content, "{json}")?;
    writeln!(content, "```")?;

    Ok(())
}
