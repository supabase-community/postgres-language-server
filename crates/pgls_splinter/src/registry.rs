//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
use pgls_analyse::RegistryVisitor;
#[doc = r" Visit all splinter rules using the visitor pattern"]
#[doc = r" This is called during registry building to collect enabled rules"]
pub fn visit_registry<V: RegistryVisitor>(registry: &mut V) {
    registry.record_category::<crate::rules::Splinter>();
}
#[doc = r" Map rule name (camelCase) to SQL file path"]
#[doc = r" Returns None if rule not found"]
pub fn get_sql_file_path(rule_name: &str) -> Option<&'static str> {
    match rule_name {
        "authRlsInitplan" => Some("vendor/performance/auth_rls_initplan.sql"),
        "authUsersExposed" => Some("vendor/security/auth_users_exposed.sql"),
        "duplicateIndex" => Some("vendor/performance/duplicate_index.sql"),
        "extensionInPublic" => Some("vendor/security/extension_in_public.sql"),
        "extensionVersionsOutdated" => Some("vendor/security/extension_versions_outdated.sql"),
        "fkeyToAuthUnique" => Some("vendor/security/fkey_to_auth_unique.sql"),
        "foreignTableInApi" => Some("vendor/security/foreign_table_in_api.sql"),
        "functionSearchPathMutable" => Some("vendor/security/function_search_path_mutable.sql"),
        "insecureQueueExposedInApi" => Some("vendor/security/insecure_queue_exposed_in_api.sql"),
        "materializedViewInApi" => Some("vendor/security/materialized_view_in_api.sql"),
        "multiplePermissivePolicies" => Some("vendor/performance/multiple_permissive_policies.sql"),
        "noPrimaryKey" => Some("vendor/performance/no_primary_key.sql"),
        "policyExistsRlsDisabled" => Some("vendor/security/policy_exists_rls_disabled.sql"),
        "rlsDisabledInPublic" => Some("vendor/security/rls_disabled_in_public.sql"),
        "rlsEnabledNoPolicy" => Some("vendor/security/rls_enabled_no_policy.sql"),
        "rlsReferencesUserMetadata" => Some("vendor/security/rls_references_user_metadata.sql"),
        "securityDefinerView" => Some("vendor/security/security_definer_view.sql"),
        "tableBloat" => Some("vendor/performance/table_bloat.sql"),
        "unindexedForeignKeys" => Some("vendor/performance/unindexed_foreign_keys.sql"),
        "unsupportedRegTypes" => Some("vendor/security/unsupported_reg_types.sql"),
        "unusedIndex" => Some("vendor/performance/unused_index.sql"),
        _ => None,
    }
}
