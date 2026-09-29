//! Generated file, do not edit by hand, see `xtask/codegen`

use pgls_analyse::declare_lint_group;
pub mod add_serial_column;
pub mod adding_field_with_default;
pub mod adding_foreign_key_constraint;
pub mod adding_not_null_field;
pub mod adding_primary_key_constraint;
pub mod avoid_adding_exclusion_constraint;
pub mod avoid_attaching_partition;
pub mod avoid_create_trigger;
pub mod avoid_enable_disable_trigger;
pub mod avoid_wide_lock_window;
pub mod ban_drop_trigger;
pub mod ban_vacuum_full;
pub mod constraint_missing_not_valid;
pub mod disallow_unique_constraint;
pub mod lock_timeout_warning;
pub mod multiple_alter_table;
pub mod prefer_robust_stmts;
pub mod require_concurrent_detach_partition;
pub mod require_concurrent_index_creation;
pub mod require_concurrent_index_deletion;
pub mod require_concurrent_refresh_matview;
pub mod require_concurrent_reindex;
pub mod require_idle_in_transaction_timeout;
pub mod require_separate_constraint_validation;
pub mod require_statement_timeout;
pub mod running_statement_while_holding_access_exclusive;
declare_lint_group! { pub Safety { name : "safety" , rules : [self :: add_serial_column :: AddSerialColumn , self :: adding_field_with_default :: AddingFieldWithDefault , self :: adding_foreign_key_constraint :: AddingForeignKeyConstraint , self :: adding_not_null_field :: AddingNotNullField , self :: adding_primary_key_constraint :: AddingPrimaryKeyConstraint , self :: avoid_adding_exclusion_constraint :: AvoidAddingExclusionConstraint , self :: avoid_attaching_partition :: AvoidAttachingPartition , self :: avoid_create_trigger :: AvoidCreateTrigger , self :: avoid_enable_disable_trigger :: AvoidEnableDisableTrigger , self :: avoid_wide_lock_window :: AvoidWideLockWindow , self :: ban_drop_trigger :: BanDropTrigger , self :: ban_vacuum_full :: BanVacuumFull , self :: constraint_missing_not_valid :: ConstraintMissingNotValid , self :: disallow_unique_constraint :: DisallowUniqueConstraint , self :: lock_timeout_warning :: LockTimeoutWarning , self :: multiple_alter_table :: MultipleAlterTable , self :: prefer_robust_stmts :: PreferRobustStmts , self :: require_concurrent_detach_partition :: RequireConcurrentDetachPartition , self :: require_concurrent_index_creation :: RequireConcurrentIndexCreation , self :: require_concurrent_index_deletion :: RequireConcurrentIndexDeletion , self :: require_concurrent_refresh_matview :: RequireConcurrentRefreshMatview , self :: require_concurrent_reindex :: RequireConcurrentReindex , self :: require_idle_in_transaction_timeout :: RequireIdleInTransactionTimeout , self :: require_separate_constraint_validation :: RequireSeparateConstraintValidation , self :: require_statement_timeout :: RequireStatementTimeout , self :: running_statement_while_holding_access_exclusive :: RunningStatementWhileHoldingAccessExclusive ,] } }
