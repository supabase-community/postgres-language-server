//! Generated file, do not edit by hand, see `xtask/codegen`

use pgls_analyse::declare_lint_group;
pub(crate) mod avoid_alter_enum_add_value;
pub(crate) mod ban_concurrent_index_creation_in_transaction;
pub(crate) mod ban_duplicate_column_assignments;
pub(crate) mod ban_non_transactional_statement_in_transaction;
pub(crate) mod ban_uncommitted_transaction;
pub(crate) mod identifier_too_long;
pub(crate) mod invalid_drop_type_signature;
pub(crate) mod transaction_nesting;
declare_lint_group! { pub Correctness { name : "correctness" , rules : [self :: avoid_alter_enum_add_value :: AvoidAlterEnumAddValue , self :: ban_concurrent_index_creation_in_transaction :: BanConcurrentIndexCreationInTransaction , self :: ban_duplicate_column_assignments :: BanDuplicateColumnAssignments , self :: ban_non_transactional_statement_in_transaction :: BanNonTransactionalStatementInTransaction , self :: ban_uncommitted_transaction :: BanUncommittedTransaction , self :: identifier_too_long :: IdentifierTooLong , self :: invalid_drop_type_signature :: InvalidDropTypeSignature , self :: transaction_nesting :: TransactionNesting ,] } }
