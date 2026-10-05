use anyhow::Result;
use convert_case::{Case, Casing};
use std::{fs, io::Write as _, path::Path};

use crate::utils::SplinterRuleMetadata;

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
    let rules_dir = docs_dir.join("reference/database-rules");

    if rules_dir.exists() {
        fs::remove_dir_all(&rules_dir)?;
    }
    fs::create_dir_all(&rules_dir)?;

    let mut visitor = crate::utils::SplinterRulesVisitor::default();
    pgls_splinter::registry::visit_registry(&mut visitor);

    let crate::utils::SplinterRulesVisitor { groups } = visitor;

    for (group, rules) in groups {
        for (rule, metadata) in rules {
            let content = generate_splinter_rule_doc(group, rule, metadata)?;
            let dashed_rule = rule.to_case(Case::Kebab);
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

    let severity = match meta.severity {
        pgls_diagnostics::Severity::Information => "Info",
        pgls_diagnostics::Severity::Warning => "Warning",
        pgls_diagnostics::Severity::Error => "Error",
        _ => "Info",
    };
    let mut properties = vec![
        format!("**Group** [`{group}`](../database_rules.md#{group})"),
        format!("**Severity** {severity}"),
    ];
    if meta.recommended {
        properties.push("**Recommended**".to_string());
    }
    if splinter_meta.requires_supabase {
        properties.push("**Requires Supabase**".to_string());
    }
    writeln!(content, "{}  ", properties.join(" · "))?;
    writeln!(content, "**Diagnostic** `splinter/{group}/{rule}`")?;
    writeln!(content)?;

    if splinter_meta.requires_supabase {
        writeln!(content, "!!! note")?;
        writeln!(
            content,
            "    This rule needs a Supabase database. It is skipped when the Supabase roles don't exist."
        )?;
        writeln!(content)?;
    }

    writeln!(content, "## Description")?;
    writeln!(content)?;
    writeln!(
        content,
        "{}",
        crate::utils::unescape_backticks(splinter_meta.description)
    )?;
    writeln!(content)?;

    writeln!(content, "## Remediation")?;
    writeln!(content)?;
    let remediation = crate::utils::unescape_backticks(splinter_meta.remediation);
    if remediation.starts_with("http") && !remediation.contains(char::is_whitespace) {
        writeln!(
            content,
            "See the [Supabase database linter docs]({remediation})."
        )?;
    } else {
        writeln!(content, "{remediation}")?;
    }
    writeln!(content)?;

    write_how_to_configure(group, rule, &mut content)?;

    // The query is long; keep it collapsed at the end of the page.
    writeln!(content)?;
    writeln!(content, "## SQL query")?;
    writeln!(content)?;
    writeln!(content, "??? note \"Show the query\"")?;
    writeln!(content)?;
    writeln!(content, "    ```sql")?;
    for line in strip_metadata_from_sql(splinter_meta.sql_content).lines() {
        if line.is_empty() {
            writeln!(content)?;
        } else {
            writeln!(content, "    {line}")?;
        }
    }
    writeln!(content, "    ```")?;

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
