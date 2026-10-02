//! Combines rule, group, preset, and deprecated settings into the effective configuration of
//! each linter rule.
//!
//! Precedence, from highest to lowest:
//! 1. `linter.rules.<rule>`
//! 2. the deprecated `linter.rules.safety.<rule>`
//! 3. `linter.groups.<group>`
//! 4. the presets `linter.rules.all` and `linter.rules.recommended` (never enabling `nursery`)

// This module reads the deprecated settings.
#![allow(deprecated)]

use pgls_analyse::{MetadataRegistry, RuleFilter};
use pgls_analyser::{LinterRules, RuleOptions};
use pgls_diagnostics::{Category, Severity};
use rustc_hash::FxHashSet;

use super::{Groups, LINTER_RULES, LinterRuleMetadata, Rules};
use crate::rules::RulePlainConfiguration;

/// The rule settings of the linter.
#[derive(Clone, Copy, Debug)]
pub struct LinterRuleSettings<'a> {
    rules: &'a Rules,
    groups: &'a Groups,
}

impl<'a> LinterRuleSettings<'a> {
    pub fn new(rules: &'a Rules, groups: &'a Groups) -> Self {
        Self { rules, groups }
    }

    /// The effective level of a rule. `None` if the rule doesn't exist.
    pub fn level(&self, rule: &str) -> Option<RulePlainConfiguration> {
        let metadata = find_rule(rule)?;
        Some(
            self.configured_level(metadata)
                .unwrap_or_else(|| self.preset_level(metadata)),
        )
    }

    /// The rules that are enabled.
    pub fn as_enabled_rules(&self) -> FxHashSet<RuleFilter<'static>> {
        LINTER_RULES
            .iter()
            .filter(|metadata| self.level(metadata.name) != Some(RulePlainConfiguration::Off))
            .map(|metadata| RuleFilter::Rule(metadata.group, metadata.name))
            .collect()
    }

    /// The rules that are explicitly turned off by a rule or group setting.
    pub fn as_disabled_rules(&self) -> FxHashSet<RuleFilter<'static>> {
        LINTER_RULES
            .iter()
            .filter(|metadata| self.configured_level(metadata) == Some(RulePlainConfiguration::Off))
            .map(|metadata| RuleFilter::Rule(metadata.group, metadata.name))
            .collect()
    }

    /// The severity of a diagnostic of the category `lint/<rule>`. Also accepts the former
    /// `lint/<group>/<rule>`.
    pub fn get_severity_from_code(&self, category: &Category) -> Option<Severity> {
        let mut parts = category.name().split('/');
        if parts.next() != Some("lint") {
            return None;
        }
        let rule = parts.next_back()?;
        let metadata = find_rule(rule)?;
        match self.level(rule)? {
            RulePlainConfiguration::Off => Some(metadata.severity),
            level => Some(level.into()),
        }
    }

    /// Passes the configured rule options to the analyser.
    pub fn push_to_analyser_rules(
        &self,
        metadata: &MetadataRegistry,
        analyser_rules: &mut LinterRules,
    ) {
        for rule in LINTER_RULES {
            let Some(options) = self.rule_options(rule.name) else {
                continue;
            };
            if let Some(key) = metadata.find_rule(rule.group, rule.name) {
                analyser_rules.push_rule(key, options);
            }
        }
    }

    /// A message for each use of deprecated settings.
    pub fn deprecations(&self) -> Vec<String> {
        let Some(legacy) = &self.rules.safety else {
            return Vec::new();
        };

        let mut messages = Vec::new();
        if legacy.recommended.is_some() {
            messages.push(
                "`linter.rules.safety.recommended` is deprecated. Use `linter.rules.recommended` or `linter.groups.<group>` instead."
                    .to_string(),
            );
        }
        if legacy.all.is_some() {
            messages.push(
                "`linter.rules.safety.all` is deprecated. Use `linter.rules.all` or `linter.groups.<group>` instead."
                    .to_string(),
            );
        }
        for rule in legacy.configured_rules() {
            messages.push(match rule {
                "preferBigintOverInt" | "preferBigintOverSmallint" => format!(
                    "`linter.rules.safety.{rule}` was replaced by `linter.rules.preferBigInt` with the options `checkInt` and `checkSmallint`."
                ),
                "concurrentRefreshMatviewLock" => format!(
                    "`linter.rules.safety.{rule}` was removed. `requireConcurrentRefreshMatview` covers it."
                ),
                _ => format!(
                    "`linter.rules.safety.{rule}` is deprecated. Use `linter.rules.{rule}` instead."
                ),
            });
        }
        messages
    }

    /// The level set by a rule, deprecated rule, or group setting.
    fn configured_level(&self, rule: &LinterRuleMetadata) -> Option<RulePlainConfiguration> {
        self.rules
            .rule_level(rule.name)
            .or_else(|| self.legacy_level(rule.name))
            .or_else(|| self.groups.level(rule.group))
    }

    fn legacy_level(&self, rule: &str) -> Option<RulePlainConfiguration> {
        let legacy = self.rules.safety.as_ref()?;
        legacy.rule_level(rule).or_else(|| {
            if rule != "preferBigInt" {
                return None;
            }
            // `preferBigInt` replaces `preferBigintOverInt` and `preferBigintOverSmallint`. It
            // is enabled if either of them is.
            let levels = [
                legacy.rule_level("preferBigintOverInt"),
                legacy.rule_level("preferBigintOverSmallint"),
            ];
            levels
                .iter()
                .flatten()
                .find(|level| **level != RulePlainConfiguration::Off)
                .or_else(|| levels.iter().flatten().next())
                .copied()
        })
    }

    fn preset_level(&self, rule: &LinterRuleMetadata) -> RulePlainConfiguration {
        let legacy = self.rules.safety.as_ref();
        let all = self.rules.all.or(legacy.and_then(|legacy| legacy.all));
        let recommended = self
            .rules
            .recommended
            .or(legacy.and_then(|legacy| legacy.recommended));

        let enabled = rule.group != "nursery"
            && (all == Some(true) || (recommended != Some(false) && rule.recommended));
        if enabled {
            severity_level(rule.severity)
        } else {
            RulePlainConfiguration::Off
        }
    }

    fn rule_options(&self, rule: &str) -> Option<RuleOptions> {
        if let Some(options) = self.rules.rule_options(rule) {
            return Some(options);
        }
        let legacy = self.rules.safety.as_ref()?;
        if let Some(options) = legacy.rule_options(rule) {
            return Some(options);
        }
        if rule != "preferBigInt" || self.rules.rule_level(rule).is_some() {
            return None;
        }

        // Map the removed rules to the options of `preferBigInt`.
        let enabled = |rule| {
            legacy
                .rule_level(rule)
                .map(|level| level != RulePlainConfiguration::Off)
        };
        let check_int = enabled("preferBigintOverInt");
        let check_smallint = enabled("preferBigintOverSmallint");
        if check_int.is_none() && check_smallint.is_none() {
            return None;
        }
        Some(RuleOptions::new(pgls_analyser::options::PreferBigInt {
            check_int: check_int.unwrap_or(false),
            check_smallint: check_smallint.unwrap_or(false),
        }))
    }
}

