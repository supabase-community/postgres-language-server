//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
use crate::rules::{RuleConfiguration, RulePlainConfiguration};
use pgls_analyse::RuleFilter;
use pgls_analyser::RuleOptions;
use pgls_configuration_macros::Merge;
use pgls_diagnostics::{Category, Severity};
use rustc_hash::FxHashSet;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Merge,
    Ord,
    PartialEq,
    PartialOrd,
    serde :: Deserialize,
    serde :: Serialize,
)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum RuleGroup {
    Performance,
    Security,
}
impl RuleGroup {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Performance => Performance::GROUP_NAME,
            Self::Security => Security::GROUP_NAME,
        }
    }
}
impl std::str::FromStr for RuleGroup {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            Performance::GROUP_NAME => Ok(Self::Performance),
            Security::GROUP_NAME => Ok(Self::Security),
            _ => Err("This rule group doesn't exist."),
        }
    }
}
#[derive(Clone, Debug, Default, Deserialize, Eq, Merge, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "SplinterRules"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Rules {
    #[doc = r" It enables the lint rules recommended by Postgres Language Server. `true` by default."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommended: Option<bool>,
    #[doc = r" It enables ALL rules. The rules that belong to `nursery` won't be enabled."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance: Option<Performance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<Security>,
}
impl Rules {
    #[doc = r" Checks if the code coming from [pgls_diagnostics::Diagnostic] corresponds to a rule."]
    #[doc = r" Usually the code is built like {group}/{rule_name}"]
    pub fn has_rule(group: RuleGroup, rule_name: &str) -> Option<&'static str> {
        match group {
            RuleGroup::Performance => Performance::has_rule(rule_name),
            RuleGroup::Security => Security::has_rule(rule_name),
        }
    }
    #[doc = r" Given a category coming from [Diagnostic](pgls_diagnostics::Diagnostic), this function returns"]
    #[doc = r" the [Severity](pgls_diagnostics::Severity) associated to the rule, if the configuration changed it."]
    #[doc = r" If the severity is off or not set, then the function returns the default severity of the rule,"]
    #[doc = r" which is configured at the rule definition."]
    #[doc = r" The function can return `None` if the rule is not properly configured."]
    pub fn get_severity_from_code(&self, category: &Category) -> Option<Severity> {
        let mut split_code = category.name().split('/');
        let _category_prefix = split_code.next();
        debug_assert_eq!(_category_prefix, Some("splinter"));
        let group = <RuleGroup as std::str::FromStr>::from_str(split_code.next()?).ok()?;
        let rule_name = split_code.next()?;
        let rule_name = Self::has_rule(group, rule_name)?;
        let severity = match group {
            RuleGroup::Performance => self
                .performance
                .as_ref()
                .and_then(|group| group.get_rule_configuration(rule_name))
                .filter(|(level, _)| !matches!(level, RulePlainConfiguration::Off))
                .map_or_else(
                    || Performance::severity(rule_name),
                    |(level, _)| level.into(),
                ),
            RuleGroup::Security => self
                .security
                .as_ref()
                .and_then(|group| group.get_rule_configuration(rule_name))
                .filter(|(level, _)| !matches!(level, RulePlainConfiguration::Off))
                .map_or_else(|| Security::severity(rule_name), |(level, _)| level.into()),
        };
        Some(severity)
    }
    #[doc = r" Ensure that `recommended` is set to `true` or implied."]
    pub fn set_recommended(&mut self) {
        if self.all != Some(true) && self.recommended == Some(false) {
            self.recommended = Some(true)
        }
        if let Some(group) = &mut self.performance {
            group.recommended = None;
        }
        if let Some(group) = &mut self.security {
            group.recommended = None;
        }
    }
    pub(crate) const fn is_recommended_false(&self) -> bool {
        matches!(self.recommended, Some(false))
    }
    pub(crate) const fn is_all_true(&self) -> bool {
        matches!(self.all, Some(true))
    }
    #[doc = r" It returns the enabled rules by default."]
    #[doc = r""]
    #[doc = r" The enabled rules are calculated from the difference with the disabled rules."]
    pub fn as_enabled_rules(&self) -> FxHashSet<RuleFilter<'static>> {
        let mut enabled_rules = FxHashSet::default();
        let mut disabled_rules = FxHashSet::default();
        if let Some(group) = self.performance.as_ref() {
            group.collect_preset_rules(
                self.is_all_true(),
                !self.is_recommended_false(),
                &mut enabled_rules,
            );
            enabled_rules.extend(&group.get_enabled_rules());
            disabled_rules.extend(&group.get_disabled_rules());
        } else if self.is_all_true() {
            enabled_rules.extend(Performance::all_rules_as_filters());
        } else if !self.is_recommended_false() {
            enabled_rules.extend(Performance::recommended_rules_as_filters());
        }
        if let Some(group) = self.security.as_ref() {
            group.collect_preset_rules(
                self.is_all_true(),
                !self.is_recommended_false(),
                &mut enabled_rules,
            );
            enabled_rules.extend(&group.get_enabled_rules());
            disabled_rules.extend(&group.get_disabled_rules());
        } else if self.is_all_true() {
            enabled_rules.extend(Security::all_rules_as_filters());
        } else if !self.is_recommended_false() {
            enabled_rules.extend(Security::recommended_rules_as_filters());
        }
        enabled_rules.difference(&disabled_rules).copied().collect()
    }
    #[doc = r" It returns the disabled rules by configuration."]
    pub fn as_disabled_rules(&self) -> FxHashSet<RuleFilter<'static>> {
        let mut disabled_rules = FxHashSet::default();
        if let Some(group) = self.performance.as_ref() {
            disabled_rules.extend(&group.get_disabled_rules());
        }
        if let Some(group) = self.security.as_ref() {
            disabled_rules.extend(&group.get_disabled_rules());
        }
        disabled_rules
    }
    #[doc = r" Build matchers for all rules that have ignore patterns configured."]
    #[doc = r" Returns a map from rule name (camelCase) to the matcher."]
    pub fn get_ignore_matchers(
        &self,
    ) -> rustc_hash::FxHashMap<&'static str, pgls_matcher::Matcher> {
        let mut matchers = rustc_hash::FxHashMap::default();
        if let Some(group) = &self.performance {
            matchers.extend(group.get_ignore_matchers());
        }
        if let Some(group) = &self.security {
            matchers.extend(group.get_ignore_matchers());
        }
        matchers
    }
}
#[derive(Clone, Debug, Default, Deserialize, Eq, Merge, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
#[doc = r" A list of rules that belong to this group"]
pub struct Performance {
    #[doc = r" It enables the recommended rules for this group"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommended: Option<bool>,
    #[doc = r" It enables ALL rules for this group."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all: Option<bool>,
    #[doc = "Auth RLS Initialization Plan: Detects if calls to `current_setting()` and `auth.()` in RLS policies are being unnecessarily re-evaluated for each row"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_rls_initplan: Option<RuleConfiguration<crate::splinter::SplinterRuleOptions>>,
    #[doc = "Duplicate Index: Detects cases where two ore more identical indexes exist."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duplicate_index: Option<RuleConfiguration<crate::splinter::SplinterRuleOptions>>,
    #[doc = "Multiple Permissive Policies: Detects if multiple permissive row level security policies are present on a table for the same `role` and `action` (e.g. insert). Multiple permissive policies are suboptimal for performance as each policy must be executed for every relevant query."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiple_permissive_policies:
        Option<RuleConfiguration<crate::splinter::SplinterRuleOptions>>,
    #[doc = "No Primary Key: Detects if a table does not have a primary key. Tables without a primary key can be inefficient to interact with at scale."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_primary_key: Option<RuleConfiguration<crate::splinter::SplinterRuleOptions>>,
    #[doc = "Table Bloat: Detects if a table has excess bloat and may benefit from maintenance operations like vacuum full or cluster."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_bloat: Option<RuleConfiguration<crate::splinter::SplinterRuleOptions>>,
    #[doc = "Unindexed foreign keys: Identifies foreign key constraints without a covering index, which can impact database performance."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unindexed_foreign_keys: Option<RuleConfiguration<crate::splinter::SplinterRuleOptions>>,
    #[doc = "Unused Index: Detects if an index has never been used and may be a candidate for removal."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unused_index: Option<RuleConfiguration<crate::splinter::SplinterRuleOptions>>,
}
impl Performance {
    const GROUP_NAME: &'static str = "performance";
    pub(crate) const GROUP_RULES: &'static [&'static str] = &[
        "authRlsInitplan",
        "duplicateIndex",
        "multiplePermissivePolicies",
        "noPrimaryKey",
        "tableBloat",
        "unindexedForeignKeys",
        "unusedIndex",
    ];
    const RECOMMENDED_RULES_AS_FILTERS: &'static [RuleFilter<'static>] = &[
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[0]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[1]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[2]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[3]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[4]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[5]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[6]),
    ];
    const ALL_RULES_AS_FILTERS: &'static [RuleFilter<'static>] = &[
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[0]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[1]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[2]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[3]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[4]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[5]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[6]),
    ];
    #[doc = r" Retrieves the recommended rules"]
    pub(crate) fn is_recommended_true(&self) -> bool {
        matches!(self.recommended, Some(true))
    }
    pub(crate) fn is_recommended_unset(&self) -> bool {
        self.recommended.is_none()
    }
    pub(crate) fn is_all_true(&self) -> bool {
        matches!(self.all, Some(true))
    }
    pub(crate) fn is_all_unset(&self) -> bool {
        self.all.is_none()
    }
    pub(crate) fn get_enabled_rules(&self) -> FxHashSet<RuleFilter<'static>> {
        let mut index_set = FxHashSet::default();
        if let Some(rule) = self.auth_rls_initplan.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[0]));
            }
        }
        if let Some(rule) = self.duplicate_index.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[1]));
            }
        }
        if let Some(rule) = self.multiple_permissive_policies.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[2]));
            }
        }
        if let Some(rule) = self.no_primary_key.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[3]));
            }
        }
        if let Some(rule) = self.table_bloat.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[4]));
            }
        }
        if let Some(rule) = self.unindexed_foreign_keys.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[5]));
            }
        }
        if let Some(rule) = self.unused_index.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[6]));
            }
        }
        index_set
    }
    pub(crate) fn get_disabled_rules(&self) -> FxHashSet<RuleFilter<'static>> {
        let mut index_set = FxHashSet::default();
        if let Some(rule) = self.auth_rls_initplan.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[0]));
            }
        }
        if let Some(rule) = self.duplicate_index.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[1]));
            }
        }
        if let Some(rule) = self.multiple_permissive_policies.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[2]));
            }
        }
        if let Some(rule) = self.no_primary_key.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[3]));
            }
        }
        if let Some(rule) = self.table_bloat.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[4]));
            }
        }
        if let Some(rule) = self.unindexed_foreign_keys.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[5]));
            }
        }
        if let Some(rule) = self.unused_index.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[6]));
            }
        }
        index_set
    }
    #[doc = r" Checks if, given a rule name, matches one of the rules contained in this category"]
    pub(crate) fn has_rule(rule_name: &str) -> Option<&'static str> {
        Some(Self::GROUP_RULES[Self::GROUP_RULES.binary_search(&rule_name).ok()?])
    }
    pub(crate) fn recommended_rules_as_filters() -> &'static [RuleFilter<'static>] {
        Self::RECOMMENDED_RULES_AS_FILTERS
    }
    pub(crate) fn all_rules_as_filters() -> &'static [RuleFilter<'static>] {
        Self::ALL_RULES_AS_FILTERS
    }
    #[doc = r" Select preset rules"]
    pub(crate) fn collect_preset_rules(
        &self,
        parent_is_all: bool,
        parent_is_recommended: bool,
        enabled_rules: &mut FxHashSet<RuleFilter<'static>>,
    ) {
        if self.is_all_true() || self.is_all_unset() && parent_is_all {
            enabled_rules.extend(Self::all_rules_as_filters());
        } else if self.is_recommended_true()
            || self.is_recommended_unset() && self.is_all_unset() && parent_is_recommended
        {
            enabled_rules.extend(Self::recommended_rules_as_filters());
        }
    }
    pub(crate) fn severity(rule_name: &str) -> Severity {
        match rule_name {
            "authRlsInitplan" => Severity::Warning,
            "duplicateIndex" => Severity::Warning,
            "multiplePermissivePolicies" => Severity::Warning,
            "noPrimaryKey" => Severity::Information,
            "tableBloat" => Severity::Information,
            "unindexedForeignKeys" => Severity::Information,
            "unusedIndex" => Severity::Information,
            _ => unreachable!(),
        }
    }
    pub(crate) fn get_rule_configuration(
        &self,
        rule_name: &str,
    ) -> Option<(RulePlainConfiguration, Option<RuleOptions>)> {
        match rule_name {
            "authRlsInitplan" => self
                .auth_rls_initplan
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "duplicateIndex" => self
                .duplicate_index
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "multiplePermissivePolicies" => self
                .multiple_permissive_policies
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "noPrimaryKey" => self
                .no_primary_key
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "tableBloat" => self
                .table_bloat
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "unindexedForeignKeys" => self
                .unindexed_foreign_keys
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "unusedIndex" => self
                .unused_index
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            _ => None,
        }
    }
    #[doc = r" Build matchers for rules in this group that have ignore patterns configured"]
    pub fn get_ignore_matchers(
        &self,
    ) -> rustc_hash::FxHashMap<&'static str, pgls_matcher::Matcher> {
        let mut matchers = rustc_hash::FxHashMap::default();
        if let Some(conf) = &self.auth_rls_initplan {
            if let Some(options) = conf.get_options_ref() {
                if !options.ignore.is_empty() {
                    let mut m = pgls_matcher::Matcher::new(pgls_matcher::MatchOptions::default());
                    for p in &options.ignore {
                        let _ = m.add_pattern(p);
                    }
                    matchers.insert("authRlsInitplan", m);
                }
            }
        }
        if let Some(conf) = &self.duplicate_index {
            if let Some(options) = conf.get_options_ref() {
                if !options.ignore.is_empty() {
                    let mut m = pgls_matcher::Matcher::new(pgls_matcher::MatchOptions::default());
                    for p in &options.ignore {
                        let _ = m.add_pattern(p);
                    }
                    matchers.insert("duplicateIndex", m);
                }
            }
        }
        if let Some(conf) = &self.multiple_permissive_policies {
            if let Some(options) = conf.get_options_ref() {
                if !options.ignore.is_empty() {
                    let mut m = pgls_matcher::Matcher::new(pgls_matcher::MatchOptions::default());
                    for p in &options.ignore {
                        let _ = m.add_pattern(p);
                    }
                    matchers.insert("multiplePermissivePolicies", m);
                }
            }
        }
        if let Some(conf) = &self.no_primary_key {
            if let Some(options) = conf.get_options_ref() {
                if !options.ignore.is_empty() {
                    let mut m = pgls_matcher::Matcher::new(pgls_matcher::MatchOptions::default());
                    for p in &options.ignore {
                        let _ = m.add_pattern(p);
                    }
                    matchers.insert("noPrimaryKey", m);
                }
            }
        }
        if let Some(conf) = &self.table_bloat {
            if let Some(options) = conf.get_options_ref() {
                if !options.ignore.is_empty() {
                    let mut m = pgls_matcher::Matcher::new(pgls_matcher::MatchOptions::default());
                    for p in &options.ignore {
                        let _ = m.add_pattern(p);
                    }
                    matchers.insert("tableBloat", m);
                }
            }
        }
        if let Some(conf) = &self.unindexed_foreign_keys {
            if let Some(options) = conf.get_options_ref() {
                if !options.ignore.is_empty() {
                    let mut m = pgls_matcher::Matcher::new(pgls_matcher::MatchOptions::default());
                    for p in &options.ignore {
                        let _ = m.add_pattern(p);
                    }
                    matchers.insert("unindexedForeignKeys", m);
                }
            }
        }
        if let Some(conf) = &self.unused_index {
            if let Some(options) = conf.get_options_ref() {
                if !options.ignore.is_empty() {
                    let mut m = pgls_matcher::Matcher::new(pgls_matcher::MatchOptions::default());
                    for p in &options.ignore {
                        let _ = m.add_pattern(p);
                    }
                    matchers.insert("unusedIndex", m);
                }
            }
        }
        matchers
    }
}
#[derive(Clone, Debug, Default, Deserialize, Eq, Merge, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
#[doc = r" A list of rules that belong to this group"]
pub struct Security {
    #[doc = r" It enables the recommended rules for this group"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommended: Option<bool>,
    #[doc = r" It enables ALL rules for this group."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all: Option<bool>,
    #[doc = "Exposed Auth Users: Detects if auth.users is exposed to anon or authenticated roles via a view or materialized view in schemas exposed to PostgREST, potentially compromising user data security."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_users_exposed: Option<RuleConfiguration<crate::splinter::SplinterRuleOptions>>,
    #[doc = "Extension in Public: Detects extensions installed in the `public` schema."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extension_in_public: Option<RuleConfiguration<crate::splinter::SplinterRuleOptions>>,
    #[doc = "Extension Versions Outdated: Detects extensions that are not using the default (recommended) version."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extension_versions_outdated:
        Option<RuleConfiguration<crate::splinter::SplinterRuleOptions>>,
    #[doc = "Foreign Key to Auth Unique Constraint: Detects user defined foreign keys to unique constraints in the auth schema."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fkey_to_auth_unique: Option<RuleConfiguration<crate::splinter::SplinterRuleOptions>>,
    #[doc = "Foreign Table in API: Detects foreign tables that are accessible over APIs. Foreign tables do not respect row level security policies."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub foreign_table_in_api: Option<RuleConfiguration<crate::splinter::SplinterRuleOptions>>,
    #[doc = "Function Search Path Mutable: Detects functions where the search_path parameter is not set."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_search_path_mutable:
        Option<RuleConfiguration<crate::splinter::SplinterRuleOptions>>,
    #[doc = "Insecure Queue Exposed in API: Detects cases where an insecure Queue is exposed over Data APIs"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insecure_queue_exposed_in_api:
        Option<RuleConfiguration<crate::splinter::SplinterRuleOptions>>,
    #[doc = "Materialized View in API: Detects materialized views that are accessible over the Data APIs."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub materialized_view_in_api: Option<RuleConfiguration<crate::splinter::SplinterRuleOptions>>,
    #[doc = "Policy Exists RLS Disabled: Detects cases where row level security (RLS) policies have been created, but RLS has not been enabled for the underlying table."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_exists_rls_disabled: Option<RuleConfiguration<crate::splinter::SplinterRuleOptions>>,
    #[doc = "RLS Disabled in Public: Detects cases where row level security (RLS) has not been enabled on tables in schemas exposed to PostgREST"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rls_disabled_in_public: Option<RuleConfiguration<crate::splinter::SplinterRuleOptions>>,
    #[doc = "RLS Enabled No Policy: Detects cases where row level security (RLS) has been enabled on a table but no RLS policies have been created."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rls_enabled_no_policy: Option<RuleConfiguration<crate::splinter::SplinterRuleOptions>>,
    #[doc = "RLS references user metadata: Detects when Supabase Auth user_metadata is referenced insecurely in a row level security (RLS) policy."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rls_references_user_metadata:
        Option<RuleConfiguration<crate::splinter::SplinterRuleOptions>>,
    #[doc = "Security Definer View: Detects views defined with the SECURITY DEFINER property. These views enforce Postgres permissions and row level security policies (RLS) of the view creator, rather than that of the querying user"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_definer_view: Option<RuleConfiguration<crate::splinter::SplinterRuleOptions>>,
    #[doc = "Unsupported reg types: Identifies columns using unsupported reg* types outside pg_catalog schema, which prevents database upgrades using pg_upgrade."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unsupported_reg_types: Option<RuleConfiguration<crate::splinter::SplinterRuleOptions>>,
}
impl Security {
    const GROUP_NAME: &'static str = "security";
    pub(crate) const GROUP_RULES: &'static [&'static str] = &[
        "authUsersExposed",
        "extensionInPublic",
        "extensionVersionsOutdated",
        "fkeyToAuthUnique",
        "foreignTableInApi",
        "functionSearchPathMutable",
        "insecureQueueExposedInApi",
        "materializedViewInApi",
        "policyExistsRlsDisabled",
        "rlsDisabledInPublic",
        "rlsEnabledNoPolicy",
        "rlsReferencesUserMetadata",
        "securityDefinerView",
        "unsupportedRegTypes",
    ];
    const RECOMMENDED_RULES_AS_FILTERS: &'static [RuleFilter<'static>] = &[
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[0]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[1]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[2]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[3]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[4]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[5]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[6]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[7]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[8]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[9]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[10]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[11]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[12]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[13]),
    ];
    const ALL_RULES_AS_FILTERS: &'static [RuleFilter<'static>] = &[
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[0]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[1]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[2]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[3]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[4]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[5]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[6]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[7]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[8]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[9]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[10]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[11]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[12]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[13]),
    ];
    #[doc = r" Retrieves the recommended rules"]
    pub(crate) fn is_recommended_true(&self) -> bool {
        matches!(self.recommended, Some(true))
    }
    pub(crate) fn is_recommended_unset(&self) -> bool {
        self.recommended.is_none()
    }
    pub(crate) fn is_all_true(&self) -> bool {
        matches!(self.all, Some(true))
    }
    pub(crate) fn is_all_unset(&self) -> bool {
        self.all.is_none()
    }
    pub(crate) fn get_enabled_rules(&self) -> FxHashSet<RuleFilter<'static>> {
        let mut index_set = FxHashSet::default();
        if let Some(rule) = self.auth_users_exposed.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[0]));
            }
        }
        if let Some(rule) = self.extension_in_public.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[1]));
            }
        }
        if let Some(rule) = self.extension_versions_outdated.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[2]));
            }
        }
        if let Some(rule) = self.fkey_to_auth_unique.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[3]));
            }
        }
        if let Some(rule) = self.foreign_table_in_api.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[4]));
            }
        }
        if let Some(rule) = self.function_search_path_mutable.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[5]));
            }
        }
        if let Some(rule) = self.insecure_queue_exposed_in_api.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[6]));
            }
        }
        if let Some(rule) = self.materialized_view_in_api.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[7]));
            }
        }
        if let Some(rule) = self.policy_exists_rls_disabled.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[8]));
            }
        }
        if let Some(rule) = self.rls_disabled_in_public.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[9]));
            }
        }
        if let Some(rule) = self.rls_enabled_no_policy.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[10]));
            }
        }
        if let Some(rule) = self.rls_references_user_metadata.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[11]));
            }
        }
        if let Some(rule) = self.security_definer_view.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[12]));
            }
        }
        if let Some(rule) = self.unsupported_reg_types.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[13]));
            }
        }
        index_set
    }
    pub(crate) fn get_disabled_rules(&self) -> FxHashSet<RuleFilter<'static>> {
        let mut index_set = FxHashSet::default();
        if let Some(rule) = self.auth_users_exposed.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[0]));
            }
        }
        if let Some(rule) = self.extension_in_public.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[1]));
            }
        }
        if let Some(rule) = self.extension_versions_outdated.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[2]));
            }
        }
        if let Some(rule) = self.fkey_to_auth_unique.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[3]));
            }
        }
        if let Some(rule) = self.foreign_table_in_api.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[4]));
            }
        }
        if let Some(rule) = self.function_search_path_mutable.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[5]));
            }
        }
        if let Some(rule) = self.insecure_queue_exposed_in_api.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[6]));
            }
        }
        if let Some(rule) = self.materialized_view_in_api.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[7]));
            }
        }
        if let Some(rule) = self.policy_exists_rls_disabled.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[8]));
            }
        }
        if let Some(rule) = self.rls_disabled_in_public.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[9]));
            }
        }
        if let Some(rule) = self.rls_enabled_no_policy.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[10]));
            }
        }
        if let Some(rule) = self.rls_references_user_metadata.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[11]));
            }
        }
        if let Some(rule) = self.security_definer_view.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[12]));
            }
        }
        if let Some(rule) = self.unsupported_reg_types.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[13]));
            }
        }
        index_set
    }
    #[doc = r" Checks if, given a rule name, matches one of the rules contained in this category"]
    pub(crate) fn has_rule(rule_name: &str) -> Option<&'static str> {
        Some(Self::GROUP_RULES[Self::GROUP_RULES.binary_search(&rule_name).ok()?])
    }
    pub(crate) fn recommended_rules_as_filters() -> &'static [RuleFilter<'static>] {
        Self::RECOMMENDED_RULES_AS_FILTERS
    }
    pub(crate) fn all_rules_as_filters() -> &'static [RuleFilter<'static>] {
        Self::ALL_RULES_AS_FILTERS
    }
    #[doc = r" Select preset rules"]
    pub(crate) fn collect_preset_rules(
        &self,
        parent_is_all: bool,
        parent_is_recommended: bool,
        enabled_rules: &mut FxHashSet<RuleFilter<'static>>,
    ) {
        if self.is_all_true() || self.is_all_unset() && parent_is_all {
            enabled_rules.extend(Self::all_rules_as_filters());
        } else if self.is_recommended_true()
            || self.is_recommended_unset() && self.is_all_unset() && parent_is_recommended
        {
            enabled_rules.extend(Self::recommended_rules_as_filters());
        }
    }
    pub(crate) fn severity(rule_name: &str) -> Severity {
        match rule_name {
            "authUsersExposed" => Severity::Error,
            "extensionInPublic" => Severity::Warning,
            "extensionVersionsOutdated" => Severity::Warning,
            "fkeyToAuthUnique" => Severity::Error,
            "foreignTableInApi" => Severity::Warning,
            "functionSearchPathMutable" => Severity::Warning,
            "insecureQueueExposedInApi" => Severity::Error,
            "materializedViewInApi" => Severity::Warning,
            "policyExistsRlsDisabled" => Severity::Error,
            "rlsDisabledInPublic" => Severity::Error,
            "rlsEnabledNoPolicy" => Severity::Information,
            "rlsReferencesUserMetadata" => Severity::Error,
            "securityDefinerView" => Severity::Error,
            "unsupportedRegTypes" => Severity::Warning,
            _ => unreachable!(),
        }
    }
    pub(crate) fn get_rule_configuration(
        &self,
        rule_name: &str,
    ) -> Option<(RulePlainConfiguration, Option<RuleOptions>)> {
        match rule_name {
            "authUsersExposed" => self
                .auth_users_exposed
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "extensionInPublic" => self
                .extension_in_public
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "extensionVersionsOutdated" => self
                .extension_versions_outdated
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "fkeyToAuthUnique" => self
                .fkey_to_auth_unique
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "foreignTableInApi" => self
                .foreign_table_in_api
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "functionSearchPathMutable" => self
                .function_search_path_mutable
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "insecureQueueExposedInApi" => self
                .insecure_queue_exposed_in_api
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "materializedViewInApi" => self
                .materialized_view_in_api
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "policyExistsRlsDisabled" => self
                .policy_exists_rls_disabled
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "rlsDisabledInPublic" => self
                .rls_disabled_in_public
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "rlsEnabledNoPolicy" => self
                .rls_enabled_no_policy
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "rlsReferencesUserMetadata" => self
                .rls_references_user_metadata
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "securityDefinerView" => self
                .security_definer_view
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "unsupportedRegTypes" => self
                .unsupported_reg_types
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            _ => None,
        }
    }
    #[doc = r" Build matchers for rules in this group that have ignore patterns configured"]
    pub fn get_ignore_matchers(
        &self,
    ) -> rustc_hash::FxHashMap<&'static str, pgls_matcher::Matcher> {
        let mut matchers = rustc_hash::FxHashMap::default();
        if let Some(conf) = &self.auth_users_exposed {
            if let Some(options) = conf.get_options_ref() {
                if !options.ignore.is_empty() {
                    let mut m = pgls_matcher::Matcher::new(pgls_matcher::MatchOptions::default());
                    for p in &options.ignore {
                        let _ = m.add_pattern(p);
                    }
                    matchers.insert("authUsersExposed", m);
                }
            }
        }
        if let Some(conf) = &self.extension_in_public {
            if let Some(options) = conf.get_options_ref() {
                if !options.ignore.is_empty() {
                    let mut m = pgls_matcher::Matcher::new(pgls_matcher::MatchOptions::default());
                    for p in &options.ignore {
                        let _ = m.add_pattern(p);
                    }
                    matchers.insert("extensionInPublic", m);
                }
            }
        }
        if let Some(conf) = &self.extension_versions_outdated {
            if let Some(options) = conf.get_options_ref() {
                if !options.ignore.is_empty() {
                    let mut m = pgls_matcher::Matcher::new(pgls_matcher::MatchOptions::default());
                    for p in &options.ignore {
                        let _ = m.add_pattern(p);
                    }
                    matchers.insert("extensionVersionsOutdated", m);
                }
            }
        }
        if let Some(conf) = &self.fkey_to_auth_unique {
            if let Some(options) = conf.get_options_ref() {
                if !options.ignore.is_empty() {
                    let mut m = pgls_matcher::Matcher::new(pgls_matcher::MatchOptions::default());
                    for p in &options.ignore {
                        let _ = m.add_pattern(p);
                    }
                    matchers.insert("fkeyToAuthUnique", m);
                }
            }
        }
        if let Some(conf) = &self.foreign_table_in_api {
            if let Some(options) = conf.get_options_ref() {
                if !options.ignore.is_empty() {
                    let mut m = pgls_matcher::Matcher::new(pgls_matcher::MatchOptions::default());
                    for p in &options.ignore {
                        let _ = m.add_pattern(p);
                    }
                    matchers.insert("foreignTableInApi", m);
                }
            }
        }
        if let Some(conf) = &self.function_search_path_mutable {
            if let Some(options) = conf.get_options_ref() {
                if !options.ignore.is_empty() {
                    let mut m = pgls_matcher::Matcher::new(pgls_matcher::MatchOptions::default());
                    for p in &options.ignore {
                        let _ = m.add_pattern(p);
                    }
                    matchers.insert("functionSearchPathMutable", m);
                }
            }
        }
        if let Some(conf) = &self.insecure_queue_exposed_in_api {
            if let Some(options) = conf.get_options_ref() {
                if !options.ignore.is_empty() {
                    let mut m = pgls_matcher::Matcher::new(pgls_matcher::MatchOptions::default());
                    for p in &options.ignore {
                        let _ = m.add_pattern(p);
                    }
                    matchers.insert("insecureQueueExposedInApi", m);
                }
            }
        }
        if let Some(conf) = &self.materialized_view_in_api {
            if let Some(options) = conf.get_options_ref() {
                if !options.ignore.is_empty() {
                    let mut m = pgls_matcher::Matcher::new(pgls_matcher::MatchOptions::default());
                    for p in &options.ignore {
                        let _ = m.add_pattern(p);
                    }
                    matchers.insert("materializedViewInApi", m);
                }
            }
        }
        if let Some(conf) = &self.policy_exists_rls_disabled {
            if let Some(options) = conf.get_options_ref() {
                if !options.ignore.is_empty() {
                    let mut m = pgls_matcher::Matcher::new(pgls_matcher::MatchOptions::default());
                    for p in &options.ignore {
                        let _ = m.add_pattern(p);
                    }
                    matchers.insert("policyExistsRlsDisabled", m);
                }
            }
        }
        if let Some(conf) = &self.rls_disabled_in_public {
            if let Some(options) = conf.get_options_ref() {
                if !options.ignore.is_empty() {
                    let mut m = pgls_matcher::Matcher::new(pgls_matcher::MatchOptions::default());
                    for p in &options.ignore {
                        let _ = m.add_pattern(p);
                    }
                    matchers.insert("rlsDisabledInPublic", m);
                }
            }
        }
        if let Some(conf) = &self.rls_enabled_no_policy {
            if let Some(options) = conf.get_options_ref() {
                if !options.ignore.is_empty() {
                    let mut m = pgls_matcher::Matcher::new(pgls_matcher::MatchOptions::default());
                    for p in &options.ignore {
                        let _ = m.add_pattern(p);
                    }
                    matchers.insert("rlsEnabledNoPolicy", m);
                }
            }
        }
        if let Some(conf) = &self.rls_references_user_metadata {
            if let Some(options) = conf.get_options_ref() {
                if !options.ignore.is_empty() {
                    let mut m = pgls_matcher::Matcher::new(pgls_matcher::MatchOptions::default());
                    for p in &options.ignore {
                        let _ = m.add_pattern(p);
                    }
                    matchers.insert("rlsReferencesUserMetadata", m);
                }
            }
        }
        if let Some(conf) = &self.security_definer_view {
            if let Some(options) = conf.get_options_ref() {
                if !options.ignore.is_empty() {
                    let mut m = pgls_matcher::Matcher::new(pgls_matcher::MatchOptions::default());
                    for p in &options.ignore {
                        let _ = m.add_pattern(p);
                    }
                    matchers.insert("securityDefinerView", m);
                }
            }
        }
        if let Some(conf) = &self.unsupported_reg_types {
            if let Some(options) = conf.get_options_ref() {
                if !options.ignore.is_empty() {
                    let mut m = pgls_matcher::Matcher::new(pgls_matcher::MatchOptions::default());
                    for p in &options.ignore {
                        let _ = m.add_pattern(p);
                    }
                    matchers.insert("unsupportedRegTypes", m);
                }
            }
        }
        matchers
    }
}
#[doc = r" Push the configured rules to the analyser"]
pub fn push_to_analyser_rules(
    rules: &Rules,
    metadata: &pgls_analyse::MetadataRegistry,
    analyser_rules: &mut pgls_analyser::LinterRules,
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
#[test]
fn test_order() {
    for items in Performance::GROUP_RULES.windows(2) {
        assert!(items[0] < items[1], "{} < {}", items[0], items[1]);
    }
    for items in Security::GROUP_RULES.windows(2) {
        assert!(items[0] < items[1], "{} < {}", items[0], items[1]);
    }
}
