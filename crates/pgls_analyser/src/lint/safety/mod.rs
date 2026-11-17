//! Generated file, do not edit by hand, see `xtask/codegen`

use pgls_analyse::declare_lint_group;
pub(crate) mod add_serial_column;
pub(crate) mod adding_field_with_default;
pub(crate) mod adding_foreign_key_constraint;
pub(crate) mod adding_not_null_field;
pub(crate) mod adding_primary_key_constraint;
pub(crate) mod avoid_adding_exclusion_constraint;
pub(crate) mod avoid_attaching_partition;
pub(crate) mod avoid_create_trigger;
pub(crate) mod avoid_enable_disable_trigger;
pub(crate) mod avoid_wide_lock_window;
pub(crate) mod ban_alter_domain_with_add_constraint;
pub(crate) mod ban_create_domain_with_constraint;
pub(crate) mod ban_drop_trigger;
pub(crate) mod ban_vacuum_full;
pub(crate) mod constraint_missing_not_valid;
pub(crate) mod disallow_unique_constraint;
pub(crate) mod lock_timeout_warning;
pub(crate) mod multiple_alter_table;
pub(crate) mod prefer_robust_stmts;
pub(crate) mod require_concurrent_detach_partition;
pub(crate) mod require_concurrent_index_creation;
pub(crate) mod require_concurrent_index_deletion;
pub(crate) mod require_concurrent_refresh_matview;
pub(crate) mod require_concurrent_reindex;
pub(crate) mod require_idle_in_transaction_timeout;
pub(crate) mod require_separate_constraint_validation;
pub(crate) mod require_statement_timeout;
pub(crate) mod running_statement_while_holding_access_exclusive;
declare_lint_group! { pub Safety { name : "safety" , rules : [self :: add_serial_column :: AddSerialColumn , self :: adding_field_with_default :: AddingFieldWithDefault , self :: adding_foreign_key_constraint :: AddingForeignKeyConstraint , self :: adding_not_null_field :: AddingNotNullField , self :: adding_primary_key_constraint :: AddingPrimaryKeyConstraint , self :: avoid_adding_exclusion_constraint :: AvoidAddingExclusionConstraint , self :: avoid_attaching_partition :: AvoidAttachingPartition , self :: avoid_create_trigger :: AvoidCreateTrigger , self :: avoid_enable_disable_trigger :: AvoidEnableDisableTrigger , self :: avoid_wide_lock_window :: AvoidWideLockWindow , self :: ban_alter_domain_with_add_constraint :: BanAlterDomainWithAddConstraint , self :: ban_create_domain_with_constraint :: BanCreateDomainWithConstraint , self :: ban_drop_trigger :: BanDropTrigger , self :: ban_vacuum_full :: BanVacuumFull , self :: constraint_missing_not_valid :: ConstraintMissingNotValid , self :: disallow_unique_constraint :: DisallowUniqueConstraint , self :: lock_timeout_warning :: LockTimeoutWarning , self :: multiple_alter_table :: MultipleAlterTable , self :: prefer_robust_stmts :: PreferRobustStmts , self :: require_concurrent_detach_partition :: RequireConcurrentDetachPartition , self :: require_concurrent_index_creation :: RequireConcurrentIndexCreation , self :: require_concurrent_index_deletion :: RequireConcurrentIndexDeletion , self :: require_concurrent_refresh_matview :: RequireConcurrentRefreshMatview , self :: require_concurrent_reindex :: RequireConcurrentReindex , self :: require_idle_in_transaction_timeout :: RequireIdleInTransactionTimeout , self :: require_separate_constraint_validation :: RequireSeparateConstraintValidation , self :: require_statement_timeout :: RequireStatementTimeout , self :: running_statement_while_holding_access_exclusive :: RunningStatementWhileHoldingAccessExclusive ,] } }
