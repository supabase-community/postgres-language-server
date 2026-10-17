//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
use crate::rules::{RuleConfiguration, RulePlainConfiguration};
use pgls_analyser::RuleOptions;
use pgls_configuration_macros::Merge;
use pgls_diagnostics::Severity;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[doc = r" The static metadata of a linter rule."]
#[derive(Clone, Copy, Debug)]
pub struct LinterRuleMetadata {
    pub group: &'static str,
    pub name: &'static str,
    pub recommended: bool,
    pub severity: Severity,
}
#[doc = r" All linter rules, sorted by name."]
pub const LINTER_RULES: &[LinterRuleMetadata] = &[
    LinterRuleMetadata {
        group: "safety",
        name: "addSerialColumn",
        recommended: true,
        severity: Severity::Warning,
    },
    LinterRuleMetadata {
        group: "safety",
        name: "addingFieldWithDefault",
        recommended: true,
        severity: Severity::Warning,
    },
    LinterRuleMetadata {
        group: "safety",
        name: "addingForeignKeyConstraint",
        recommended: true,
        severity: Severity::Warning,
    },
    LinterRuleMetadata {
        group: "safety",
        name: "addingNotNullField",
        recommended: true,
        severity: Severity::Warning,
    },
    LinterRuleMetadata {
        group: "safety",
        name: "addingPrimaryKeyConstraint",
        recommended: true,
        severity: Severity::Warning,
    },
    LinterRuleMetadata {
        group: "destructive",
        name: "addingRequiredField",
        recommended: false,
        severity: Severity::Error,
    },
    LinterRuleMetadata {
        group: "typecheck",
        name: "ambiguousColumn",
        recommended: true,
        severity: Severity::Error,
    },
    LinterRuleMetadata {
        group: "typecheck",
        name: "assignmentTypeMismatch",
        recommended: true,
        severity: Severity::Error,
    },
    LinterRuleMetadata {
        group: "safety",
        name: "avoidAddingExclusionConstraint",
        recommended: true,
        severity: Severity::Warning,
    },
    LinterRuleMetadata {
        group: "correctness",
        name: "avoidAlterEnumAddValue",
        recommended: false,
        severity: Severity::Warning,
    },
    LinterRuleMetadata {
        group: "safety",
        name: "avoidAttachingPartition",
        recommended: true,
        severity: Severity::Warning,
    },
    LinterRuleMetadata {
        group: "safety",
        name: "avoidCreateTrigger",
        recommended: false,
        severity: Severity::Warning,
    },
    LinterRuleMetadata {
        group: "safety",
        name: "avoidEnableDisableTrigger",
        recommended: false,
        severity: Severity::Warning,
    },
    LinterRuleMetadata {
        group: "safety",
        name: "avoidWideLockWindow",
        recommended: true,
        severity: Severity::Warning,
    },
    LinterRuleMetadata {
        group: "style",
        name: "banCharField",
        recommended: false,
        severity: Severity::Warning,
    },
    LinterRuleMetadata {
        group: "correctness",
        name: "banConcurrentIndexCreationInTransaction",
        recommended: true,
        severity: Severity::Error,
    },
    LinterRuleMetadata {
        group: "destructive",
        name: "banDeleteWithoutWhere",
        recommended: true,
        severity: Severity::Warning,
    },
    LinterRuleMetadata {
        group: "destructive",
        name: "banDropColumn",
        recommended: true,
        severity: Severity::Warning,
    },
    LinterRuleMetadata {
        group: "destructive",
        name: "banDropDatabase",
        recommended: false,
        severity: Severity::Warning,
    },
    LinterRuleMetadata {
        group: "destructive",
        name: "banDropNotNull",
        recommended: true,
        severity: Severity::Warning,
    },
    LinterRuleMetadata {
        group: "destructive",
        name: "banDropSchema",
        recommended: true,
        severity: Severity::Error,
    },
    LinterRuleMetadata {
        group: "destructive",
        name: "banDropTable",
        recommended: true,
        severity: Severity::Warning,
    },
    LinterRuleMetadata {
        group: "safety",
        name: "banDropTrigger",
        recommended: false,
        severity: Severity::Warning,
    },
    LinterRuleMetadata {
        group: "destructive",
        name: "banTruncate",
        recommended: true,
        severity: Severity::Error,
    },
    LinterRuleMetadata {
        group: "destructive",
        name: "banTruncateCascade",
        recommended: false,
        severity: Severity::Error,
    },
    LinterRuleMetadata {
        group: "destructive",
        name: "banUpdateWithoutWhere",
        recommended: true,
        severity: Severity::Warning,
    },
    LinterRuleMetadata {
        group: "safety",
        name: "banVacuumFull",
        recommended: true,
        severity: Severity::Error,
    },
    LinterRuleMetadata {
        group: "destructive",
        name: "changingColumnType",
        recommended: false,
        severity: Severity::Warning,
    },
    LinterRuleMetadata {
        group: "safety",
        name: "constraintMissingNotValid",
        recommended: false,
        severity: Severity::Warning,
    },
    LinterRuleMetadata {
        group: "style",
        name: "creatingEnum",
        recommended: false,
        severity: Severity::Warning,
    },
    LinterRuleMetadata {
        group: "safety",
        name: "disallowUniqueConstraint",
        recommended: false,
        severity: Severity::Error,
    },
    LinterRuleMetadata {
        group: "typecheck",
        name: "functionArgumentMismatch",
        recommended: true,
        severity: Severity::Error,
    },
    LinterRuleMetadata {
        group: "typecheck",
        name: "functionReturnTypeMismatch",
        recommended: true,
        severity: Severity::Error,
    },
    LinterRuleMetadata {
        group: "typecheck",
        name: "insertColumnMismatch",
        recommended: true,
        severity: Severity::Error,
    },
    LinterRuleMetadata {
        group: "typecheck",
        name: "invalidCast",
        recommended: true,
        severity: Severity::Error,
    },
    LinterRuleMetadata {
        group: "correctness",
        name: "invalidDropTypeSignature",
        recommended: true,
        severity: Severity::Error,
    },
    LinterRuleMetadata {
        group: "safety",
        name: "lockTimeoutWarning",
        recommended: true,
        severity: Severity::Warning,
    },
    LinterRuleMetadata {
        group: "typecheck",
        name: "missingFromClauseEntry",
        recommended: true,
        severity: Severity::Error,
    },
    LinterRuleMetadata {
        group: "safety",
        name: "multipleAlterTable",
        recommended: true,
        severity: Severity::Warning,
    },
    LinterRuleMetadata {
        group: "typecheck",
        name: "operatorTypeMismatch",
        recommended: true,
        severity: Severity::Error,
    },
    LinterRuleMetadata {
        group: "style",
        name: "preferBigInt",
        recommended: false,
        severity: Severity::Warning,
    },
    LinterRuleMetadata {
        group: "style",
        name: "preferIdentity",
        recommended: false,
        severity: Severity::Warning,
    },
    LinterRuleMetadata {
        group: "style",
        name: "preferJsonb",
        recommended: false,
        severity: Severity::Warning,
    },
    LinterRuleMetadata {
        group: "safety",
        name: "preferRobustStmts",
        recommended: false,
        severity: Severity::Warning,
    },
    LinterRuleMetadata {
        group: "style",
        name: "preferTextField",
        recommended: false,
        severity: Severity::Warning,
    },
    LinterRuleMetadata {
        group: "style",
        name: "preferTimestamptz",
        recommended: false,
        severity: Severity::Warning,
    },
    LinterRuleMetadata {
        group: "destructive",
        name: "renamingColumn",
        recommended: false,
        severity: Severity::Warning,
    },
    LinterRuleMetadata {
        group: "destructive",
        name: "renamingTable",
        recommended: false,
        severity: Severity::Warning,
    },
    LinterRuleMetadata {
        group: "safety",
        name: "requireConcurrentDetachPartition",
        recommended: true,
        severity: Severity::Warning,
    },
    LinterRuleMetadata {
        group: "safety",
        name: "requireConcurrentIndexCreation",
        recommended: false,
        severity: Severity::Warning,
    },
    LinterRuleMetadata {
        group: "safety",
        name: "requireConcurrentIndexDeletion",
        recommended: false,
        severity: Severity::Warning,
    },
    LinterRuleMetadata {
        group: "safety",
        name: "requireConcurrentRefreshMatview",
        recommended: true,
        severity: Severity::Warning,
    },
    LinterRuleMetadata {
        group: "safety",
        name: "requireConcurrentReindex",
        recommended: true,
        severity: Severity::Warning,
    },
    LinterRuleMetadata {
        group: "safety",
        name: "requireIdleInTransactionTimeout",
        recommended: false,
        severity: Severity::Warning,
    },
    LinterRuleMetadata {
        group: "safety",
        name: "requireSeparateConstraintValidation",
        recommended: true,
        severity: Severity::Error,
    },
    LinterRuleMetadata {
        group: "safety",
        name: "requireStatementTimeout",
        recommended: false,
        severity: Severity::Warning,
    },
    LinterRuleMetadata {
        group: "safety",
        name: "runningStatementWhileHoldingAccessExclusive",
        recommended: true,
        severity: Severity::Warning,
    },
    LinterRuleMetadata {
        group: "correctness",
        name: "transactionNesting",
        recommended: false,
        severity: Severity::Warning,
    },
    LinterRuleMetadata {
        group: "typecheck",
        name: "unknownColumn",
        recommended: true,
        severity: Severity::Error,
    },
    LinterRuleMetadata {
        group: "typecheck",
        name: "unknownFunction",
        recommended: true,
        severity: Severity::Error,
    },
    LinterRuleMetadata {
        group: "typecheck",
        name: "unknownRelation",
        recommended: true,
        severity: Severity::Error,
    },
    LinterRuleMetadata {
        group: "typecheck",
        name: "unknownSchema",
        recommended: true,
        severity: Severity::Error,
    },
    LinterRuleMetadata {
        group: "typecheck",
        name: "unknownType",
        recommended: true,
        severity: Severity::Error,
    },
];
#[doc = r" All linter groups."]
pub const LINTER_GROUPS: &[&str] = &[
    "correctness",
    "safety",
    "destructive",
    "style",
    "typecheck",
    "nursery",
];
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
    #[doc = "An unqualified column name matches columns of more than one relation in scope."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ambiguous_column: Option<RuleConfiguration<pgls_analyser::options::AmbiguousColumn>>,
    #[doc = "An expression assigned to a column cannot be coerced to that column's type."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignment_type_mismatch:
        Option<RuleConfiguration<pgls_analyser::options::AssignmentTypeMismatch>>,
    #[doc = "Adding an exclusion constraint acquires an ACCESS EXCLUSIVE lock."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avoid_adding_exclusion_constraint:
        Option<RuleConfiguration<pgls_analyser::options::AvoidAddingExclusionConstraint>>,
    #[doc = "ALTER TYPE ... ADD VALUE cannot run inside a transaction block in older Postgres versions."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avoid_alter_enum_add_value:
        Option<RuleConfiguration<pgls_analyser::options::AvoidAlterEnumAddValue>>,
    #[doc = "Attaching a partition acquires an ACCESS EXCLUSIVE lock on the parent table."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avoid_attaching_partition:
        Option<RuleConfiguration<pgls_analyser::options::AvoidAttachingPartition>>,
    #[doc = "Creating a trigger acquires a SHARE ROW EXCLUSIVE lock on the table."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avoid_create_trigger: Option<RuleConfiguration<pgls_analyser::options::AvoidCreateTrigger>>,
    #[doc = "Enabling or disabling a trigger acquires a SHARE ROW EXCLUSIVE lock."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avoid_enable_disable_trigger:
        Option<RuleConfiguration<pgls_analyser::options::AvoidEnableDisableTrigger>>,
    #[doc = "Acquiring ACCESS EXCLUSIVE locks on multiple tables widens the lock window."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avoid_wide_lock_window:
        Option<RuleConfiguration<pgls_analyser::options::AvoidWideLockWindow>>,
    #[doc = "Using CHAR(n) or CHARACTER(n) types is discouraged."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ban_char_field: Option<RuleConfiguration<pgls_analyser::options::BanCharField>>,
    #[doc = "Concurrent index creation is not allowed within a transaction."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ban_concurrent_index_creation_in_transaction:
        Option<RuleConfiguration<pgls_analyser::options::BanConcurrentIndexCreationInTransaction>>,
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
    #[doc = "A function name and argument count exist, but its argument types do not select exactly one overload."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_argument_mismatch:
        Option<RuleConfiguration<pgls_analyser::options::FunctionArgumentMismatch>>,
    #[doc = "The final statement of a SQL function doesn't return what the function is declared to return."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_return_type_mismatch:
        Option<RuleConfiguration<pgls_analyser::options::FunctionReturnTypeMismatch>>,
    #[doc = "An INSERT has a different number of target columns than values."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insert_column_mismatch:
        Option<RuleConfiguration<pgls_analyser::options::InsertColumnMismatch>>,
    #[doc = "An explicit cast is not permitted between the source and target types."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invalid_cast: Option<RuleConfiguration<pgls_analyser::options::InvalidCast>>,
    #[doc = "DROP TYPE and DROP DOMAIN don't take a parameter list."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invalid_drop_type_signature:
        Option<RuleConfiguration<pgls_analyser::options::InvalidDropTypeSignature>>,
    #[doc = "Taking a dangerous lock without setting a lock timeout can cause indefinite blocking."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lock_timeout_warning: Option<RuleConfiguration<pgls_analyser::options::LockTimeoutWarning>>,
    #[doc = "A column is qualified with a name that is not in the FROM clause."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub missing_from_clause_entry:
        Option<RuleConfiguration<pgls_analyser::options::MissingFromClauseEntry>>,
    #[doc = "Multiple ALTER TABLE statements on the same table should be combined into a single statement."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiple_alter_table: Option<RuleConfiguration<pgls_analyser::options::MultipleAlterTable>>,
    #[doc = "An operator exists by name but cannot be resolved for the operand types, or has multiple equally suitable candidates."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator_type_mismatch:
        Option<RuleConfiguration<pgls_analyser::options::OperatorTypeMismatch>>,
    #[doc = "Prefer BIGINT over smaller integer types."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefer_big_int: Option<RuleConfiguration<pgls_analyser::options::PreferBigInt>>,
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
    #[doc = "REFRESH MATERIALIZED VIEW without CONCURRENTLY acquires an ACCESS EXCLUSIVE lock."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_concurrent_refresh_matview:
        Option<RuleConfiguration<pgls_analyser::options::RequireConcurrentRefreshMatview>>,
    #[doc = "REINDEX without CONCURRENTLY acquires an ACCESS EXCLUSIVE lock on the table."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_concurrent_reindex:
        Option<RuleConfiguration<pgls_analyser::options::RequireConcurrentReindex>>,
    #[doc = "Dangerous lock statements should be preceded by SET idle_in_transaction_session_timeout."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_idle_in_transaction_timeout:
        Option<RuleConfiguration<pgls_analyser::options::RequireIdleInTransactionTimeout>>,
    #[doc = "Validating a constraint in the same transaction it was added as NOT VALID defeats the purpose."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_separate_constraint_validation:
        Option<RuleConfiguration<pgls_analyser::options::RequireSeparateConstraintValidation>>,
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
    #[doc = "A column does not exist on the relation or record it is taken from."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_column: Option<RuleConfiguration<pgls_analyser::options::UnknownColumn>>,
    #[doc = "No function with this name accepts this number of arguments."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_function: Option<RuleConfiguration<pgls_analyser::options::UnknownFunction>>,
    #[doc = "A table, view, or materialized view does not exist."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_relation: Option<RuleConfiguration<pgls_analyser::options::UnknownRelation>>,
    #[doc = "A schema does not exist."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_schema: Option<RuleConfiguration<pgls_analyser::options::UnknownSchema>>,
    #[doc = "A type does not exist."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_type: Option<RuleConfiguration<pgls_analyser::options::UnknownType>>,
}
impl Rules {
    #[doc = r" The level configured for a rule, if any."]
    pub fn rule_level(&self, rule: &str) -> Option<RulePlainConfiguration> {
        match rule {
            "addSerialColumn" => self
                .add_serial_column
                .as_ref()
                .map(RuleConfiguration::level),
            "addingFieldWithDefault" => self
                .adding_field_with_default
                .as_ref()
                .map(RuleConfiguration::level),
            "addingForeignKeyConstraint" => self
                .adding_foreign_key_constraint
                .as_ref()
                .map(RuleConfiguration::level),
            "addingNotNullField" => self
                .adding_not_null_field
                .as_ref()
                .map(RuleConfiguration::level),
            "addingPrimaryKeyConstraint" => self
                .adding_primary_key_constraint
                .as_ref()
                .map(RuleConfiguration::level),
            "addingRequiredField" => self
                .adding_required_field
                .as_ref()
                .map(RuleConfiguration::level),
            "ambiguousColumn" => self.ambiguous_column.as_ref().map(RuleConfiguration::level),
            "assignmentTypeMismatch" => self
                .assignment_type_mismatch
                .as_ref()
                .map(RuleConfiguration::level),
            "avoidAddingExclusionConstraint" => self
                .avoid_adding_exclusion_constraint
                .as_ref()
                .map(RuleConfiguration::level),
            "avoidAlterEnumAddValue" => self
                .avoid_alter_enum_add_value
                .as_ref()
                .map(RuleConfiguration::level),
            "avoidAttachingPartition" => self
                .avoid_attaching_partition
                .as_ref()
                .map(RuleConfiguration::level),
            "avoidCreateTrigger" => self
                .avoid_create_trigger
                .as_ref()
                .map(RuleConfiguration::level),
            "avoidEnableDisableTrigger" => self
                .avoid_enable_disable_trigger
                .as_ref()
                .map(RuleConfiguration::level),
            "avoidWideLockWindow" => self
                .avoid_wide_lock_window
                .as_ref()
                .map(RuleConfiguration::level),
            "banCharField" => self.ban_char_field.as_ref().map(RuleConfiguration::level),
            "banConcurrentIndexCreationInTransaction" => self
                .ban_concurrent_index_creation_in_transaction
                .as_ref()
                .map(RuleConfiguration::level),
            "banDeleteWithoutWhere" => self
                .ban_delete_without_where
                .as_ref()
                .map(RuleConfiguration::level),
            "banDropColumn" => self.ban_drop_column.as_ref().map(RuleConfiguration::level),
            "banDropDatabase" => self
                .ban_drop_database
                .as_ref()
                .map(RuleConfiguration::level),
            "banDropNotNull" => self
                .ban_drop_not_null
                .as_ref()
                .map(RuleConfiguration::level),
            "banDropSchema" => self.ban_drop_schema.as_ref().map(RuleConfiguration::level),
            "banDropTable" => self.ban_drop_table.as_ref().map(RuleConfiguration::level),
            "banDropTrigger" => self.ban_drop_trigger.as_ref().map(RuleConfiguration::level),
            "banTruncate" => self.ban_truncate.as_ref().map(RuleConfiguration::level),
            "banTruncateCascade" => self
                .ban_truncate_cascade
                .as_ref()
                .map(RuleConfiguration::level),
            "banUpdateWithoutWhere" => self
                .ban_update_without_where
                .as_ref()
                .map(RuleConfiguration::level),
            "banVacuumFull" => self.ban_vacuum_full.as_ref().map(RuleConfiguration::level),
            "changingColumnType" => self
                .changing_column_type
                .as_ref()
                .map(RuleConfiguration::level),
            "constraintMissingNotValid" => self
                .constraint_missing_not_valid
                .as_ref()
                .map(RuleConfiguration::level),
            "creatingEnum" => self.creating_enum.as_ref().map(RuleConfiguration::level),
            "disallowUniqueConstraint" => self
                .disallow_unique_constraint
                .as_ref()
                .map(RuleConfiguration::level),
            "functionArgumentMismatch" => self
                .function_argument_mismatch
                .as_ref()
                .map(RuleConfiguration::level),
            "functionReturnTypeMismatch" => self
                .function_return_type_mismatch
                .as_ref()
                .map(RuleConfiguration::level),
            "insertColumnMismatch" => self
                .insert_column_mismatch
                .as_ref()
                .map(RuleConfiguration::level),
            "invalidCast" => self.invalid_cast.as_ref().map(RuleConfiguration::level),
            "invalidDropTypeSignature" => self
                .invalid_drop_type_signature
                .as_ref()
                .map(RuleConfiguration::level),
            "lockTimeoutWarning" => self
                .lock_timeout_warning
                .as_ref()
                .map(RuleConfiguration::level),
            "missingFromClauseEntry" => self
                .missing_from_clause_entry
                .as_ref()
                .map(RuleConfiguration::level),
            "multipleAlterTable" => self
                .multiple_alter_table
                .as_ref()
                .map(RuleConfiguration::level),
            "operatorTypeMismatch" => self
                .operator_type_mismatch
                .as_ref()
                .map(RuleConfiguration::level),
            "preferBigInt" => self.prefer_big_int.as_ref().map(RuleConfiguration::level),
            "preferIdentity" => self.prefer_identity.as_ref().map(RuleConfiguration::level),
            "preferJsonb" => self.prefer_jsonb.as_ref().map(RuleConfiguration::level),
            "preferRobustStmts" => self
                .prefer_robust_stmts
                .as_ref()
                .map(RuleConfiguration::level),
            "preferTextField" => self
                .prefer_text_field
                .as_ref()
                .map(RuleConfiguration::level),
            "preferTimestamptz" => self
                .prefer_timestamptz
                .as_ref()
                .map(RuleConfiguration::level),
            "renamingColumn" => self.renaming_column.as_ref().map(RuleConfiguration::level),
            "renamingTable" => self.renaming_table.as_ref().map(RuleConfiguration::level),
            "requireConcurrentDetachPartition" => self
                .require_concurrent_detach_partition
                .as_ref()
                .map(RuleConfiguration::level),
            "requireConcurrentIndexCreation" => self
                .require_concurrent_index_creation
                .as_ref()
                .map(RuleConfiguration::level),
            "requireConcurrentIndexDeletion" => self
                .require_concurrent_index_deletion
                .as_ref()
                .map(RuleConfiguration::level),
            "requireConcurrentRefreshMatview" => self
                .require_concurrent_refresh_matview
                .as_ref()
                .map(RuleConfiguration::level),
            "requireConcurrentReindex" => self
                .require_concurrent_reindex
                .as_ref()
                .map(RuleConfiguration::level),
            "requireIdleInTransactionTimeout" => self
                .require_idle_in_transaction_timeout
                .as_ref()
                .map(RuleConfiguration::level),
            "requireSeparateConstraintValidation" => self
                .require_separate_constraint_validation
                .as_ref()
                .map(RuleConfiguration::level),
            "requireStatementTimeout" => self
                .require_statement_timeout
                .as_ref()
                .map(RuleConfiguration::level),
            "runningStatementWhileHoldingAccessExclusive" => self
                .running_statement_while_holding_access_exclusive
                .as_ref()
                .map(RuleConfiguration::level),
            "transactionNesting" => self
                .transaction_nesting
                .as_ref()
                .map(RuleConfiguration::level),
            "unknownColumn" => self.unknown_column.as_ref().map(RuleConfiguration::level),
            "unknownFunction" => self.unknown_function.as_ref().map(RuleConfiguration::level),
            "unknownRelation" => self.unknown_relation.as_ref().map(RuleConfiguration::level),
            "unknownSchema" => self.unknown_schema.as_ref().map(RuleConfiguration::level),
            "unknownType" => self.unknown_type.as_ref().map(RuleConfiguration::level),
            _ => None,
        }
    }
    #[doc = r" The options configured for a rule, if any."]
    pub fn rule_options(&self, rule: &str) -> Option<RuleOptions> {
        match rule {
            "addSerialColumn" => self
                .add_serial_column
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "addingFieldWithDefault" => self
                .adding_field_with_default
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "addingForeignKeyConstraint" => self
                .adding_foreign_key_constraint
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "addingNotNullField" => self
                .adding_not_null_field
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "addingPrimaryKeyConstraint" => self
                .adding_primary_key_constraint
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "addingRequiredField" => self
                .adding_required_field
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "ambiguousColumn" => self
                .ambiguous_column
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "assignmentTypeMismatch" => self
                .assignment_type_mismatch
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "avoidAddingExclusionConstraint" => self
                .avoid_adding_exclusion_constraint
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "avoidAlterEnumAddValue" => self
                .avoid_alter_enum_add_value
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "avoidAttachingPartition" => self
                .avoid_attaching_partition
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "avoidCreateTrigger" => self
                .avoid_create_trigger
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "avoidEnableDisableTrigger" => self
                .avoid_enable_disable_trigger
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "avoidWideLockWindow" => self
                .avoid_wide_lock_window
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "banCharField" => self
                .ban_char_field
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "banConcurrentIndexCreationInTransaction" => self
                .ban_concurrent_index_creation_in_transaction
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "banDeleteWithoutWhere" => self
                .ban_delete_without_where
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "banDropColumn" => self
                .ban_drop_column
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "banDropDatabase" => self
                .ban_drop_database
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "banDropNotNull" => self
                .ban_drop_not_null
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "banDropSchema" => self
                .ban_drop_schema
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "banDropTable" => self
                .ban_drop_table
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "banDropTrigger" => self
                .ban_drop_trigger
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "banTruncate" => self
                .ban_truncate
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "banTruncateCascade" => self
                .ban_truncate_cascade
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "banUpdateWithoutWhere" => self
                .ban_update_without_where
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "banVacuumFull" => self
                .ban_vacuum_full
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "changingColumnType" => self
                .changing_column_type
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "constraintMissingNotValid" => self
                .constraint_missing_not_valid
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "creatingEnum" => self
                .creating_enum
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "disallowUniqueConstraint" => self
                .disallow_unique_constraint
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "functionArgumentMismatch" => self
                .function_argument_mismatch
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "functionReturnTypeMismatch" => self
                .function_return_type_mismatch
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "insertColumnMismatch" => self
                .insert_column_mismatch
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "invalidCast" => self
                .invalid_cast
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "invalidDropTypeSignature" => self
                .invalid_drop_type_signature
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "lockTimeoutWarning" => self
                .lock_timeout_warning
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "missingFromClauseEntry" => self
                .missing_from_clause_entry
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "multipleAlterTable" => self
                .multiple_alter_table
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "operatorTypeMismatch" => self
                .operator_type_mismatch
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "preferBigInt" => self
                .prefer_big_int
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "preferIdentity" => self
                .prefer_identity
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "preferJsonb" => self
                .prefer_jsonb
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "preferRobustStmts" => self
                .prefer_robust_stmts
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "preferTextField" => self
                .prefer_text_field
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "preferTimestamptz" => self
                .prefer_timestamptz
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "renamingColumn" => self
                .renaming_column
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "renamingTable" => self
                .renaming_table
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "requireConcurrentDetachPartition" => self
                .require_concurrent_detach_partition
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "requireConcurrentIndexCreation" => self
                .require_concurrent_index_creation
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "requireConcurrentIndexDeletion" => self
                .require_concurrent_index_deletion
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "requireConcurrentRefreshMatview" => self
                .require_concurrent_refresh_matview
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "requireConcurrentReindex" => self
                .require_concurrent_reindex
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "requireIdleInTransactionTimeout" => self
                .require_idle_in_transaction_timeout
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "requireSeparateConstraintValidation" => self
                .require_separate_constraint_validation
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "requireStatementTimeout" => self
                .require_statement_timeout
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "runningStatementWhileHoldingAccessExclusive" => self
                .running_statement_while_holding_access_exclusive
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "transactionNesting" => self
                .transaction_nesting
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "unknownColumn" => self
                .unknown_column
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "unknownFunction" => self
                .unknown_function
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "unknownRelation" => self
                .unknown_relation
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "unknownSchema" => self
                .unknown_schema
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            "unknownType" => self
                .unknown_type
                .as_ref()
                .and_then(RuleConfiguration::get_options),
            _ => None,
        }
    }
}
#[doc = r" The level of all rules of a group, unless a rule is configured individually."]
#[derive(Clone, Debug, Default, Deserialize, Eq, Merge, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "LinterGroups"))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct Groups {
    #[doc = "Code that fails at runtime for reasons other than names or types."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correctness: Option<RulePlainConfiguration>,
    #[doc = "Valid code that may be dangerous against a live database: locks, rewrites, or blocking."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety: Option<RulePlainConfiguration>,
    #[doc = "Code that loses data or breaks existing clients."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destructive: Option<RulePlainConfiguration>,
    #[doc = "Schema design preferences. Not enabled by the recommended preset."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<RulePlainConfiguration>,
    #[doc = "Code that fails at runtime because of names or types. Needs a database connection."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub typecheck: Option<RulePlainConfiguration>,
    #[doc = "New rules that are still being tested. Never enabled by presets."]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nursery: Option<RulePlainConfiguration>,
}
impl Groups {
    #[doc = r" The level configured for a group, if any."]
    pub fn level(&self, group: &str) -> Option<RulePlainConfiguration> {
        match group {
            "correctness" => self.correctness,
            "safety" => self.safety,
            "destructive" => self.destructive,
            "style" => self.style,
            "typecheck" => self.typecheck,
            "nursery" => self.nursery,
            _ => None,
        }
    }
}
