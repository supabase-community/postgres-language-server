//! Generated file, do not edit by hand, see `xtask/codegen`

use crate::lint;
pub type AddSerialColumn =
    <lint::safety::add_serial_column::AddSerialColumn as crate::LinterRule>::Options;
pub type AddingFieldWithDefault =
    <lint::safety::adding_field_with_default::AddingFieldWithDefault as crate::LinterRule>::Options;
pub type AddingForeignKeyConstraint = < lint :: safety :: adding_foreign_key_constraint :: AddingForeignKeyConstraint as crate :: LinterRule > :: Options ;
pub type AddingNotNullField =
    <lint::safety::adding_not_null_field::AddingNotNullField as crate::LinterRule>::Options;
pub type AddingPrimaryKeyConstraint = < lint :: safety :: adding_primary_key_constraint :: AddingPrimaryKeyConstraint as crate :: LinterRule > :: Options ;
pub type AddingRequiredField =
    <lint::destructive::adding_required_field::AddingRequiredField as crate::LinterRule>::Options;
pub type AmbiguousColumn =
    <lint::typecheck::ambiguous_column::AmbiguousColumn as crate::LinterRule>::Options;
pub type AssignmentTypeMismatch = < lint :: typecheck :: assignment_type_mismatch :: AssignmentTypeMismatch as crate :: LinterRule > :: Options ;
pub type AvoidAddingExclusionConstraint = < lint :: safety :: avoid_adding_exclusion_constraint :: AvoidAddingExclusionConstraint as crate :: LinterRule > :: Options ;
pub type AvoidAlterEnumAddValue = < lint :: correctness :: avoid_alter_enum_add_value :: AvoidAlterEnumAddValue as crate :: LinterRule > :: Options ;
pub type AvoidAttachingPartition = < lint :: safety :: avoid_attaching_partition :: AvoidAttachingPartition as crate :: LinterRule > :: Options ;
pub type AvoidCreateTrigger =
    <lint::safety::avoid_create_trigger::AvoidCreateTrigger as crate::LinterRule>::Options;
pub type AvoidEnableDisableTrigger = < lint :: safety :: avoid_enable_disable_trigger :: AvoidEnableDisableTrigger as crate :: LinterRule > :: Options ;
pub type AvoidWideLockWindow =
    <lint::safety::avoid_wide_lock_window::AvoidWideLockWindow as crate::LinterRule>::Options;
pub type BanCharField = <lint::style::ban_char_field::BanCharField as crate::LinterRule>::Options;
pub type BanConcurrentIndexCreationInTransaction = < lint :: correctness :: ban_concurrent_index_creation_in_transaction :: BanConcurrentIndexCreationInTransaction as crate :: LinterRule > :: Options ;
pub type BanDeleteWithoutWhere = < lint :: destructive :: ban_delete_without_where :: BanDeleteWithoutWhere as crate :: LinterRule > :: Options ;
pub type BanDropColumn =
    <lint::destructive::ban_drop_column::BanDropColumn as crate::LinterRule>::Options;
pub type BanDropDatabase =
    <lint::destructive::ban_drop_database::BanDropDatabase as crate::LinterRule>::Options;
pub type BanDropNotNull =
    <lint::destructive::ban_drop_not_null::BanDropNotNull as crate::LinterRule>::Options;
pub type BanDropSchema =
    <lint::destructive::ban_drop_schema::BanDropSchema as crate::LinterRule>::Options;
pub type BanDropTable =
    <lint::destructive::ban_drop_table::BanDropTable as crate::LinterRule>::Options;
pub type BanDropTrigger =
    <lint::safety::ban_drop_trigger::BanDropTrigger as crate::LinterRule>::Options;
pub type BanTruncate = <lint::destructive::ban_truncate::BanTruncate as crate::LinterRule>::Options;
pub type BanTruncateCascade =
    <lint::destructive::ban_truncate_cascade::BanTruncateCascade as crate::LinterRule>::Options;
pub type BanUpdateWithoutWhere = < lint :: destructive :: ban_update_without_where :: BanUpdateWithoutWhere as crate :: LinterRule > :: Options ;
pub type BanVacuumFull =
    <lint::safety::ban_vacuum_full::BanVacuumFull as crate::LinterRule>::Options;
pub type ChangingColumnType =
    <lint::destructive::changing_column_type::ChangingColumnType as crate::LinterRule>::Options;
