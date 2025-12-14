//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
pub mod auth_rls_initplan;
pub mod duplicate_index;
pub mod multiple_permissive_policies;
pub mod no_primary_key;
pub mod table_bloat;
pub mod unindexed_foreign_keys;
pub mod unused_index;
::pgls_analyse::declare_lint_group! { pub Performance { name : "performance" , rules : [self :: auth_rls_initplan :: AuthRlsInitplan , self :: duplicate_index :: DuplicateIndex , self :: multiple_permissive_policies :: MultiplePermissivePolicies , self :: no_primary_key :: NoPrimaryKey , self :: table_bloat :: TableBloat , self :: unindexed_foreign_keys :: UnindexedForeignKeys , self :: unused_index :: UnusedIndex ,] } }
