use pgls_analyse::{
    GroupCategory, RegistryVisitor, RuleCategory, RuleGroup, RuleMeta, RuleMetadata,
};
use regex::Regex;
use std::collections::BTreeMap;

/// Metadata for a splinter rule with SQL content and metadata from trait
#[derive(Clone)]
pub(crate) struct SplinterRuleMetadata {
    pub(crate) metadata: RuleMetadata,
    pub(crate) sql_content: &'static str,
    pub(crate) description: &'static str,
    pub(crate) remediation: &'static str,
    pub(crate) requires_supabase: bool,
}

pub(crate) fn replace_section(
    content: &str,
    section_identifier: &str,
    replacement: &str,
) -> String {
    let pattern = format!(
        r"(\[//\]: # \(BEGIN {section_identifier}\)\n)(?s).*?(\n\[//\]: # \(END {section_identifier}\))"
    );
    let re = Regex::new(&pattern).unwrap();

    // Use a replacement function instead of a replacement string to avoid
    // issues with special characters like $ in the replacement text
    re.replace_all(content, |caps: &regex::Captures| {
        format!("{}{}{}", &caps[1], replacement, &caps[2])
    })
    .to_string()
}

#[derive(Default)]
pub(crate) struct LintRulesVisitor {
    /// This is mapped to:
    /// - group (correctness) -> list of rules
    /// - list or rules is mapped to
    /// - rule name -> metadata
    pub(crate) groups: BTreeMap<&'static str, BTreeMap<&'static str, RuleMetadata>>,
}

impl LintRulesVisitor {
    fn push_rule<R>(&mut self)
    where
        R: RuleMeta + 'static,
    {
        let group = self
            .groups
            .entry(<R::Group as RuleGroup>::NAME)
            .or_default();

        group.insert(R::METADATA.name, R::METADATA);
    }
}

impl RegistryVisitor for LintRulesVisitor {
    fn record_category<C: GroupCategory>(&mut self) {
        if matches!(C::CATEGORY, RuleCategory::Lint) {
            C::record_groups(self);
        }
    }

    fn record_rule<R>(&mut self)
    where
        R: RuleMeta + 'static,
    {
        self.push_rule::<R>()
    }
}

#[derive(Default)]
pub(crate) struct SplinterRulesVisitor {
    /// This is mapped to:
    /// - group (performance, security) -> list of rules
    /// - list or rules is mapped to
    /// - rule name -> splinter metadata
    pub(crate) groups: BTreeMap<&'static str, BTreeMap<&'static str, SplinterRuleMetadata>>,
}

impl RegistryVisitor for SplinterRulesVisitor {
    fn record_category<C: GroupCategory>(&mut self) {
        // Splinter uses Lint category (like linter), so we need to accept it
        if matches!(C::CATEGORY, RuleCategory::Lint) {
            C::record_groups(self);
        }
    }

    fn record_rule<R>(&mut self)
    where
        R: RuleMeta + 'static,
    {
        let group = self
            .groups
            .entry(<R::Group as RuleGroup>::NAME)
            .or_default();

        // Get SQL content and metadata from registry
        let sql_content = pgls_splinter::registry::get_sql_content(R::METADATA.name)
            .unwrap_or("-- SQL content not found");
        let (description, remediation, requires_supabase) =
            pgls_splinter::registry::get_rule_metadata_fields(R::METADATA.name).unwrap_or((
                "Detects potential issues in your database schema.",
                "https://supabase.com/docs/guides/database/database-advisors",
                false,
            ));

        group.insert(
            R::METADATA.name,
            SplinterRuleMetadata {
                metadata: R::METADATA,
                sql_content,
                description,
                remediation,
                requires_supabase,
            },
        );
    }
}