fn find_rule(rule: &str) -> Option<&'static LinterRuleMetadata> {
    LINTER_RULES
        .binary_search_by(|metadata| metadata.name.cmp(rule))
        .ok()
        .map(|index| &LINTER_RULES[index])
}

fn severity_level(severity: Severity) -> RulePlainConfiguration {
    match severity {
        Severity::Error | Severity::Fatal => RulePlainConfiguration::Error,
        Severity::Warning => RulePlainConfiguration::Warn,
        Severity::Hint | Severity::Information => RulePlainConfiguration::Info,
    }
}

#[cfg(test)]
mod tests {
    use pgls_diagnostics::category;

    use super::*;
    use crate::linter::LegacySafetyRules;
    use crate::rules::RuleConfiguration;

    fn settings(json: serde_json::Value) -> (Rules, Groups) {
        let rules = serde_json::from_value(json["rules"].clone()).unwrap();
        let groups = json
            .get("groups")
            .map(|groups| serde_json::from_value(groups.clone()).unwrap())
            .unwrap_or_default();
        (rules, groups)
    }

    fn enabled(rules: &Rules, groups: &Groups, rule: &str) -> bool {
        LinterRuleSettings::new(rules, groups)
            .as_enabled_rules()
            .iter()
            .any(|filter| matches!(filter, RuleFilter::Rule(_, name) if *name == rule))
    }

