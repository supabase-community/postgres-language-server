//! Generated file, do not edit by hand, see `xtask/codegen`

use crate::lint;
pub type AddingFieldWithDefault =
    <lint::safety::adding_field_with_default::AddingFieldWithDefault as pgt_analyse::Rule>::Options;
pub type AddingForeignKeyConstraint = < lint :: safety :: adding_foreign_key_constraint :: AddingForeignKeyConstraint as pgt_analyse :: Rule > :: Options ;
pub type AddingNotNullField =
    <lint::safety::adding_not_null_field::AddingNotNullField as pgt_analyse::Rule>::Options;
pub type AddingPrimaryKeyConstraint = < lint :: safety :: adding_primary_key_constraint :: AddingPrimaryKeyConstraint as pgt_analyse :: Rule > :: Options ;
pub type AddingRequiredField =
    <lint::safety::adding_required_field::AddingRequiredField as pgt_analyse::Rule>::Options;
pub type BanCharField = <lint::safety::ban_char_field::BanCharField as pgt_analyse::Rule>::Options;
pub type BanConcurrentIndexCreationInTransaction = < lint :: safety :: ban_concurrent_index_creation_in_transaction :: BanConcurrentIndexCreationInTransaction as pgt_analyse :: Rule > :: Options ;
pub type BanDropColumn =
    <lint::safety::ban_drop_column::BanDropColumn as pgt_analyse::Rule>::Options;
pub type BanDropDatabase =
    <lint::safety::ban_drop_database::BanDropDatabase as pgt_analyse::Rule>::Options;
pub type BanDropNotNull =
    <lint::safety::ban_drop_not_null::BanDropNotNull as pgt_analyse::Rule>::Options;
pub type BanDropTable = <lint::safety::ban_drop_table::BanDropTable as pgt_analyse::Rule>::Options;
pub type BanTruncateCascade =
    <lint::safety::ban_truncate_cascade::BanTruncateCascade as pgt_analyse::Rule>::Options;
pub type ChangingColumnType =
    <lint::safety::changing_column_type::ChangingColumnType as pgt_analyse::Rule>::Options;
pub type ConstraintMissingNotValid = < lint :: safety :: constraint_missing_not_valid :: ConstraintMissingNotValid as pgt_analyse :: Rule > :: Options ;
pub type DisallowUniqueConstraint = < lint :: safety :: disallow_unique_constraint :: DisallowUniqueConstraint as pgt_analyse :: Rule > :: Options ;
pub type PreferBigInt = <lint::safety::prefer_big_int::PreferBigInt as pgt_analyse::Rule>::Options;
pub type PreferBigintOverInt =
    <lint::safety::prefer_bigint_over_int::PreferBigintOverInt as pgt_analyse::Rule>::Options;
pub type PreferBigintOverSmallint = < lint :: safety :: prefer_bigint_over_smallint :: PreferBigintOverSmallint as pgt_analyse :: Rule > :: Options ;
pub type PreferIdentity =
    <lint::safety::prefer_identity::PreferIdentity as pgt_analyse::Rule>::Options;
pub type PreferJsonb = <lint::safety::prefer_jsonb::PreferJsonb as pgt_analyse::Rule>::Options;
pub type PreferRobustStmts =
    <lint::safety::prefer_robust_stmts::PreferRobustStmts as pgt_analyse::Rule>::Options;
pub type PreferTextField =
    <lint::safety::prefer_text_field::PreferTextField as pgt_analyse::Rule>::Options;
pub type PreferTimestamptz =
    <lint::safety::prefer_timestamptz::PreferTimestamptz as pgt_analyse::Rule>::Options;
pub type RenamingColumn =
    <lint::safety::renaming_column::RenamingColumn as pgt_analyse::Rule>::Options;
pub type RenamingTable =
    <lint::safety::renaming_table::RenamingTable as pgt_analyse::Rule>::Options;
pub type RequireConcurrentIndexCreation = < lint :: safety :: require_concurrent_index_creation :: RequireConcurrentIndexCreation as pgt_analyse :: Rule > :: Options ;
pub type RequireConcurrentIndexDeletion = < lint :: safety :: require_concurrent_index_deletion :: RequireConcurrentIndexDeletion as pgt_analyse :: Rule > :: Options ;
pub type TransactionNesting =
    <lint::safety::transaction_nesting::TransactionNesting as pgt_analyse::Rule>::Options;
