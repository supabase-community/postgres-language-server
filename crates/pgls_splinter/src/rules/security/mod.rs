//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
pub mod auth_users_exposed;
pub mod extension_in_public;
pub mod extension_versions_outdated;
pub mod fkey_to_auth_unique;
pub mod foreign_table_in_api;
pub mod function_search_path_mutable;
pub mod insecure_queue_exposed_in_api;
pub mod materialized_view_in_api;
pub mod policy_exists_rls_disabled;
pub mod rls_disabled_in_public;
pub mod rls_enabled_no_policy;
pub mod rls_references_user_metadata;
pub mod security_definer_view;
pub mod unsupported_reg_types;
::pgls_analyse::declare_lint_group! { pub Security { name : "security" , rules : [self :: auth_users_exposed :: AuthUsersExposed , self :: extension_in_public :: ExtensionInPublic , self :: extension_versions_outdated :: ExtensionVersionsOutdated , self :: fkey_to_auth_unique :: FkeyToAuthUnique , self :: foreign_table_in_api :: ForeignTableInApi , self :: function_search_path_mutable :: FunctionSearchPathMutable , self :: insecure_queue_exposed_in_api :: InsecureQueueExposedInApi , self :: materialized_view_in_api :: MaterializedViewInApi , self :: policy_exists_rls_disabled :: PolicyExistsRlsDisabled , self :: rls_disabled_in_public :: RlsDisabledInPublic , self :: rls_enabled_no_policy :: RlsEnabledNoPolicy , self :: rls_references_user_metadata :: RlsReferencesUserMetadata , self :: security_definer_view :: SecurityDefinerView , self :: unsupported_reg_types :: UnsupportedRegTypes ,] } }