    #[test]
    fn recommended_rules_are_enabled_by_default() {
        let (rules, groups) = settings(serde_json::json!({ "rules": {} }));
        assert!(enabled(&rules, &groups, "banDropColumn"));
        assert!(!enabled(&rules, &groups, "preferBigInt"));

        let (rules, groups) = settings(serde_json::json!({ "rules": { "recommended": false } }));
        assert!(!enabled(&rules, &groups, "banDropColumn"));

        let (rules, groups) = settings(serde_json::json!({ "rules": { "all": true } }));
        assert!(enabled(&rules, &groups, "preferBigInt"));
    }

    #[test]
    fn flat_rules_with_options() {
        let (rules, groups) = settings(serde_json::json!({
            "rules": {
                "banDropColumn": "off",
                "preferBigInt": { "level": "warn", "options": { "checkSmallint": false } }
            }
        }));
        let settings = LinterRuleSettings::new(&rules, &groups);
        assert_eq!(
            settings.level("banDropColumn"),
            Some(RulePlainConfiguration::Off)
        );
        assert_eq!(
            settings.level("preferBigInt"),
            Some(RulePlainConfiguration::Warn)
        );
        assert!(
            settings
                .as_disabled_rules()
                .contains(&RuleFilter::Rule("destructive", "banDropColumn"))
        );

        let options = rules
            .prefer_big_int
            .as_ref()
            .unwrap()
            .get_options_ref()
            .unwrap();
        assert!(options.check_int);
        assert!(!options.check_smallint);
    }

    #[test]
    fn groups_apply_unless_a_rule_is_configured() {
        let (rules, groups) = settings(serde_json::json!({
            "rules": { "banTruncate": "error" },
            "groups": { "destructive": "off", "style": "info" }
        }));
        let settings = LinterRuleSettings::new(&rules, &groups);
        assert_eq!(
            settings.level("banDropColumn"),
            Some(RulePlainConfiguration::Off)
        );
        assert_eq!(
            settings.level("banTruncate"),
            Some(RulePlainConfiguration::Error)
        );
        assert_eq!(
            settings.level("preferBigInt"),
            Some(RulePlainConfiguration::Info)
        );
        assert!(
            settings
                .as_disabled_rules()
                .contains(&RuleFilter::Rule("destructive", "banDropColumn"))
        );
    }

    #[test]
    fn severity_from_code() {
        let (rules, groups) = settings(serde_json::json!({
            "rules": { "banDropColumn": "info" }
        }));
        let settings = LinterRuleSettings::new(&rules, &groups);
        assert_eq!(
            settings.get_severity_from_code(category!("lint/banDropColumn")),
            Some(Severity::Information)
        );
        assert_eq!(
            settings.get_severity_from_code(category!("lint/banTruncate")),
            find_rule("banTruncate").map(|rule| rule.severity)
        );
    }

    #[test]
    fn legacy_safety_rules() {
        let (rules, groups) = settings(serde_json::json!({
            "rules": {
                "banTruncate": "warn",
                "safety": {
                    "banDropColumn": "off",
                    "banTruncate": "off",
                    "preferBigintOverSmallint": "error",
                    "concurrentRefreshMatviewLock": "warn"
                }
            }
        }));
        let settings = LinterRuleSettings::new(&rules, &groups);
        assert_eq!(
            settings.level("banDropColumn"),
            Some(RulePlainConfiguration::Off)
        );
        // Flat settings take precedence.
        assert_eq!(
            settings.level("banTruncate"),
            Some(RulePlainConfiguration::Warn)
        );
        assert_eq!(
            settings.level("preferBigInt"),
            Some(RulePlainConfiguration::Error)
        );

        let options = settings.rule_options("preferBigInt").unwrap();
        let options = options.value::<pgls_analyser::options::PreferBigInt>();
        assert!(!options.check_int);
        assert!(options.check_smallint);

        let deprecations = settings.deprecations();
        assert_eq!(deprecations.len(), 4, "{deprecations:?}");
        assert!(
            deprecations
                .iter()
                .any(|message| message.contains("preferBigInt"))
        );
    }

    #[test]
    fn legacy_presets() {
        let rules = Rules {
            safety: Some(LegacySafetyRules {
                recommended: Some(false),
                ban_truncate: Some(RuleConfiguration::Plain(RulePlainConfiguration::Warn)),
                ..Default::default()
            }),
            ..Default::default()
        };
        let groups = Groups::default();
        assert!(!enabled(&rules, &groups, "banDropColumn"));
        assert!(enabled(&rules, &groups, "banTruncate"));
    }
}
