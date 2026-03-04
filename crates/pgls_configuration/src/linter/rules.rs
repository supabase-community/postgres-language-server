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
#[cfg_attr(feature = "schema", schemars(rename = "LinterRules"))]
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
        let _category_prefix = split_code.next();
        debug_assert_eq!(_category_prefix, Some("lint"));
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
    #[doc = "Adding an exclusion constraint acquires an ACCESS EXCLUSIVE lock."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ban_add_exclusion_constraint:
        Option<RuleConfiguration<pgls_analyser::options::BanAddExclusionConstraint>>,
    #[doc = "ALTER TYPE ... ADD VALUE cannot run inside a transaction block in older Postgres versions."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ban_alter_enum_add_value:
        Option<RuleConfiguration<pgls_analyser::options::BanAlterEnumAddValue>>,
    #[doc = "Attaching a partition acquires an ACCESS EXCLUSIVE lock on the parent table."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ban_attach_partition: Option<RuleConfiguration<pgls_analyser::options::BanAttachPartition>>,
    #[doc = "REFRESH MATERIALIZED VIEW without CONCURRENTLY acquires an ACCESS EXCLUSIVE lock."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ban_blocking_refresh_matview:
        Option<RuleConfiguration<pgls_analyser::options::BanBlockingRefreshMatview>>,
    #[doc = "Using CHAR(n) or CHARACTER(n) types is discouraged."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ban_char_field: Option<RuleConfiguration<pgls_analyser::options::BanCharField>>,
    #[doc = "Concurrent index creation is not allowed within a transaction."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ban_concurrent_index_creation_in_transaction:
        Option<RuleConfiguration<pgls_analyser::options::BanConcurrentIndexCreationInTransaction>>,
    #[doc = "Creating a trigger acquires a SHARE ROW EXCLUSIVE lock on the table."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ban_create_trigger: Option<RuleConfiguration<pgls_analyser::options::BanCreateTrigger>>,
    #[doc = "A DELETE statement without a WHERE clause will remove all rows from the table."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ban_delete_without_where:
        Option<RuleConfiguration<pgls_analyser::options::BanDeleteWithoutWhere>>,
    #[doc = "Dropping a column may break existing clients."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ban_drop_column: Option<RuleConfiguration<pgls_analyser::options::BanDropColumn>>,
    #[doc = "Dropping a database may break existing clients (and everything else, really)."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ban_drop_database: Option<RuleConfiguration<pgls_analyser::options::BanDropDatabase>>,
    #[doc = "Dropping a NOT NULL constraint may break existing clients."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ban_drop_not_null: Option<RuleConfiguration<pgls_analyser::options::BanDropNotNull>>,
    #[doc = "Dropping a schema will remove all objects within it and may break existing clients."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ban_drop_schema: Option<RuleConfiguration<pgls_analyser::options::BanDropSchema>>,
    #[doc = "Dropping a table may break existing clients."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ban_drop_table: Option<RuleConfiguration<pgls_analyser::options::BanDropTable>>,
    #[doc = "Dropping a trigger acquires an ACCESS EXCLUSIVE lock on the table."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ban_drop_trigger: Option<RuleConfiguration<pgls_analyser::options::BanDropTrigger>>,
    #[doc = "Enabling or disabling a trigger acquires a SHARE ROW EXCLUSIVE lock."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ban_enable_disable_trigger:
        Option<RuleConfiguration<pgls_analyser::options::BanEnableDisableTrigger>>,
    #[doc = "Validating a constraint in the same transaction it was added as NOT VALID defeats the purpose."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ban_not_valid_validate_same_transaction:
        Option<RuleConfiguration<pgls_analyser::options::BanNotValidValidateSameTransaction>>,
    #[doc = "Truncating a table removes all rows and can cause data loss in production."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ban_truncate: Option<RuleConfiguration<pgls_analyser::options::BanTruncate>>,
    #[doc = "Using TRUNCATE's CASCADE option will truncate any tables that are also foreign-keyed to the specified tables."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ban_truncate_cascade: Option<RuleConfiguration<pgls_analyser::options::BanTruncateCascade>>,
    #[doc = "An UPDATE statement without a WHERE clause will modify all rows in the table."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ban_update_without_where:
        Option<RuleConfiguration<pgls_analyser::options::BanUpdateWithoutWhere>>,
    #[doc = "VACUUM FULL rewrites the entire table and acquires an ACCESS EXCLUSIVE lock."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ban_vacuum_full: Option<RuleConfiguration<pgls_analyser::options::BanVacuumFull>>,
    #[doc = "Changing a column type may require a table rewrite and break existing clients."]
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
    #[doc = "Detaching a partition without CONCURRENTLY acquires an ACCESS EXCLUSIVE lock."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_concurrent_detach_partition:
        Option<RuleConfiguration<pgls_analyser::options::RequireConcurrentDetachPartition>>,
    #[doc = "Creating indexes non-concurrently can lock the table for writes."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_concurrent_index_creation:
        Option<RuleConfiguration<pgls_analyser::options::RequireConcurrentIndexCreation>>,
    #[doc = "Dropping indexes non-concurrently can lock the table for reads."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_concurrent_index_deletion:
        Option<RuleConfiguration<pgls_analyser::options::RequireConcurrentIndexDeletion>>,
    #[doc = "REINDEX without CONCURRENTLY acquires an ACCESS EXCLUSIVE lock on the table."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_concurrent_reindex:
        Option<RuleConfiguration<pgls_analyser::options::RequireConcurrentReindex>>,
    #[doc = "Dangerous lock statements should be preceded by SET idle_in_transaction_session_timeout."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_idle_in_transaction_timeout:
        Option<RuleConfiguration<pgls_analyser::options::RequireIdleInTransactionTimeout>>,
    #[doc = "Dangerous lock statements should be preceded by SET statement_timeout."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_statement_timeout:
        Option<RuleConfiguration<pgls_analyser::options::RequireStatementTimeout>>,
    #[doc = "Running additional statements while holding an ACCESS EXCLUSIVE lock blocks all table access."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub running_statement_while_holding_access_exclusive: Option<
        RuleConfiguration<pgls_analyser::options::RunningStatementWhileHoldingAccessExclusive>,
    >,
    #[doc = "Detects problematic transaction nesting that could lead to unexpected behavior."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_nesting: Option<RuleConfiguration<pgls_analyser::options::TransactionNesting>>,
    #[doc = "REFRESH MATERIALIZED VIEW CONCURRENTLY still acquires an EXCLUSIVE lock."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warn_refresh_matview_concurrent:
        Option<RuleConfiguration<pgls_analyser::options::WarnRefreshMatviewConcurrent>>,
    #[doc = "Acquiring ACCESS EXCLUSIVE locks on multiple tables widens the lock window."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warn_wide_lock_window:
        Option<RuleConfiguration<pgls_analyser::options::WarnWideLockWindow>>,
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
        "banAddExclusionConstraint",
        "banAlterEnumAddValue",
        "banAttachPartition",
        "banBlockingRefreshMatview",
        "banCharField",
        "banConcurrentIndexCreationInTransaction",
        "banCreateTrigger",
        "banDeleteWithoutWhere",
        "banDropColumn",
        "banDropDatabase",
        "banDropNotNull",
        "banDropSchema",
        "banDropTable",
        "banDropTrigger",
        "banEnableDisableTrigger",
        "banNotValidValidateSameTransaction",
        "banTruncate",
        "banTruncateCascade",
        "banUpdateWithoutWhere",
        "banVacuumFull",
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
        "requireConcurrentDetachPartition",
        "requireConcurrentIndexCreation",
        "requireConcurrentIndexDeletion",
        "requireConcurrentReindex",
        "requireIdleInTransactionTimeout",
        "requireStatementTimeout",
        "runningStatementWhileHoldingAccessExclusive",
        "transactionNesting",
        "warnRefreshMatviewConcurrent",
        "warnWideLockWindow",
    ];
    const RECOMMENDED_RULES_AS_FILTERS: &'static [RuleFilter<'static>] = &[
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[0]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[1]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[2]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[3]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[4]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[6]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[8]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[9]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[11]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[13]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[14]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[16]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[17]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[18]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[21]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[22]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[24]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[25]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[30]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[31]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[42]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[45]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[48]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[51]),
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
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[33]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[34]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[35]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[36]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[37]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[38]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[39]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[40]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[41]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[42]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[43]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[44]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[45]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[46]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[47]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[48]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[49]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[50]),
        RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[51]),
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
        if let Some(rule) = self.ban_add_exclusion_constraint.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[6]));
            }
        }
        if let Some(rule) = self.ban_alter_enum_add_value.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[7]));
            }
        }
        if let Some(rule) = self.ban_attach_partition.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[8]));
            }
        }
        if let Some(rule) = self.ban_blocking_refresh_matview.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[9]));
            }
        }
        if let Some(rule) = self.ban_char_field.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[10]));
            }
        }
        if let Some(rule) = self.ban_concurrent_index_creation_in_transaction.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[11]));
            }
        }
        if let Some(rule) = self.ban_create_trigger.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[12]));
            }
        }
        if let Some(rule) = self.ban_delete_without_where.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[13]));
            }
        }
        if let Some(rule) = self.ban_drop_column.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[14]));
            }
        }
        if let Some(rule) = self.ban_drop_database.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[15]));
            }
        }
        if let Some(rule) = self.ban_drop_not_null.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[16]));
            }
        }
        if let Some(rule) = self.ban_drop_schema.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[17]));
            }
        }
        if let Some(rule) = self.ban_drop_table.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[18]));
            }
        }
        if let Some(rule) = self.ban_drop_trigger.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[19]));
            }
        }
        if let Some(rule) = self.ban_enable_disable_trigger.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[20]));
            }
        }
        if let Some(rule) = self.ban_not_valid_validate_same_transaction.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[21]));
            }
        }
        if let Some(rule) = self.ban_truncate.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[22]));
            }
        }
        if let Some(rule) = self.ban_truncate_cascade.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[23]));
            }
        }
        if let Some(rule) = self.ban_update_without_where.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[24]));
            }
        }
        if let Some(rule) = self.ban_vacuum_full.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[25]));
            }
        }
        if let Some(rule) = self.changing_column_type.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[26]));
            }
        }
        if let Some(rule) = self.constraint_missing_not_valid.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[27]));
            }
        }
        if let Some(rule) = self.creating_enum.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[28]));
            }
        }
        if let Some(rule) = self.disallow_unique_constraint.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[29]));
            }
        }
        if let Some(rule) = self.lock_timeout_warning.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[30]));
            }
        }
        if let Some(rule) = self.multiple_alter_table.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[31]));
            }
        }
        if let Some(rule) = self.prefer_big_int.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[32]));
            }
        }
        if let Some(rule) = self.prefer_bigint_over_int.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[33]));
            }
        }
        if let Some(rule) = self.prefer_bigint_over_smallint.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[34]));
            }
        }
        if let Some(rule) = self.prefer_identity.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[35]));
            }
        }
        if let Some(rule) = self.prefer_jsonb.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[36]));
            }
        }
        if let Some(rule) = self.prefer_robust_stmts.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[37]));
            }
        }
        if let Some(rule) = self.prefer_text_field.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[38]));
            }
        }
        if let Some(rule) = self.prefer_timestamptz.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[39]));
            }
        }
        if let Some(rule) = self.renaming_column.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[40]));
            }
        }
        if let Some(rule) = self.renaming_table.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[41]));
            }
        }
        if let Some(rule) = self.require_concurrent_detach_partition.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[42]));
            }
        }
        if let Some(rule) = self.require_concurrent_index_creation.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[43]));
            }
        }
        if let Some(rule) = self.require_concurrent_index_deletion.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[44]));
            }
        }
        if let Some(rule) = self.require_concurrent_reindex.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[45]));
            }
        }
        if let Some(rule) = self.require_idle_in_transaction_timeout.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[46]));
            }
        }
        if let Some(rule) = self.require_statement_timeout.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[47]));
            }
        }
        if let Some(rule) = self
            .running_statement_while_holding_access_exclusive
            .as_ref()
        {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[48]));
            }
        }
        if let Some(rule) = self.transaction_nesting.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[49]));
            }
        }
        if let Some(rule) = self.warn_refresh_matview_concurrent.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[50]));
            }
        }
        if let Some(rule) = self.warn_wide_lock_window.as_ref() {
            if rule.is_enabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[51]));
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
        if let Some(rule) = self.ban_add_exclusion_constraint.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[6]));
            }
        }
        if let Some(rule) = self.ban_alter_enum_add_value.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[7]));
            }
        }
        if let Some(rule) = self.ban_attach_partition.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[8]));
            }
        }
        if let Some(rule) = self.ban_blocking_refresh_matview.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[9]));
            }
        }
        if let Some(rule) = self.ban_char_field.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[10]));
            }
        }
        if let Some(rule) = self.ban_concurrent_index_creation_in_transaction.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[11]));
            }
        }
        if let Some(rule) = self.ban_create_trigger.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[12]));
            }
        }
        if let Some(rule) = self.ban_delete_without_where.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[13]));
            }
        }
        if let Some(rule) = self.ban_drop_column.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[14]));
            }
        }
        if let Some(rule) = self.ban_drop_database.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[15]));
            }
        }
        if let Some(rule) = self.ban_drop_not_null.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[16]));
            }
        }
        if let Some(rule) = self.ban_drop_schema.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[17]));
            }
        }
        if let Some(rule) = self.ban_drop_table.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[18]));
            }
        }
        if let Some(rule) = self.ban_drop_trigger.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[19]));
            }
        }
        if let Some(rule) = self.ban_enable_disable_trigger.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[20]));
            }
        }
        if let Some(rule) = self.ban_not_valid_validate_same_transaction.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[21]));
            }
        }
        if let Some(rule) = self.ban_truncate.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[22]));
            }
        }
        if let Some(rule) = self.ban_truncate_cascade.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[23]));
            }
        }
        if let Some(rule) = self.ban_update_without_where.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[24]));
            }
        }
        if let Some(rule) = self.ban_vacuum_full.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[25]));
            }
        }
        if let Some(rule) = self.changing_column_type.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[26]));
            }
        }
        if let Some(rule) = self.constraint_missing_not_valid.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[27]));
            }
        }
        if let Some(rule) = self.creating_enum.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[28]));
            }
        }
        if let Some(rule) = self.disallow_unique_constraint.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[29]));
            }
        }
        if let Some(rule) = self.lock_timeout_warning.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[30]));
            }
        }
        if let Some(rule) = self.multiple_alter_table.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[31]));
            }
        }
        if let Some(rule) = self.prefer_big_int.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[32]));
            }
        }
        if let Some(rule) = self.prefer_bigint_over_int.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[33]));
            }
        }
        if let Some(rule) = self.prefer_bigint_over_smallint.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[34]));
            }
        }
        if let Some(rule) = self.prefer_identity.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[35]));
            }
        }
        if let Some(rule) = self.prefer_jsonb.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[36]));
            }
        }
        if let Some(rule) = self.prefer_robust_stmts.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[37]));
            }
        }
        if let Some(rule) = self.prefer_text_field.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[38]));
            }
        }
        if let Some(rule) = self.prefer_timestamptz.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[39]));
            }
        }
        if let Some(rule) = self.renaming_column.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[40]));
            }
        }
        if let Some(rule) = self.renaming_table.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[41]));
            }
        }
        if let Some(rule) = self.require_concurrent_detach_partition.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[42]));
            }
        }
        if let Some(rule) = self.require_concurrent_index_creation.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[43]));
            }
        }
        if let Some(rule) = self.require_concurrent_index_deletion.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[44]));
            }
        }
        if let Some(rule) = self.require_concurrent_reindex.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[45]));
            }
        }
        if let Some(rule) = self.require_idle_in_transaction_timeout.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[46]));
            }
        }
        if let Some(rule) = self.require_statement_timeout.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[47]));
            }
        }
        if let Some(rule) = self
            .running_statement_while_holding_access_exclusive
            .as_ref()
        {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[48]));
            }
        }
        if let Some(rule) = self.transaction_nesting.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[49]));
            }
        }
        if let Some(rule) = self.warn_refresh_matview_concurrent.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[50]));
            }
        }
        if let Some(rule) = self.warn_wide_lock_window.as_ref() {
            if rule.is_disabled() {
                index_set.insert(RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[51]));
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
            "banAddExclusionConstraint" => Severity::Warning,
            "banAlterEnumAddValue" => Severity::Warning,
            "banAttachPartition" => Severity::Warning,
            "banBlockingRefreshMatview" => Severity::Warning,
            "banCharField" => Severity::Warning,
            "banConcurrentIndexCreationInTransaction" => Severity::Error,
            "banCreateTrigger" => Severity::Warning,
            "banDeleteWithoutWhere" => Severity::Warning,
            "banDropColumn" => Severity::Warning,
            "banDropDatabase" => Severity::Warning,
            "banDropNotNull" => Severity::Warning,
            "banDropSchema" => Severity::Error,
            "banDropTable" => Severity::Warning,
            "banDropTrigger" => Severity::Warning,
            "banEnableDisableTrigger" => Severity::Warning,
            "banNotValidValidateSameTransaction" => Severity::Error,
            "banTruncate" => Severity::Error,
            "banTruncateCascade" => Severity::Error,
            "banUpdateWithoutWhere" => Severity::Warning,
            "banVacuumFull" => Severity::Error,
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
            "requireConcurrentDetachPartition" => Severity::Warning,
            "requireConcurrentIndexCreation" => Severity::Warning,
            "requireConcurrentIndexDeletion" => Severity::Warning,
            "requireConcurrentReindex" => Severity::Warning,
            "requireIdleInTransactionTimeout" => Severity::Warning,
            "requireStatementTimeout" => Severity::Warning,
            "runningStatementWhileHoldingAccessExclusive" => Severity::Warning,
            "transactionNesting" => Severity::Warning,
            "warnRefreshMatviewConcurrent" => Severity::Warning,
            "warnWideLockWindow" => Severity::Warning,
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
            "banAddExclusionConstraint" => self
                .ban_add_exclusion_constraint
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "banAlterEnumAddValue" => self
                .ban_alter_enum_add_value
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "banAttachPartition" => self
                .ban_attach_partition
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "banBlockingRefreshMatview" => self
                .ban_blocking_refresh_matview
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
            "banCreateTrigger" => self
                .ban_create_trigger
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "banDeleteWithoutWhere" => self
                .ban_delete_without_where
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
            "banDropSchema" => self
                .ban_drop_schema
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "banDropTable" => self
                .ban_drop_table
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "banDropTrigger" => self
                .ban_drop_trigger
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "banEnableDisableTrigger" => self
                .ban_enable_disable_trigger
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "banNotValidValidateSameTransaction" => self
                .ban_not_valid_validate_same_transaction
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "banTruncate" => self
                .ban_truncate
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "banTruncateCascade" => self
                .ban_truncate_cascade
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "banUpdateWithoutWhere" => self
                .ban_update_without_where
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "banVacuumFull" => self
                .ban_vacuum_full
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
            "requireConcurrentDetachPartition" => self
                .require_concurrent_detach_partition
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
            "requireConcurrentReindex" => self
                .require_concurrent_reindex
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "requireIdleInTransactionTimeout" => self
                .require_idle_in_transaction_timeout
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "requireStatementTimeout" => self
                .require_statement_timeout
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
            "warnRefreshMatviewConcurrent" => self
                .warn_refresh_matview_concurrent
                .as_ref()
                .map(|conf| (conf.level(), conf.get_options())),
            "warnWideLockWindow" => self
                .warn_wide_lock_window
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
    if let Some(rules) = rules.safety.as_ref() {
        for rule_name in Safety::GROUP_RULES {
            if let Some((_, Some(rule_options))) = rules.get_rule_configuration(rule_name) {
                if let Some(rule_key) = metadata.find_rule("safety", rule_name) {
                    analyser_rules.push_rule(rule_key, rule_options);
                }
            }
        }
    }
}
#[test]
fn test_order() {
    for items in Safety::GROUP_RULES.windows(2) {
        assert!(items[0] < items[1], "{} < {}", items[0], items[1]);
    }
}
