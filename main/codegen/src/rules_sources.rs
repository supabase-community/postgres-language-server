use anyhow::Result;
use biome_string_case::Case;
use pgt_analyse::RuleMetadata;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::Path;

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
    let rule_sources_file = docs_dir.join("rule_sources.md");

    let mut visitor = crate::utils::LintRulesVisitor::default();
    pgt_analyser::visit_registry(&mut visitor);

    let crate::utils::LintRulesVisitor { groups } = visitor;

    let mut buffer = Vec::new();

    let rules = groups
        .into_iter()
        .flat_map(|(_, rule)| rule)
        .collect::<BTreeMap<&str, RuleMetadata>>();

    let mut rules_by_source = BTreeMap::<String, BTreeSet<SourceSet>>::new();
    let mut exclusive_rules = BTreeSet::<(String, String)>::new();

    for (rule_name, metadata) in rules {
        let kebab_rule_name = Case::Kebab.convert(rule_name);
        if metadata.sources.is_empty() {
            exclusive_rules.insert((rule_name.to_string(), format!("./rules/{kebab_rule_name}")));
        } else {
            for source in metadata.sources {
                let source_set = SourceSet {
                    rule_name: rule_name.to_string(),
                    link: format!("./rules/{kebab_rule_name}"),
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

    writeln!(buffer, "## Exclusive rules",)?;
    for (rule, link) in exclusive_rules {
        writeln!(buffer, "- [{rule}]({link}) ")?;
    }

    writeln!(buffer, "## Rules from other sources",)?;

    for (source, rules) in rules_by_source {
        writeln!(buffer, "### {source}")?;
        writeln!(buffer, r#"| {source} Rule Name | Rule Name |"#)?;
        writeln!(buffer, r#"| ---- | ---- |"#)?;

        push_to_table(rules, &mut buffer)?;
    }

    let new_content = String::from_utf8(buffer)?;

    fs::write(rule_sources_file, new_content)?;

    Ok(())
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
