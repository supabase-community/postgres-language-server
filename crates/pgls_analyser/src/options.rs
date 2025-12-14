//! Generated file, do not edit by hand, see `xtask/codegen`

use crate::lint;
pub type AddSerialColumn =
    <lint::safety::add_serial_column::AddSerialColumn as crate::LinterRule>::Options;
pub type AddingFieldWithDefault =
    <lint::safety::adding_field_with_default::AddingFieldWithDefault as crate::LinterRule>::Options;
pub type AddingForeignKeyConstraint = < lint :: safety :: adding_foreign_key_constraint :: AddingForeignKeyConstraint as crate::LinterRule > :: Options ;
pub type AddingNotNullField =
    <lint::safety::adding_not_null_field::AddingNotNullField as crate::LinterRule>::Options;
pub type AddingPrimaryKeyConstraint = < lint :: safety :: adding_primary_key_constraint :: AddingPrimaryKeyConstraint as crate::LinterRule > :: Options ;
pub type AddingRequiredField =
    <lint::safety::adding_required_field::AddingRequiredField as crate::LinterRule>::Options;
pub type BanCharField = <lint::safety::ban_char_field::BanCharField as crate::LinterRule>::Options;
pub type BanConcurrentIndexCreationInTransaction = < lint :: safety :: ban_concurrent_index_creation_in_transaction :: BanConcurrentIndexCreationInTransaction as crate::LinterRule > :: Options ;
pub type BanDropColumn =
    <lint::safety::ban_drop_column::BanDropColumn as crate::LinterRule>::Options;
pub type BanDropDatabase =
    <lint::safety::ban_drop_database::BanDropDatabase as crate::LinterRule>::Options;
pub type BanDropNotNull =
    <lint::safety::ban_drop_not_null::BanDropNotNull as crate::LinterRule>::Options;
pub type BanDropTable = <lint::safety::ban_drop_table::BanDropTable as crate::LinterRule>::Options;
pub type BanTruncateCascade =
    <lint::safety::ban_truncate_cascade::BanTruncateCascade as crate::LinterRule>::Options;
pub type ChangingColumnType =
    <lint::safety::changing_column_type::ChangingColumnType as crate::LinterRule>::Options;
pub type ConstraintMissingNotValid = < lint :: safety :: constraint_missing_not_valid :: ConstraintMissingNotValid as crate::LinterRule > :: Options ;
pub type CreatingEnum = <lint::safety::creating_enum::CreatingEnum as crate::LinterRule>::Options;
pub type DisallowUniqueConstraint = < lint :: safety :: disallow_unique_constraint :: DisallowUniqueConstraint as crate::LinterRule > :: Options ;
pub type LockTimeoutWarning =
    <lint::safety::lock_timeout_warning::LockTimeoutWarning as crate::LinterRule>::Options;
pub type MultipleAlterTable =
    <lint::safety::multiple_alter_table::MultipleAlterTable as crate::LinterRule>::Options;
pub type PreferBigInt = <lint::safety::prefer_big_int::PreferBigInt as crate::LinterRule>::Options;
pub type PreferBigintOverInt =
    <lint::safety::prefer_bigint_over_int::PreferBigintOverInt as crate::LinterRule>::Options;
pub type PreferBigintOverSmallint = < lint :: safety :: prefer_bigint_over_smallint :: PreferBigintOverSmallint as crate::LinterRule > :: Options ;
pub type PreferIdentity =
    <lint::safety::prefer_identity::PreferIdentity as crate::LinterRule>::Options;
pub type PreferJsonb = <lint::safety::prefer_jsonb::PreferJsonb as crate::LinterRule>::Options;
pub type PreferRobustStmts =
    <lint::safety::prefer_robust_stmts::PreferRobustStmts as crate::LinterRule>::Options;
pub type PreferTextField =
    <lint::safety::prefer_text_field::PreferTextField as crate::LinterRule>::Options;
pub type PreferTimestamptz =
    <lint::safety::prefer_timestamptz::PreferTimestamptz as crate::LinterRule>::Options;
pub type RenamingColumn =
    <lint::safety::renaming_column::RenamingColumn as crate::LinterRule>::Options;
pub type RenamingTable =
    <lint::safety::renaming_table::RenamingTable as crate::LinterRule>::Options;
pub type RequireConcurrentIndexCreation = < lint :: safety :: require_concurrent_index_creation :: RequireConcurrentIndexCreation as crate::LinterRule > :: Options ;
pub type RequireConcurrentIndexDeletion = < lint :: safety :: require_concurrent_index_deletion :: RequireConcurrentIndexDeletion as crate::LinterRule > :: Options ;
pub type RunningStatementWhileHoldingAccessExclusive = < lint :: safety :: running_statement_while_holding_access_exclusive :: RunningStatementWhileHoldingAccessExclusive as crate::LinterRule > :: Options ;
pub type TransactionNesting =
    <lint::safety::transaction_nesting::TransactionNesting as crate::LinterRule>::Options;
