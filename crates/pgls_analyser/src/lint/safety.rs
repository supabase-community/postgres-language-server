//! Generated file, do not edit by hand, see `xtask/codegen`

use pgls_analyse::declare_lint_group;
pub mod add_serial_column;
pub mod adding_field_with_default;
pub mod adding_foreign_key_constraint;
pub mod adding_not_null_field;
pub mod adding_primary_key_constraint;
pub mod adding_required_field;
pub mod ban_char_field;
pub mod ban_concurrent_index_creation_in_transaction;
pub mod ban_drop_column;
pub mod ban_drop_database;
pub mod ban_drop_not_null;
pub mod ban_drop_table;
pub mod ban_truncate_cascade;
pub mod changing_column_type;
pub mod constraint_missing_not_valid;
pub mod creating_enum;
pub mod disallow_unique_constraint;
pub mod lock_timeout_warning;
pub mod multiple_alter_table;
pub mod prefer_big_int;
pub mod prefer_bigint_over_int;
pub mod prefer_bigint_over_smallint;
pub mod prefer_identity;
pub mod prefer_jsonb;
pub mod prefer_robust_stmts;
pub mod prefer_text_field;
pub mod prefer_timestamptz;
pub mod renaming_column;
pub mod renaming_table;
pub mod require_concurrent_index_creation;
pub mod require_concurrent_index_deletion;
pub mod running_statement_while_holding_access_exclusive;
pub mod transaction_nesting;
declare_lint_group! { pub Safety { name : "safety" , rules : [self :: add_serial_column :: AddSerialColumn , self :: adding_field_with_default :: AddingFieldWithDefault , self :: adding_foreign_key_constraint :: AddingForeignKeyConstraint , self :: adding_not_null_field :: AddingNotNullField , self :: adding_primary_key_constraint :: AddingPrimaryKeyConstraint , self :: adding_required_field :: AddingRequiredField , self :: ban_char_field :: BanCharField , self :: ban_concurrent_index_creation_in_transaction :: BanConcurrentIndexCreationInTransaction , self :: ban_drop_column :: BanDropColumn , self :: ban_drop_database :: BanDropDatabase , self :: ban_drop_not_null :: BanDropNotNull , self :: ban_drop_table :: BanDropTable , self :: ban_truncate_cascade :: BanTruncateCascade , self :: changing_column_type :: ChangingColumnType , self :: constraint_missing_not_valid :: ConstraintMissingNotValid , self :: creating_enum :: CreatingEnum , self :: disallow_unique_constraint :: DisallowUniqueConstraint , self :: lock_timeout_warning :: LockTimeoutWarning , self :: multiple_alter_table :: MultipleAlterTable , self :: prefer_big_int :: PreferBigInt , self :: prefer_bigint_over_int :: PreferBigintOverInt , self :: prefer_bigint_over_smallint :: PreferBigintOverSmallint , self :: prefer_identity :: PreferIdentity , self :: prefer_jsonb :: PreferJsonb , self :: prefer_robust_stmts :: PreferRobustStmts , self :: prefer_text_field :: PreferTextField , self :: prefer_timestamptz :: PreferTimestamptz , self :: renaming_column :: RenamingColumn , self :: renaming_table :: RenamingTable , self :: require_concurrent_index_creation :: RequireConcurrentIndexCreation , self :: require_concurrent_index_deletion :: RequireConcurrentIndexDeletion , self :: running_statement_while_holding_access_exclusive :: RunningStatementWhileHoldingAccessExclusive , self :: transaction_nesting :: TransactionNesting ,] } }
