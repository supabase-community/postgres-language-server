use anyhow::Result;
use convert_case::{Case, Casing};
use pgls_analyse::RuleMetadata;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::Path;

use crate::utils;

#[derive(Debug, Eq, PartialEq)]
struct SourceSet {
    source_rule_name: String,
    source_link: String,
    rule_name: String,
    link: String,
}

impl Ord for SourceSet {
    fn cmp(&self, other: &Self) -> Ordering {
        self.source_rule_name.cmp(&other.source_rule_name)
    }
}

impl PartialOrd for SourceSet {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn generate_rule_sources(docs_dir: &Path) -> anyhow::Result<()> {
    let rule_sources_file = docs_dir.join("reference/rule_sources.md");

    let mut visitor = crate::utils::LintRulesVisitor::default();
    pgls_analyser::visit_registry(&mut visitor);

    let crate::utils::LintRulesVisitor { groups } = visitor;

    let mut buffer = Vec::new();

    let rules = groups
        .into_iter()
        .flat_map(|(_, rule)| rule)
        .collect::<BTreeMap<&str, RuleMetadata>>();

    let mut rules_by_source = BTreeMap::<String, BTreeSet<SourceSet>>::new();
    let mut exclusive_rules = BTreeSet::<(String, String)>::new();

    for (rule_name, metadata) in rules {
        let kebab_rule_name = rule_name.to_case(Case::Kebab);
        if metadata.sources.is_empty() {
            exclusive_rules.insert((
                rule_name.to_string(),
                format!("./rules/{kebab_rule_name}.md"),
            ));
        } else {
            for source in metadata.sources {
                let source_set = SourceSet {
                    rule_name: rule_name.to_string(),
                    link: format!("./rules/{kebab_rule_name}.md"),
                    source_link: source.to_rule_url(),
                    source_rule_name: source.as_rule_name().to_string(),
                };

                if let Some(set) = rules_by_source.get_mut(&format!("{source}")) {
                    set.insert(source_set);
                } else {
                    let mut set = BTreeSet::new();
                    set.insert(source_set);
                    rules_by_source.insert(format!("{source}"), set);
                }
            }
        }
    }

    writeln!(buffer, "# Rule Sources",)?;
    writeln!(
        buffer,
        "Many rules are inspired by or directly ported from other tools. This page lists the sources of each rule.",
    )?;

    writeln!(buffer)?;
    writeln!(buffer, "## Exclusive rules")?;
    writeln!(buffer)?;
    if exclusive_rules.is_empty() {
        writeln!(buffer, "_No exclusive rules available._")?;
    }
    for (rule, link) in exclusive_rules {
        writeln!(buffer, "- [{rule}]({link}) ")?;
    }

    writeln!(buffer)?;
    writeln!(buffer, "## Rules from other sources")?;

    for (source, rules) in rules_by_source {
        writeln!(buffer)?;
        writeln!(buffer, "### {source}")?;
        writeln!(buffer)?;
        writeln!(buffer, r#"| {source} Rule Name | Rule Name |"#)?;
        writeln!(buffer, r#"| ---- | ---- |"#)?;

        push_to_table(rules, &mut buffer)?;
    }

    let new_content = String::from_utf8(buffer)?;

    fs::write(rule_sources_file, new_content)?;

    Ok(())
}

pub fn generate_database_rule_sources(docs_dir: &Path) -> anyhow::Result<()> {
    let rule_sources_file = docs_dir.join("reference/database_rule_sources.md");

    let mut visitor = crate::utils::SplinterRulesVisitor::default();
    pgls_splinter::registry::visit_registry(&mut visitor);

    let crate::utils::SplinterRulesVisitor { groups } = visitor;

    let rules: Vec<_> = groups
        .into_iter()
        .flat_map(|(_, rules)| rules.into_iter())
        .collect();

    // Group rules by source (currently all from Splinter)
    let mut rules_by_source = BTreeMap::<&str, Vec<(&str, &str)>>::new();

    for (rule_name, _metadata) in &rules {
        let kebab_rule_name = rule_name.to_case(Case::Kebab);
        rules_by_source
            .entry("Splinter")
            .or_default()
            .push((rule_name, Box::leak(kebab_rule_name.into_boxed_str())));
    }

    let new_content = generate_database_sources_content(&rules_by_source)?;

    let file_content = fs::read_to_string(&rule_sources_file)?;

    let new_content = utils::replace_section(&file_content, "DATABASE_RULE_SOURCES", &new_content);

    fs::write(rule_sources_file, new_content)?;

    Ok(())
}

fn generate_database_sources_content(
    rules_by_source: &BTreeMap<&str, Vec<(&str, &str)>>,
) -> Result<String> {
    let mut buffer = Vec::new();

    for (source, rules) in rules_by_source {
        let source_url = match *source {
            "Splinter" => "https://github.com/supabase/splinter",
            _ => "",
        };

        writeln!(buffer)?;
        writeln!(buffer, "### {source}")?;
        writeln!(buffer)?;
        writeln!(buffer, r#"| {source} Rule Name | Rule Name |"#)?;
        writeln!(buffer, r#"| ---- | ---- |"#)?;

        for (rule_name, kebab_rule_name) in rules {
            writeln!(
                buffer,
                "| [{rule_name}]({source_url}) | [{rule_name}](./rules/{kebab_rule_name}.md) |"
            )?;
        }
    }

    Ok(String::from_utf8(buffer)?)
}

fn push_to_table(source_set: BTreeSet<SourceSet>, buffer: &mut Vec<u8>) -> Result<()> {
    for source_set in source_set {
        write!(
            buffer,
            "| [{}]({}) |[{}]({})",
            source_set.source_rule_name,
            source_set.source_link,
            source_set.rule_name,
            source_set.link
        )?;

        writeln!(buffer, " |")?;
    }

    Ok(())
}