pub type ConstraintMissingNotValid = < lint :: safety :: constraint_missing_not_valid :: ConstraintMissingNotValid as crate :: LinterRule > :: Options ;
pub type CreatingEnum = <lint::style::creating_enum::CreatingEnum as crate::LinterRule>::Options;
pub type DisallowUniqueConstraint = < lint :: safety :: disallow_unique_constraint :: DisallowUniqueConstraint as crate :: LinterRule > :: Options ;
pub type FunctionArgumentMismatch = < lint :: typecheck :: function_argument_mismatch :: FunctionArgumentMismatch as crate :: LinterRule > :: Options ;
pub type FunctionReturnTypeMismatch = < lint :: typecheck :: function_return_type_mismatch :: FunctionReturnTypeMismatch as crate :: LinterRule > :: Options ;
pub type InsertColumnMismatch =
    <lint::typecheck::insert_column_mismatch::InsertColumnMismatch as crate::LinterRule>::Options;
pub type InvalidCast = <lint::typecheck::invalid_cast::InvalidCast as crate::LinterRule>::Options;
pub type InvalidDropTypeSignature = < lint :: correctness :: invalid_drop_type_signature :: InvalidDropTypeSignature as crate :: LinterRule > :: Options ;
pub type LockTimeoutWarning =
    <lint::safety::lock_timeout_warning::LockTimeoutWarning as crate::LinterRule>::Options;
pub type MissingFromClauseEntry = < lint :: typecheck :: missing_from_clause_entry :: MissingFromClauseEntry as crate :: LinterRule > :: Options ;
pub type MultipleAlterTable =
    <lint::safety::multiple_alter_table::MultipleAlterTable as crate::LinterRule>::Options;
pub type OperatorTypeMismatch =
    <lint::typecheck::operator_type_mismatch::OperatorTypeMismatch as crate::LinterRule>::Options;
pub type PreferBigInt = <lint::style::prefer_big_int::PreferBigInt as crate::LinterRule>::Options;
pub type PreferIdentity =
    <lint::style::prefer_identity::PreferIdentity as crate::LinterRule>::Options;
pub type PreferJsonb = <lint::style::prefer_jsonb::PreferJsonb as crate::LinterRule>::Options;
pub type PreferRobustStmts =
    <lint::safety::prefer_robust_stmts::PreferRobustStmts as crate::LinterRule>::Options;
pub type PreferTextField =
    <lint::style::prefer_text_field::PreferTextField as crate::LinterRule>::Options;
pub type PreferTimestamptz =
    <lint::style::prefer_timestamptz::PreferTimestamptz as crate::LinterRule>::Options;
pub type RenamingColumn =
    <lint::destructive::renaming_column::RenamingColumn as crate::LinterRule>::Options;
pub type RenamingTable =
    <lint::destructive::renaming_table::RenamingTable as crate::LinterRule>::Options;
pub type RequireConcurrentDetachPartition = < lint :: safety :: require_concurrent_detach_partition :: RequireConcurrentDetachPartition as crate :: LinterRule > :: Options ;
pub type RequireConcurrentIndexCreation = < lint :: safety :: require_concurrent_index_creation :: RequireConcurrentIndexCreation as crate :: LinterRule > :: Options ;
pub type RequireConcurrentIndexDeletion = < lint :: safety :: require_concurrent_index_deletion :: RequireConcurrentIndexDeletion as crate :: LinterRule > :: Options ;
pub type RequireConcurrentRefreshMatview = < lint :: safety :: require_concurrent_refresh_matview :: RequireConcurrentRefreshMatview as crate :: LinterRule > :: Options ;
pub type RequireConcurrentReindex = < lint :: safety :: require_concurrent_reindex :: RequireConcurrentReindex as crate :: LinterRule > :: Options ;
pub type RequireIdleInTransactionTimeout = < lint :: safety :: require_idle_in_transaction_timeout :: RequireIdleInTransactionTimeout as crate :: LinterRule > :: Options ;
pub type RequireSeparateConstraintValidation = < lint :: safety :: require_separate_constraint_validation :: RequireSeparateConstraintValidation as crate :: LinterRule > :: Options ;
pub type RequireStatementTimeout = < lint :: safety :: require_statement_timeout :: RequireStatementTimeout as crate :: LinterRule > :: Options ;
pub type RunningStatementWhileHoldingAccessExclusive = < lint :: safety :: running_statement_while_holding_access_exclusive :: RunningStatementWhileHoldingAccessExclusive as crate :: LinterRule > :: Options ;
pub type TransactionNesting =
    <lint::correctness::transaction_nesting::TransactionNesting as crate::LinterRule>::Options;
pub type UnknownColumn =
    <lint::typecheck::unknown_column::UnknownColumn as crate::LinterRule>::Options;
pub type UnknownFunction =
    <lint::typecheck::unknown_function::UnknownFunction as crate::LinterRule>::Options;
pub type UnknownRelation =
    <lint::typecheck::unknown_relation::UnknownRelation as crate::LinterRule>::Options;
pub type UnknownSchema =
    <lint::typecheck::unknown_schema::UnknownSchema as crate::LinterRule>::Options;
pub type UnknownType = <lint::typecheck::unknown_type::UnknownType as crate::LinterRule>::Options;
