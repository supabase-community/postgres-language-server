//! Generated file, do not edit by hand, see `xtask/codegen`

use crate::analyser::splinter::*;
use pgls_analyse::MetadataRegistry;
use pgls_analyser::LinterRules;
pub fn push_to_analyser_splinter(
    rules: &Rules,
    metadata: &MetadataRegistry,
    analyser_rules: &mut LinterRules,
) {
    if let Some(rules) = rules.performance.as_ref() {
        for rule_name in Performance::GROUP_RULES {
            if let Some((_, Some(rule_options))) = rules.get_rule_configuration(rule_name) {
                if let Some(rule_key) = metadata.find_rule("performance", rule_name) {
                    analyser_rules.push_rule(rule_key, rule_options);
                }
            }
        }
    }
    if let Some(rules) = rules.security.as_ref() {
        for rule_name in Security::GROUP_RULES {
            if let Some((_, Some(rule_options))) = rules.get_rule_configuration(rule_name) {
                if let Some(rule_key) = metadata.find_rule("security", rule_name) {
                    analyser_rules.push_rule(rule_key, rule_options);
                }
            }
        }
    }
}
