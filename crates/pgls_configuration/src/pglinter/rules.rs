//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
use crate::rules::{RuleConfiguration, RulePlainConfiguration};
use biome_deserialize_macros::Merge;
use pgls_analyse::RuleFilter;
use pgls_analyser::RuleOptions;
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
    Base,
    Cluster,
    Schema,
}
impl RuleGroup {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Base => Base::GROUP_NAME,
            Self::Cluster => Cluster::GROUP_NAME,
            Self::Schema => Schema::GROUP_NAME,
        }
    }
}
impl std::str::FromStr for RuleGroup {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            Base::GROUP_NAME => Ok(Self::Base),
            Cluster::GROUP_NAME => Ok(Self::Cluster),
            Schema::GROUP_NAME => Ok(Self::Schema),
            _ => Err("This rule group doesn't exist."),
        }
    }
}
#[derive(Clone, Debug, Default, Deserialize, Eq, Merge, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "PglinterRules"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Rules {
    #[doc = r" It enables the lint rules recommended by Postgres Language Server. `true` by default."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommended: Option<bool>,
    #[doc = r" It enables ALL rules. The rules that belong to `nursery` won't be enabled."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base: Option<Base>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster: Option<Cluster>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<Schema>,
}
impl Rules {
    #[doc = r" Checks if the code coming from [pgls_diagnostics::Diagnostic] corresponds to a rule."]
    #[doc = r" Usually the code is built like {group}/{rule_name}"]
    pub fn has_rule(group: RuleGroup, rule_name: &str) -> Option<&'static str> {
        match group {
            RuleGroup::Base => Base::has_rule(rule_name),
            RuleGroup::Cluster => Cluster::has_rule(rule_name),
            RuleGroup::Schema => Schema::has_rule(rule_name),
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
        debug_assert_eq!(_category_prefix, Some("pglinter"));
        let group = <RuleGroup as std::str::FromStr>::from_str(split_code.next()?).ok()?;
        let rule_name = split_code.next()?;
        let rule_name = Self::has_rule(group, rule_name)?;
        let severity = match group {
            RuleGroup::Base => self
                .base
                .as_ref()
                .and_then(|group| group.get_rule_configuration(rule_name))
                .filter(|(level, _)| !matches!(level, RulePlainConfiguration::Off))
                .map_or_else(|| Base::severity(rule_name), |(level, _)| level.into()),
            RuleGroup::Cluster => self
                .cluster
                .as_ref()
                .and_then(|group| group.get_rule_configuration(rule_name))
                .filter(|(level, _)| !matches!(level, RulePlainConfiguration::Off))
                .map_or_else(|| Cluster::severity(rule_name), |(level, _)| level.into()),
            RuleGroup::Schema => self
                .schema
                .as_ref()
                .and_then(|group| group.get_rule_configuration(rule_name))
                .filter(|(level, _)| !matches!(level, RulePlainConfiguration::Off))
                .map_or_else(|| Schema::severity(rule_name), |(level, _)| level.into()),
        };
        Some(severity)
    }
    #[doc = r" Ensure that `recommended` is set to `true` or implied."]
    pub fn set_recommended(&mut self) {
        if self.all != Some(true) && self.recommended == Some(false) {
            self.recommended = Some(true)
        }
        if let Some(group) = &mut self.base {
            group.recommended = None;
        }
        if let Some(group) = &mut self.cluster {
            group.recommended = None;
        }
        if let Some(group) = &mut self.schema {
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
        if let Some(group) = self.base.as_ref() {
            group.collect_preset_rules(
                self.is_all_true(),
                !self.is_recommended_false(),
                &mut enabled_rules,
            );
            enabled_rules.extend(&group.get_enabled_rules());
            disabled_rules.extend(&group.get_disabled_rules());
        } else if self.is_all_true() {
            enabled_rules.extend(Base::all_rules_as_filters());
        } else if !self.is_recommended_false() {
            enabled_rules.extend(Base::recommended_rules_as_filters());
        }
        if let Some(group) = self.cluster.as_ref() {
            group.collect_preset_rules(
                self.is_all_true(),
                !self.is_recommended_false(),
                &mut enabled_rules,
            );
            enabled_rules.extend(&group.get_enabled_rules());
            disabled_rules.extend(&group.get_disabled_rules());
        } else if self.is_all_true() {
            enabled_rules.extend(Cluster::all_rules_as_filters());
        } else if !self.is_recommended_false() {
            enabled_rules.extend(Cluster::recommended_rules_as_filters());
        }
        if let Some(group) = self.schema.as_ref() {
            group.collect_preset_rules(
                self.is_all_true(),
                !self.is_recommended_false(),
                &mut enabled_rules,
            );
            enabled_rules.extend(&group.get_enabled_rules());
            disabled_rules.extend(&group.get_disabled_rules());
        } else if self.is_all_true() {
            enabled_rules.extend(Schema::all_rules_as_filters());
        } else if !self.is_recommended_false() {
            enabled_rules.extend(Schema::recommended_rules_as_filters());
        }
        enabled_rules.difference(&disabled_rules).copied().collect()
    }
    #[doc = r" It returns the disabled rules by configuration."]
    pub fn as_disabled_rules(&self) -> FxHashSet<RuleFilter<'static>> {
        let mut disabled_rules = FxHashSet::default();
        if let Some(group) = self.base.as_ref() {
            disabled_rules.extend(&group.get_disabled_rules());
        }
        if let Some(group) = self.cluster.as_ref() {
            disabled_rules.extend(&group.get_disabled_rules());
        }
        if let Some(group) = self.schema.as_ref() {
            disabled_rules.extend(&group.get_disabled_rules());
        }
        disabled_rules
    }
}
#[derive(Clone, Debug, Default, Deserialize, Eq, Merge, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
#[doc = r" A list of rules that belong to this group"]
pub struct Base {
    #[doc = r" It enables the recommended rules for this group"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommended: Option<bool>,
    #[doc = r" It enables ALL rules for this group."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all: Option<bool>,
    #[doc = "CompositePrimaryKeyTooManyColumns (B012): Detect tables with composite primary keys involving more than 4 columns"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub composite_primary_key_too_many_columns: Option<RuleConfiguration<()>>,
    #[doc = "HowManyObjectsWithUppercase (B005): Count number of objects with uppercase in name or in columns."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub how_many_objects_with_uppercase: Option<RuleConfiguration<()>>,
    #[doc = "HowManyRedudantIndex (B002): Count number of redundant index vs nb index."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub how_many_redudant_index: Option<RuleConfiguration<()>>,
    #[doc = "HowManyTableWithoutIndexOnFk (B003): Count number of tables without index on foreign key."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub how_many_table_without_index_on_fk: Option<RuleConfiguration<()>>,
    #[doc = "HowManyTableWithoutPrimaryKey (B001): Count number of tables without primary key."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub how_many_table_without_primary_key: Option<RuleConfiguration<()>>,
    #[doc = "HowManyTablesNeverSelected (B006): Count number of table(s) that has never been selected."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub how_many_tables_never_selected: Option<RuleConfiguration<()>>,
    #[doc = "HowManyTablesWithFkMismatch (B008): Count number of tables with foreign keys that do not match the key reference type."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub how_many_tables_with_fk_mismatch: Option<RuleConfiguration<()>>,
    #[doc = "HowManyTablesWithFkOutsideSchema (B007): Count number of tables with foreign keys outside their schema."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub how_many_tables_with_fk_outside_schema: Option<RuleConfiguration<()>>,
    #[doc = "HowManyTablesWithReservedKeywords (B010): Count number of database objects using reserved keywords in their names."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub how_many_tables_with_reserved_keywords: Option<RuleConfiguration<()>>,
    #[doc = "HowManyTablesWithSameTrigger (B009): Count number of tables using the same trigger vs nb table with their own triggers."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub how_many_tables_with_same_trigger: Option<RuleConfiguration<()>>,
    #[doc = "HowManyUnusedIndex (B004): Count number of unused index vs nb index (base on pg_stat_user_indexes, indexes associated to unique constraints are discard.)"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub how_many_unused_index: Option<RuleConfiguration<()>>,
    #[doc = "SeveralTableOwnerInSchema (B011): In a schema there are several tables owned by different owners."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub several_table_owner_in_schema: Option<RuleConfiguration<()>>,
}
impl Base {
    const GROUP_NAME: &'static str = "base";
    pub(crate) const GROUP_RULES: &'static [&'static str] = &[
        "compositePrimaryKeyTooManyColumns",
        "howManyObjectsWithUppercase",
        "howManyRedudantIndex",
        "howManyTableWithoutIndexOnFk",
        "howManyTableWithoutPrimaryKey",
        "howManyTablesNeverSelected",
        "howManyTablesWithFkMismatch",
        "howManyTablesWithFkOutsideSchema",
        "howManyTablesWithReservedKeywords",
        "howManyTablesWithSameTrigger",
        "howManyUnusedIndex",
        "severalTableOwnerInSchema",
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
        if let Some(rule) = self.composite_primary_key_too_many_columns.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[0]));
            }
        }
        if let Some(rule) = self.how_many_objects_with_uppercase.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[1]));
            }
        }
        if let Some(rule) = self.how_many_redudant_index.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[2]));
            }
        }
        if let Some(rule) = self.how_many_table_without_index_on_fk.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[3]));
            }
        }
        if let Some(rule) = self.how_many_table_without_primary_key.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[4]));
            }
        }
        if let Some(rule) = self.how_many_tables_never_selected.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[5]));
            }
        }
        if let Some(rule) = self.how_many_tables_with_fk_mismatch.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[6]));
            }
        }
        if let Some(rule) = self.how_many_tables_with_fk_outside_schema.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[7]));
            }
        }
        if let Some(rule) = self.how_many_tables_with_reserved_keywords.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[8]));
            }
        }
        if let Some(rule) = self.how_many_tables_with_same_trigger.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[9]));
            }
        }
        if let Some(rule) = self.how_many_unused_index.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[10]));
            }
        }
        if let Some(rule) = self.several_table_owner_in_schema.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[11]));
            }
        }
        index_set
    }
    pub(crate) fn get_disabled_rules(&self) -> FxHashSet<RuleFilter<'static>> {
        let mut index_set = FxHashSet::default();
        if let Some(rule) = self.composite_primary_key_too_many_columns.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[0]));
            }
        }
        if let Some(rule) = self.how_many_objects_with_uppercase.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[1]));
            }
        }
        if let Some(rule) = self.how_many_redudant_index.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[2]));
            }
        }
        if let Some(rule) = self.how_many_table_without_index_on_fk.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[3]));
            }
        }
        if let Some(rule) = self.how_many_table_without_primary_key.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[4]));
            }
        }
        if let Some(rule) = self.how_many_tables_never_selected.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[5]));
            }
        }
        if let Some(rule) = self.how_many_tables_with_fk_mismatch.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[6]));
            }
        }
        if let Some(rule) = self.how_many_tables_with_fk_outside_schema.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[7]));
            }
        }
        if let Some(rule) = self.how_many_tables_with_reserved_keywords.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[8]));
            }
        }
        if let Some(rule) = self.how_many_tables_with_same_trigger.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[9]));
            }
        }
        if let Some(rule) = self.how_many_unused_index.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[10]));
            }
        }
        if let Some(rule) = self.several_table_owner_in_schema.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[11]));
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
            "compositePrimaryKeyTooManyColumns" => Severity::Warning,
            "howManyObjectsWithUppercase" => Severity::Warning,
            "howManyRedudantIndex" => Severity::Warning,
            "howManyTableWithoutIndexOnFk" => Severity::Warning,
            "howManyTableWithoutPrimaryKey" => Severity::Warning,
            "howManyTablesNeverSelected" => Severity::Warning,
            "howManyTablesWithFkMismatch" => Severity::Warning,
            "howManyTablesWithFkOutsideSchema" => Severity::Warning,
            "howManyTablesWithReservedKeywords" => Severity::Warning,
            "howManyTablesWithSameTrigger" => Severity::Warning,
            "howManyUnusedIndex" => Severity::Warning,
            "severalTableOwnerInSchema" => Severity::Warning,
            _ => unreachable!(),
        }
    }
    pub(crate) fn get_rule_configuration(
        &self,
        rule_name: &str,
    ) -> Option<(RulePlainConfiguration, Option<RuleOptions>)> {
        match rule_name {
            "compositePrimaryKeyTooManyColumns" => self
                .composite_primary_key_too_many_columns
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "howManyObjectsWithUppercase" => self
                .how_many_objects_with_uppercase
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "howManyRedudantIndex" => self
                .how_many_redudant_index
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "howManyTableWithoutIndexOnFk" => self
                .how_many_table_without_index_on_fk
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "howManyTableWithoutPrimaryKey" => self
                .how_many_table_without_primary_key
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "howManyTablesNeverSelected" => self
                .how_many_tables_never_selected
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "howManyTablesWithFkMismatch" => self
                .how_many_tables_with_fk_mismatch
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "howManyTablesWithFkOutsideSchema" => self
                .how_many_tables_with_fk_outside_schema
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "howManyTablesWithReservedKeywords" => self
                .how_many_tables_with_reserved_keywords
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "howManyTablesWithSameTrigger" => self
                .how_many_tables_with_same_trigger
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "howManyUnusedIndex" => self
                .how_many_unused_index
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "severalTableOwnerInSchema" => self
                .several_table_owner_in_schema
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            _ => None,
        }
    }
}
#[derive(Clone, Debug, Default, Deserialize, Eq, Merge, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
#[doc = r" A list of rules that belong to this group"]
pub struct Cluster {
    #[doc = r" It enables the recommended rules for this group"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommended: Option<bool>,
    #[doc = r" It enables ALL rules for this group."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all: Option<bool>,
    #[doc = "PasswordEncryptionIsMd5 (C003): This configuration is not secure anymore and will prevent an upgrade to Postgres 18. Warning, you will need to reset all passwords after this is changed to scram-sha-256."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_encryption_is_md5: Option<RuleConfiguration<()>>,
    #[doc = "PgHbaEntriesWithMethodTrustOrPasswordShouldNotExists (C002): This configuration is extremely insecure and should only be used in a controlled, non-production environment for testing purposes. In a production environment, you should use more secure authentication methods such as md5, scram-sha-256, or cert, and restrict access to trusted IP addresses only."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pg_hba_entries_with_method_trust_or_password_should_not_exists:
        Option<RuleConfiguration<()>>,
    #[doc = "PgHbaEntriesWithMethodTrustShouldNotExists (C001): This configuration is extremely insecure and should only be used in a controlled, non-production environment for testing purposes. In a production environment, you should use more secure authentication methods such as md5, scram-sha-256, or cert, and restrict access to trusted IP addresses only."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pg_hba_entries_with_method_trust_should_not_exists: Option<RuleConfiguration<()>>,
}
impl Cluster {
    const GROUP_NAME: &'static str = "cluster";
    pub(crate) const GROUP_RULES: &'static [&'static str] = &[
        "passwordEncryptionIsMd5",
        "pgHbaEntriesWithMethodTrustOrPasswordShouldNotExists",
        "pgHbaEntriesWithMethodTrustShouldNotExists",
    ];
    const RECOMMENDED_RULES_AS_FILTERS: &'static [RuleFilter<'static>] = &[
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[0]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[1]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[2]),
    ];
    const ALL_RULES_AS_FILTERS: &'static [RuleFilter<'static>] = &[
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[0]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[1]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[2]),
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
        if let Some(rule) = self.password_encryption_is_md5.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[0]));
            }
        }
        if let Some(rule) = self
            .pg_hba_entries_with_method_trust_or_password_should_not_exists
            .as_ref()
        {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[1]));
            }
        }
        if let Some(rule) = self
            .pg_hba_entries_with_method_trust_should_not_exists
            .as_ref()
        {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[2]));
            }
        }
        index_set
    }
    pub(crate) fn get_disabled_rules(&self) -> FxHashSet<RuleFilter<'static>> {
        let mut index_set = FxHashSet::default();
        if let Some(rule) = self.password_encryption_is_md5.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[0]));
            }
        }
        if let Some(rule) = self
            .pg_hba_entries_with_method_trust_or_password_should_not_exists
            .as_ref()
        {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[1]));
            }
        }
        if let Some(rule) = self
            .pg_hba_entries_with_method_trust_should_not_exists
            .as_ref()
        {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[2]));
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
            "passwordEncryptionIsMd5" => Severity::Warning,
            "pgHbaEntriesWithMethodTrustOrPasswordShouldNotExists" => Severity::Warning,
            "pgHbaEntriesWithMethodTrustShouldNotExists" => Severity::Warning,
            _ => unreachable!(),
        }
    }
    pub(crate) fn get_rule_configuration(
        &self,
        rule_name: &str,
    ) -> Option<(RulePlainConfiguration, Option<RuleOptions>)> {
        match rule_name {
            "passwordEncryptionIsMd5" => self
                .password_encryption_is_md5
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "pgHbaEntriesWithMethodTrustOrPasswordShouldNotExists" => self
                .pg_hba_entries_with_method_trust_or_password_should_not_exists
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "pgHbaEntriesWithMethodTrustShouldNotExists" => self
                .pg_hba_entries_with_method_trust_should_not_exists
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            _ => None,
        }
    }
}
#[derive(Clone, Debug, Default, Deserialize, Eq, Merge, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
#[doc = r" A list of rules that belong to this group"]
pub struct Schema {
    #[doc = r" It enables the recommended rules for this group"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommended: Option<bool>,
    #[doc = r" It enables ALL rules for this group."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all: Option<bool>,
    #[doc = "OwnerSchemaIsInternalRole (S004): Owner of schema should not be any internal pg roles, or owner is a superuser (not sure it is necesary)."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_schema_is_internal_role: Option<RuleConfiguration<()>>,
    #[doc = "SchemaOwnerDoNotMatchTableOwner (S005): The schema owner and tables in the schema do not match."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_owner_do_not_match_table_owner: Option<RuleConfiguration<()>>,
    #[doc = "SchemaPrefixedOrSuffixedWithEnvt (S002): The schema is prefixed with one of staging,stg,preprod,prod,sandbox,sbox string. Means that when you refresh your preprod, staging environments from production, you have to rename the target schema from prod_ to stg_ or something like. It is possible, but it is never easy."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_prefixed_or_suffixed_with_envt: Option<RuleConfiguration<()>>,
    #[doc = "SchemaWithDefaultRoleNotGranted (S001): The schema has no default role. Means that futur table will not be granted through a role. So you will have to re-execute grants on it."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_with_default_role_not_granted: Option<RuleConfiguration<()>>,
    #[doc = "UnsecuredPublicSchema (S003): Only authorized users should be allowed to create objects."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unsecured_public_schema: Option<RuleConfiguration<()>>,
}
impl Schema {
    const GROUP_NAME: &'static str = "schema";
    pub(crate) const GROUP_RULES: &'static [&'static str] = &[
        "ownerSchemaIsInternalRole",
        "schemaOwnerDoNotMatchTableOwner",
        "schemaPrefixedOrSuffixedWithEnvt",
        "schemaWithDefaultRoleNotGranted",
        "unsecuredPublicSchema",
    ];
    const RECOMMENDED_RULES_AS_FILTERS: &'static [RuleFilter<'static>] = &[
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[0]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[1]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[2]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[3]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[4]),
    ];
    const ALL_RULES_AS_FILTERS: &'static [RuleFilter<'static>] = &[
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[0]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[1]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[2]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[3]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[4]),
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
        if let Some(rule) = self.owner_schema_is_internal_role.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[0]));
            }
        }
        if let Some(rule) = self.schema_owner_do_not_match_table_owner.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[1]));
            }
        }
        if let Some(rule) = self.schema_prefixed_or_suffixed_with_envt.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[2]));
            }
        }
        if let Some(rule) = self.schema_with_default_role_not_granted.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[3]));
            }
        }
        if let Some(rule) = self.unsecured_public_schema.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[4]));
            }
        }
        index_set
    }
    pub(crate) fn get_disabled_rules(&self) -> FxHashSet<RuleFilter<'static>> {
        let mut index_set = FxHashSet::default();
        if let Some(rule) = self.owner_schema_is_internal_role.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[0]));
            }
        }
        if let Some(rule) = self.schema_owner_do_not_match_table_owner.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[1]));
            }
        }
        if let Some(rule) = self.schema_prefixed_or_suffixed_with_envt.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[2]));
            }
        }
        if let Some(rule) = self.schema_with_default_role_not_granted.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[3]));
            }
        }
        if let Some(rule) = self.unsecured_public_schema.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[4]));
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
            "ownerSchemaIsInternalRole" => Severity::Warning,
            "schemaOwnerDoNotMatchTableOwner" => Severity::Warning,
            "schemaPrefixedOrSuffixedWithEnvt" => Severity::Warning,
            "schemaWithDefaultRoleNotGranted" => Severity::Warning,
            "unsecuredPublicSchema" => Severity::Warning,
            _ => unreachable!(),
        }
    }
    pub(crate) fn get_rule_configuration(
        &self,
        rule_name: &str,
    ) -> Option<(RulePlainConfiguration, Option<RuleOptions>)> {
        match rule_name {
            "ownerSchemaIsInternalRole" => self
                .owner_schema_is_internal_role
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "schemaOwnerDoNotMatchTableOwner" => self
                .schema_owner_do_not_match_table_owner
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "schemaPrefixedOrSuffixedWithEnvt" => self
                .schema_prefixed_or_suffixed_with_envt
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "schemaWithDefaultRoleNotGranted" => self
                .schema_with_default_role_not_granted
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "unsecuredPublicSchema" => self
                .unsecured_public_schema
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            _ => None,
        }
    }
}
#[doc = r" Push the configured rules to the analyser"]
pub fn push_to_analyser_rules(
    rules: &Rules,
    metadata: &pgls_analyse::MetadataRegistry,
    analyser_rules: &mut pgls_analyser::LinterRules,
) {
    if let Some(rules) = rules.base.as_ref() {
        for rule_name in Base::GROUP_RULES {
            if let Some((_, Some(rule_options))) = rules.get_rule_configuration(rule_name) {
                if let Some(rule_key) = metadata.find_rule("base", rule_name) {
                    analyser_rules.push_rule(rule_key, rule_options);
                }
            }
        }
    }
    if let Some(rules) = rules.cluster.as_ref() {
        for rule_name in Cluster::GROUP_RULES {
            if let Some((_, Some(rule_options))) = rules.get_rule_configuration(rule_name) {
                if let Some(rule_key) = metadata.find_rule("cluster", rule_name) {
                    analyser_rules.push_rule(rule_key, rule_options);
                }
            }
        }
    }
    if let Some(rules) = rules.schema.as_ref() {
        for rule_name in Schema::GROUP_RULES {
            if let Some((_, Some(rule_options))) = rules.get_rule_configuration(rule_name) {
                if let Some(rule_key) = metadata.find_rule("schema", rule_name) {
                    analyser_rules.push_rule(rule_key, rule_options);
                }
            }
        }
    }
}
#[test]
fn test_order() {
    for items in Base::GROUP_RULES.windows(2) {
        assert!(items[0] < items[1], "{} < {}", items[0], items[1]);
    }
    for items in Cluster::GROUP_RULES.windows(2) {
        assert!(items[0] < items[1], "{} < {}", items[0], items[1]);
    }
    for items in Schema::GROUP_RULES.windows(2) {
        assert!(items[0] < items[1], "{} < {}", items[0], items[1]);
    }
}
