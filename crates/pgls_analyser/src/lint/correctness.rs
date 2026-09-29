//! Generated file, do not edit by hand, see `xtask/codegen`

use pgls_analyse::declare_lint_group;
pub mod avoid_alter_enum_add_value;
pub mod ban_concurrent_index_creation_in_transaction;
pub mod invalid_drop_type_signature;
pub mod transaction_nesting;
declare_lint_group! { pub Correctness { name : "correctness" , rules : [self :: avoid_alter_enum_add_value :: AvoidAlterEnumAddValue , self :: ban_concurrent_index_creation_in_transaction :: BanConcurrentIndexCreationInTransaction , self :: invalid_drop_type_signature :: InvalidDropTypeSignature , self :: transaction_nesting :: TransactionNesting ,] } }
