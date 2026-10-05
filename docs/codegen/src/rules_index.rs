use convert_case::{Case, Casing};
use pgls_analyse::RuleMetadata;
use pgls_console::fmt::{Formatter, HTML};
use pgls_console::{Markup, markup};
use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use std::{
    collections::BTreeMap,
    fs,
    io::{self},
    path::Path,
    str::{self},
};

use crate::utils::{self, SplinterRuleMetadata};

/// Generates the lint rules index.
///
/// * `docs_dir`: Path to the docs directory.
pub fn generate_rules_index(docs_dir: &Path) -> anyhow::Result<()> {
    let index_file = docs_dir.join("reference/rules.md");

    let mut visitor = crate::utils::LintRulesVisitor::default();
    pgls_analyser::visit_registry(&mut visitor);

    let crate::utils::LintRulesVisitor { groups } = visitor;

    let mut content = Vec::new();

    for (group, rules) in groups {
        generate_group(group, rules, &mut content)?;
    }

    let new_content = String::from_utf8(content)?;

    let file_content = fs::read_to_string(&index_file)?;

    let new_content = utils::replace_section(&file_content, "RULES_INDEX", &new_content);

    fs::write(index_file, new_content)?;

    Ok(())
}

fn generate_group(
    group: &'static str,
    rules: BTreeMap<&'static str, RuleMetadata>,
    content: &mut dyn io::Write,
) -> io::Result<()> {
    let (group_name, description) = extract_group_metadata(group);

    writeln!(content, "\n## {group_name}")?;
    writeln!(content)?;
    if let Some(desc) = description {
        write_markup_to_string(content, desc)?;
        writeln!(content)?;
        writeln!(content)?;
    }
    writeln!(content, "| Rule | Description |")?;
    writeln!(content, "| --- | --- |")?;

    for (rule_name, rule_metadata) in rules {
        let dashed_rule = rule_name.to_case(Case::Kebab);

        let mut markers = String::new();
        if rule_metadata.recommended {
            markers.push_str(" ✅");
        }
        if rule_metadata.applies_to == pgls_analyse::AppliesTo::Migration {
            markers.push_str(" 🛠");
        }
        if group == pgls_analyser::TYPECHECK_GROUP {
            markers.push_str(" 🔌");
        }

        let summary = generate_rule_summary(rule_metadata.docs)?;

        write!(
            content,
            "| [{rule_name}](./rules/{dashed_rule}.md){markers} | {summary} |"
        )?;

        writeln!(content)?;
    }

    Ok(())
}

fn extract_group_metadata(group: &str) -> (&str, Option<Markup<'_>>) {
    match group {
        "correctness" => (
            "Correctness",
            Some(markup! {
                "Code that fails at runtime for reasons other than names or types."
            }),
        ),
        "safety" => (
            "Safety",
            Some(markup! {
                "Valid code that may be dangerous against a live database: locks, rewrites, or blocking."
            }),
        ),
        "destructive" => (
            "Destructive",
            Some(markup! {
                "Code that loses data or breaks existing clients."
            }),
        ),
        "style" => (
            "Style",
            Some(markup! {
                "Schema design preferences. Not enabled by the recommended preset."
            }),
        ),
        "security" => (
            "Security",
            Some(markup! {
                "Security issues in new DDL."
            }),
        ),
        "typecheck" => (
            "Typecheck",
            Some(markup! {
                "Code that fails at runtime because of names or types."
            }),
        ),
        "nursery" => (
            "Nursery",
            Some(markup! {
                "New rules that are still being tested. Never enabled by presets."
            }),
        ),
        _ => {
            let title = group
                .chars()
                .next()
                .map(|c| c.to_uppercase().to_string())
                .unwrap_or_default()
                + &group[1..];
            // Leak the title so it lives as long as &'static str is not needed;
            // we return &str tied to group's lifetime which is 'static.
            (Box::leak(title.into_boxed_str()), None)
        }
    }
}

fn write_markup_to_string(buffer: &mut dyn io::Write, markup: Markup) -> io::Result<()> {
    let mut write = HTML::new(buffer).with_mdx();
    let mut fmt = Formatter::new(&mut write);
    fmt.write_markup(markup)
}

/// Parsed the rule documentation to extract the summary.
/// The summary is the first paragraph in the rule documentation.
fn generate_rule_summary(docs: &'static str) -> io::Result<String> {
    let parser = Parser::new(docs);

    let mut buffer = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::Paragraph) => {
                continue;
            }
            Event::Text(text) => {
                buffer.push_str(&text);
            }
            Event::Code(code) => {
                buffer.push_str(format!("`{code}`").as_str());
            }
            Event::SoftBreak | Event::HardBreak => {
                buffer.push(' ');
            }
            Event::End(TagEnd::Paragraph) => {
                return Ok(buffer);
            }
            _ => {}
        }
    }

    panic!("No summary found in rule documentation");
}

/// Generates the splinter (database linter) rules index.
///
/// * `docs_dir`: Path to the docs directory.
pub fn generate_splinter_rules_index(docs_dir: &Path) -> anyhow::Result<()> {
    let index_file = docs_dir.join("reference/database_rules.md");

    let mut visitor = crate::utils::SplinterRulesVisitor::default();
    pgls_splinter::registry::visit_registry(&mut visitor);

    let crate::utils::SplinterRulesVisitor { groups } = visitor;

    let mut content = Vec::new();

    for (group, rules) in groups {
        generate_splinter_group(group, rules, &mut content)?;
    }

    let new_content = String::from_utf8(content)?;

    let file_content = fs::read_to_string(&index_file)?;

    let new_content = utils::replace_section(&file_content, "SPLINTER_RULES_INDEX", &new_content);

    fs::write(index_file, new_content)?;

    Ok(())
}

fn generate_splinter_group(
    group: &'static str,
    rules: BTreeMap<&'static str, SplinterRuleMetadata>,
    content: &mut dyn io::Write,
) -> io::Result<()> {
    let (group_name, description) = extract_splinter_group_metadata(group);

    writeln!(content, "\n## {group_name}")?;
    writeln!(content)?;
    write_markup_to_string(content, description)?;
    writeln!(content)?;
    writeln!(content)?;
    writeln!(content, "| Rule | Description |")?;
    writeln!(content, "| --- | --- |")?;

    for (rule_name, rule_metadata) in rules {
        let is_recommended = rule_metadata.metadata.recommended;
        let requires_supabase = rule_metadata.requires_supabase;
        let dashed_rule = rule_name.to_case(Case::Kebab);

        let mut markers = String::new();
        if is_recommended {
            markers.push_str(" ✅");
        }
        if requires_supabase {
            markers.push_str(" ⚡");
        }

        let summary = utils::unescape_backticks(rule_metadata.description);

        write!(
            content,
            "| [{rule_name}](./database-rules/{dashed_rule}.md){markers} | {summary} |"
        )?;

        writeln!(content)?;
    }

    Ok(())
}

fn extract_splinter_group_metadata(group: &str) -> (&str, Markup<'_>) {
    match group {
        "performance" => (
            "Performance",
            markup! {
                "Rules that detect potential performance issues in your database schema."
            },
        ),
        "security" => (
            "Security",
            markup! {
                "Rules that detect potential security vulnerabilities in your database schema."
            },
        ),
        _ => panic!("Unknown splinter group ID {group:?}"),
    }
}
