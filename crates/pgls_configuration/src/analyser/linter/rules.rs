//! Generated file, do not edit by hand, see `xtask/codegen`

use crate::analyser::{RuleConfiguration, RulePlainConfiguration};
use biome_deserialize_macros::Merge;
use pgls_analyse::{RuleFilter, options::RuleOptions};
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
    Safety,
}
impl RuleGroup {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Safety => Safety::GROUP_NAME,
        }
    }
}
impl std::str::FromStr for RuleGroup {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            Safety::GROUP_NAME => Ok(Self::Safety),
            _ => Err("This rule group doesn't exist."),
        }
    }
}
#[derive(Clone, Debug, Default, Deserialize, Eq, Merge, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Rules {
    #[doc = r" It enables the lint rules recommended by Postgres Language Server. `true` by default."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommended: Option<bool>,
    #[doc = r" It enables ALL rules. The rules that belong to `nursery` won't be enabled."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety: Option<Safety>,
}
impl Rules {
    #[doc = r" Checks if the code coming from [pgls_diagnostics::Diagnostic] corresponds to a rule."]
    #[doc = r" Usually the code is built like {group}/{rule_name}"]
    pub fn has_rule(group: RuleGroup, rule_name: &str) -> Option<&'static str> {
        match group {
            RuleGroup::Safety => Safety::has_rule(rule_name),
        }
    }
    #[doc = r" Given a category coming from [Diagnostic](pgls_diagnostics::Diagnostic), this function returns"]
    #[doc = r" the [Severity](pgls_diagnostics::Severity) associated to the rule, if the configuration changed it."]
    #[doc = r" If the severity is off or not set, then the function returns the default severity of the rule,"]
    #[doc = r" which is configured at the rule definition."]
    #[doc = r" The function can return `None` if the rule is not properly configured."]
    pub fn get_severity_from_code(&self, category: &Category) -> Option<Severity> {
        let mut split_code = category.name().split('/');
        let _lint = split_code.next();
        debug_assert_eq!(_lint, Some("lint"));
        let group = <RuleGroup as std::str::FromStr>::from_str(split_code.next()?).ok()?;
        let rule_name = split_code.next()?;
        let rule_name = Self::has_rule(group, rule_name)?;
        let severity = match group {
            RuleGroup::Safety => self
                .safety
                .as_ref()
                .and_then(|group| group.get_rule_configuration(rule_name))
                .filter(|(level, _)| !matches!(level, RulePlainConfiguration::Off))
                .map_or_else(|| Safety::severity(rule_name), |(level, _)| level.into()),
        };
        Some(severity)
    }
    #[doc = r" Ensure that `recommended` is set to `true` or implied."]
    pub fn set_recommended(&mut self) {
        if self.all != Some(true) && self.recommended == Some(false) {
            self.recommended = Some(true)
        }
        if let Some(group) = &mut self.safety {
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
        if let Some(group) = self.safety.as_ref() {
            group.collect_preset_rules(
                self.is_all_true(),
                !self.is_recommended_false(),
                &mut enabled_rules,
            );
            enabled_rules.extend(&group.get_enabled_rules());
            disabled_rules.extend(&group.get_disabled_rules());
        } else if self.is_all_true() {
            enabled_rules.extend(Safety::all_rules_as_filters());
        } else if !self.is_recommended_false() {
            enabled_rules.extend(Safety::recommended_rules_as_filters());
        }
        enabled_rules.difference(&disabled_rules).copied().collect()
    }
    #[doc = r" It returns the disabled rules by configuration."]
    pub fn as_disabled_rules(&self) -> FxHashSet<RuleFilter<'static>> {
        let mut disabled_rules = FxHashSet::default();
        if let Some(group) = self.safety.as_ref() {
            disabled_rules.extend(&group.get_disabled_rules());
        }
        disabled_rules
    }
}
#[derive(Clone, Debug, Default, Deserialize, Eq, Merge, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
#[doc = r" A list of rules that belong to this group"]
pub struct Safety {
    #[doc = r" It enables the recommended rules for this group"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommended: Option<bool>,
    #[doc = r" It enables ALL rules for this group."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all: Option<bool>,
    #[doc = "Adding a column with a SERIAL type or GENERATED ALWAYS AS ... STORED causes a full table rewrite."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub add_serial_column: Option<RuleConfiguration<pgls_analyser::options::AddSerialColumn>>,
    #[doc = "Adding a column with a DEFAULT value may lead to a table rewrite while holding an ACCESS EXCLUSIVE lock."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adding_field_with_default:
        Option<RuleConfiguration<pgls_analyser::options::AddingFieldWithDefault>>,
    #[doc = "Adding a foreign key constraint requires a table scan and a SHARE ROW EXCLUSIVE lock on both tables, which blocks writes."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adding_foreign_key_constraint:
        Option<RuleConfiguration<pgls_analyser::options::AddingForeignKeyConstraint>>,
    #[doc = "Setting a column NOT NULL blocks reads while the table is scanned."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adding_not_null_field:
        Option<RuleConfiguration<pgls_analyser::options::AddingNotNullField>>,
    #[doc = "Adding a primary key constraint results in locks and table rewrites."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adding_primary_key_constraint:
        Option<RuleConfiguration<pgls_analyser::options::AddingPrimaryKeyConstraint>>,
    #[doc = "Adding a new column that is NOT NULL and has no default value to an existing table effectively makes it required."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adding_required_field:
        Option<RuleConfiguration<pgls_analyser::options::AddingRequiredField>>,
    #[doc = "Using CHAR(n) or CHARACTER(n) types is discouraged."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ban_char_field: Option<RuleConfiguration<pgls_analyser::options::BanCharField>>,
    #[doc = "Concurrent index creation is not allowed within a transaction."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ban_concurrent_index_creation_in_transaction:
        Option<RuleConfiguration<pgls_analyser::options::BanConcurrentIndexCreationInTransaction>>,
    #[doc = "Dropping a column may break existing clients."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ban_drop_column: Option<RuleConfiguration<pgls_analyser::options::BanDropColumn>>,
    #[doc = "Dropping a database may break existing clients (and everything else, really)."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ban_drop_database: Option<RuleConfiguration<pgls_analyser::options::BanDropDatabase>>,
    #[doc = "Dropping a NOT NULL constraint may break existing clients."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ban_drop_not_null: Option<RuleConfiguration<pgls_analyser::options::BanDropNotNull>>,
    #[doc = "Dropping a table may break existing clients."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ban_drop_table: Option<RuleConfiguration<pgls_analyser::options::BanDropTable>>,
    #[doc = "Using TRUNCATE's CASCADE option will truncate any tables that are also foreign-keyed to the specified tables."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ban_truncate_cascade: Option<RuleConfiguration<pgls_analyser::options::BanTruncateCascade>>,
    #[doc = "Changing a column type may break existing clients."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changing_column_type: Option<RuleConfiguration<pgls_analyser::options::ChangingColumnType>>,
    #[doc = "Adding constraints without NOT VALID blocks all reads and writes."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraint_missing_not_valid:
        Option<RuleConfiguration<pgls_analyser::options::ConstraintMissingNotValid>>,
    #[doc = "Creating enum types is not recommended for new applications."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creating_enum: Option<RuleConfiguration<pgls_analyser::options::CreatingEnum>>,
    #[doc = "Disallow adding a UNIQUE constraint without using an existing index."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disallow_unique_constraint:
        Option<RuleConfiguration<pgls_analyser::options::DisallowUniqueConstraint>>,
    #[doc = "Taking a dangerous lock without setting a lock timeout can cause indefinite blocking."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lock_timeout_warning: Option<RuleConfiguration<pgls_analyser::options::LockTimeoutWarning>>,
    #[doc = "Multiple ALTER TABLE statements on the same table should be combined into a single statement."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiple_alter_table: Option<RuleConfiguration<pgls_analyser::options::MultipleAlterTable>>,
    #[doc = "Prefer BIGINT over smaller integer types."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefer_big_int: Option<RuleConfiguration<pgls_analyser::options::PreferBigInt>>,
    #[doc = "Prefer BIGINT over INT/INTEGER types."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefer_bigint_over_int:
        Option<RuleConfiguration<pgls_analyser::options::PreferBigintOverInt>>,
    #[doc = "Prefer BIGINT over SMALLINT types."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefer_bigint_over_smallint:
        Option<RuleConfiguration<pgls_analyser::options::PreferBigintOverSmallint>>,
    #[doc = "Prefer using IDENTITY columns over serial columns."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefer_identity: Option<RuleConfiguration<pgls_analyser::options::PreferIdentity>>,
    #[doc = "Prefer JSONB over JSON types."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefer_jsonb: Option<RuleConfiguration<pgls_analyser::options::PreferJsonb>>,
    #[doc = "Prefer statements with guards for robustness in migrations."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefer_robust_stmts: Option<RuleConfiguration<pgls_analyser::options::PreferRobustStmts>>,
    #[doc = "Prefer using TEXT over VARCHAR(n) types."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefer_text_field: Option<RuleConfiguration<pgls_analyser::options::PreferTextField>>,
    #[doc = "Prefer TIMESTAMPTZ over TIMESTAMP types."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefer_timestamptz: Option<RuleConfiguration<pgls_analyser::options::PreferTimestamptz>>,
    #[doc = "Renaming columns may break existing queries and application code."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub renaming_column: Option<RuleConfiguration<pgls_analyser::options::RenamingColumn>>,
    #[doc = "Renaming tables may break existing queries and application code."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub renaming_table: Option<RuleConfiguration<pgls_analyser::options::RenamingTable>>,
    #[doc = "Creating indexes non-concurrently can lock the table for writes."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_concurrent_index_creation:
        Option<RuleConfiguration<pgls_analyser::options::RequireConcurrentIndexCreation>>,
    #[doc = "Dropping indexes non-concurrently can lock the table for reads."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_concurrent_index_deletion:
        Option<RuleConfiguration<pgls_analyser::options::RequireConcurrentIndexDeletion>>,
    #[doc = "Running additional statements while holding an ACCESS EXCLUSIVE lock blocks all table access."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub running_statement_while_holding_access_exclusive: Option<
        RuleConfiguration<pgls_analyser::options::RunningStatementWhileHoldingAccessExclusive>,
    >,
    #[doc = "Detects problematic transaction nesting that could lead to unexpected behavior."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_nesting: Option<RuleConfiguration<pgls_analyser::options::TransactionNesting>>,
}
impl Safety {
    const GROUP_NAME: &'static str = "safety";
    pub(crate) const GROUP_RULES: &'static [&'static str] = &[
        "addSerialColumn",
        "addingFieldWithDefault",
        "addingForeignKeyConstraint",
        "addingNotNullField",
        "addingPrimaryKeyConstraint",
        "addingRequiredField",
        "banCharField",
        "banConcurrentIndexCreationInTransaction",
        "banDropColumn",
        "banDropDatabase",
        "banDropNotNull",
        "banDropTable",
        "banTruncateCascade",
        "changingColumnType",
        "constraintMissingNotValid",
        "creatingEnum",
        "disallowUniqueConstraint",
        "lockTimeoutWarning",
        "multipleAlterTable",
        "preferBigInt",
        "preferBigintOverInt",
        "preferBigintOverSmallint",
        "preferIdentity",
        "preferJsonb",
        "preferRobustStmts",
        "preferTextField",
        "preferTimestamptz",
        "renamingColumn",
        "renamingTable",
        "requireConcurrentIndexCreation",
        "requireConcurrentIndexDeletion",
        "runningStatementWhileHoldingAccessExclusive",
        "transactionNesting",
    ];
    const RECOMMENDED_RULES_AS_FILTERS: &'static [RuleFilter<'static>] = &[
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[0]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[1]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[2]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[3]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[4]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[7]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[8]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[10]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[11]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[17]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[18]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[31]),
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
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[14]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[15]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[16]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[17]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[18]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[19]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[20]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[21]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[22]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[23]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[24]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[25]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[26]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[27]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[28]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[29]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[30]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[31]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[32]),
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
        if let Some(rule) = self.add_serial_column.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[0]));
            }
        }
        if let Some(rule) = self.adding_field_with_default.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[1]));
            }
        }
        if let Some(rule) = self.adding_foreign_key_constraint.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[2]));
            }
        }
        if let Some(rule) = self.adding_not_null_field.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[3]));
            }
        }
        if let Some(rule) = self.adding_primary_key_constraint.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[4]));
            }
        }
        if let Some(rule) = self.adding_required_field.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[5]));
            }
        }
        if let Some(rule) = self.ban_char_field.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[6]));
            }
        }
        if let Some(rule) = self.ban_concurrent_index_creation_in_transaction.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[7]));
            }
        }
        if let Some(rule) = self.ban_drop_column.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[8]));
            }
        }
        if let Some(rule) = self.ban_drop_database.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[9]));
            }
        }
        if let Some(rule) = self.ban_drop_not_null.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[10]));
            }
        }
        if let Some(rule) = self.ban_drop_table.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[11]));
            }
        }
        if let Some(rule) = self.ban_truncate_cascade.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[12]));
            }
        }
        if let Some(rule) = self.changing_column_type.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[13]));
            }
        }
        if let Some(rule) = self.constraint_missing_not_valid.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[14]));
            }
        }
        if let Some(rule) = self.creating_enum.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[15]));
            }
        }
        if let Some(rule) = self.disallow_unique_constraint.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[16]));
            }
        }
        if let Some(rule) = self.lock_timeout_warning.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[17]));
            }
        }
        if let Some(rule) = self.multiple_alter_table.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[18]));
            }
        }
        if let Some(rule) = self.prefer_big_int.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[19]));
            }
        }
        if let Some(rule) = self.prefer_bigint_over_int.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[20]));
            }
        }
        if let Some(rule) = self.prefer_bigint_over_smallint.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[21]));
            }
        }
        if let Some(rule) = self.prefer_identity.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[22]));
            }
        }
        if let Some(rule) = self.prefer_jsonb.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[23]));
            }
        }
        if let Some(rule) = self.prefer_robust_stmts.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[24]));
            }
        }
        if let Some(rule) = self.prefer_text_field.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[25]));
            }
        }
        if let Some(rule) = self.prefer_timestamptz.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[26]));
            }
        }
        if let Some(rule) = self.renaming_column.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[27]));
            }
        }
        if let Some(rule) = self.renaming_table.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[28]));
            }
        }
        if let Some(rule) = self.require_concurrent_index_creation.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[29]));
            }
        }
        if let Some(rule) = self.require_concurrent_index_deletion.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[30]));
            }
        }
        if let Some(rule) = self
            .running_statement_while_holding_access_exclusive
            .as_ref()
        {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[31]));
            }
        }
        if let Some(rule) = self.transaction_nesting.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[32]));
            }
        }
        index_set
    }
    pub(crate) fn get_disabled_rules(&self) -> FxHashSet<RuleFilter<'static>> {
        let mut index_set = FxHashSet::default();
        if let Some(rule) = self.add_serial_column.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[0]));
            }
        }
        if let Some(rule) = self.adding_field_with_default.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[1]));
            }
        }
        if let Some(rule) = self.adding_foreign_key_constraint.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[2]));
            }
        }
        if let Some(rule) = self.adding_not_null_field.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[3]));
            }
        }
        if let Some(rule) = self.adding_primary_key_constraint.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[4]));
            }
        }
        if let Some(rule) = self.adding_required_field.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[5]));
            }
        }
        if let Some(rule) = self.ban_char_field.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[6]));
            }
        }
        if let Some(rule) = self.ban_concurrent_index_creation_in_transaction.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[7]));
            }
        }
        if let Some(rule) = self.ban_drop_column.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[8]));
            }
        }
        if let Some(rule) = self.ban_drop_database.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[9]));
            }
        }
        if let Some(rule) = self.ban_drop_not_null.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[10]));
            }
        }
        if let Some(rule) = self.ban_drop_table.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[11]));
            }
        }
        if let Some(rule) = self.ban_truncate_cascade.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[12]));
            }
        }
        if let Some(rule) = self.changing_column_type.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[13]));
            }
        }
        if let Some(rule) = self.constraint_missing_not_valid.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[14]));
            }
        }
        if let Some(rule) = self.creating_enum.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[15]));
            }
        }
        if let Some(rule) = self.disallow_unique_constraint.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[16]));
            }
        }
        if let Some(rule) = self.lock_timeout_warning.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[17]));
            }
        }
        if let Some(rule) = self.multiple_alter_table.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[18]));
            }
        }
        if let Some(rule) = self.prefer_big_int.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[19]));
            }
        }
        if let Some(rule) = self.prefer_bigint_over_int.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[20]));
            }
        }
        if let Some(rule) = self.prefer_bigint_over_smallint.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[21]));
            }
        }
        if let Some(rule) = self.prefer_identity.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[22]));
            }
        }
        if let Some(rule) = self.prefer_jsonb.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[23]));
            }
        }
        if let Some(rule) = self.prefer_robust_stmts.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[24]));
            }
        }
        if let Some(rule) = self.prefer_text_field.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[25]));
            }
        }
        if let Some(rule) = self.prefer_timestamptz.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[26]));
            }
        }
        if let Some(rule) = self.renaming_column.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[27]));
            }
        }
        if let Some(rule) = self.renaming_table.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[28]));
            }
        }
        if let Some(rule) = self.require_concurrent_index_creation.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[29]));
            }
        }
        if let Some(rule) = self.require_concurrent_index_deletion.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[30]));
            }
        }
        if let Some(rule) = self
            .running_statement_while_holding_access_exclusive
            .as_ref()
        {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[31]));
            }
        }
        if let Some(rule) = self.transaction_nesting.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[32]));
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
            "addSerialColumn" => Severity::Warning,
            "addingFieldWithDefault" => Severity::Warning,
            "addingForeignKeyConstraint" => Severity::Warning,
            "addingNotNullField" => Severity::Warning,
            "addingPrimaryKeyConstraint" => Severity::Warning,
            "addingRequiredField" => Severity::Error,
            "banCharField" => Severity::Warning,
            "banConcurrentIndexCreationInTransaction" => Severity::Error,
            "banDropColumn" => Severity::Warning,
            "banDropDatabase" => Severity::Warning,
            "banDropNotNull" => Severity::Warning,
            "banDropTable" => Severity::Warning,
            "banTruncateCascade" => Severity::Error,
            "changingColumnType" => Severity::Warning,
            "constraintMissingNotValid" => Severity::Warning,
            "creatingEnum" => Severity::Warning,
            "disallowUniqueConstraint" => Severity::Error,
            "lockTimeoutWarning" => Severity::Warning,
            "multipleAlterTable" => Severity::Warning,
            "preferBigInt" => Severity::Warning,
            "preferBigintOverInt" => Severity::Warning,
            "preferBigintOverSmallint" => Severity::Warning,
            "preferIdentity" => Severity::Warning,
            "preferJsonb" => Severity::Warning,
            "preferRobustStmts" => Severity::Warning,
            "preferTextField" => Severity::Warning,
            "preferTimestamptz" => Severity::Warning,
            "renamingColumn" => Severity::Warning,
            "renamingTable" => Severity::Warning,
            "requireConcurrentIndexCreation" => Severity::Warning,
            "requireConcurrentIndexDeletion" => Severity::Warning,
            "runningStatementWhileHoldingAccessExclusive" => Severity::Warning,
            "transactionNesting" => Severity::Warning,
            _ => unreachable!(),
        }
    }
    pub(crate) fn get_rule_configuration(
        &self,
        rule_name: &str,
    ) -> Option<(RulePlainConfiguration, Option<RuleOptions>)> {
        match rule_name {
            "addSerialColumn" => self
                .add_serial_column
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "addingFieldWithDefault" => self
                .adding_field_with_default
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "addingForeignKeyConstraint" => self
                .adding_foreign_key_constraint
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "addingNotNullField" => self
                .adding_not_null_field
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "addingPrimaryKeyConstraint" => self
                .adding_primary_key_constraint
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "addingRequiredField" => self
                .adding_required_field
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "banCharField" => self
                .ban_char_field
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "banConcurrentIndexCreationInTransaction" => self
                .ban_concurrent_index_creation_in_transaction
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "banDropColumn" => self
                .ban_drop_column
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "banDropDatabase" => self
                .ban_drop_database
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "banDropNotNull" => self
                .ban_drop_not_null
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "banDropTable" => self
                .ban_drop_table
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "banTruncateCascade" => self
                .ban_truncate_cascade
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "changingColumnType" => self
                .changing_column_type
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "constraintMissingNotValid" => self
                .constraint_missing_not_valid
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "creatingEnum" => self
                .creating_enum
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "disallowUniqueConstraint" => self
                .disallow_unique_constraint
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "lockTimeoutWarning" => self
                .lock_timeout_warning
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "multipleAlterTable" => self
                .multiple_alter_table
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "preferBigInt" => self
                .prefer_big_int
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "preferBigintOverInt" => self
                .prefer_bigint_over_int
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "preferBigintOverSmallint" => self
                .prefer_bigint_over_smallint
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "preferIdentity" => self
                .prefer_identity
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "preferJsonb" => self
                .prefer_jsonb
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "preferRobustStmts" => self
                .prefer_robust_stmts
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "preferTextField" => self
                .prefer_text_field
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "preferTimestamptz" => self
                .prefer_timestamptz
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "renamingColumn" => self
                .renaming_column
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "renamingTable" => self
                .renaming_table
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "requireConcurrentIndexCreation" => self
                .require_concurrent_index_creation
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "requireConcurrentIndexDeletion" => self
                .require_concurrent_index_deletion
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "runningStatementWhileHoldingAccessExclusive" => self
                .running_statement_while_holding_access_exclusive
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "transactionNesting" => self
                .transaction_nesting
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            _ => None,
        }
    }
}
#[test]
fn test_order() {
    for items in Safety::GROUP_RULES.windows(2) {
        assert!(items[0] < items[1], "{} < {}", items[0], items[1]);
    }
}
