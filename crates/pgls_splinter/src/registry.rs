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
#[deprecated(note = "Use get_sql_content() instead - SQL is now embedded at compile time")]
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
#[doc = r" Get embedded SQL content for a rule (camelCase name)"]
#[doc = r" Returns None if rule not found"]
#[doc = r""]
#[doc = r" SQL files are embedded at compile time using include_str! for performance"]
#[doc = r" and to make the binary self-contained."]
pub fn get_sql_content(rule_name: &str) -> Option<&'static str> {
    match rule_name {
        "authRlsInitplan" => Some(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/",
            "vendor/performance/auth_rls_initplan.sql"
        ))),
        "authUsersExposed" => Some(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/",
            "vendor/security/auth_users_exposed.sql"
        ))),
        "duplicateIndex" => Some(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/",
            "vendor/performance/duplicate_index.sql"
        ))),
        "extensionInPublic" => Some(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/",
            "vendor/security/extension_in_public.sql"
        ))),
        "extensionVersionsOutdated" => Some(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/",
            "vendor/security/extension_versions_outdated.sql"
        ))),
        "fkeyToAuthUnique" => Some(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/",
            "vendor/security/fkey_to_auth_unique.sql"
        ))),
        "foreignTableInApi" => Some(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/",
            "vendor/security/foreign_table_in_api.sql"
        ))),
        "functionSearchPathMutable" => Some(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/",
            "vendor/security/function_search_path_mutable.sql"
        ))),
        "insecureQueueExposedInApi" => Some(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/",
            "vendor/security/insecure_queue_exposed_in_api.sql"
        ))),
        "materializedViewInApi" => Some(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/",
            "vendor/security/materialized_view_in_api.sql"
        ))),
        "multiplePermissivePolicies" => Some(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/",
            "vendor/performance/multiple_permissive_policies.sql"
        ))),
        "noPrimaryKey" => Some(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/",
            "vendor/performance/no_primary_key.sql"
        ))),
        "policyExistsRlsDisabled" => Some(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/",
            "vendor/security/policy_exists_rls_disabled.sql"
        ))),
        "rlsDisabledInPublic" => Some(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/",
            "vendor/security/rls_disabled_in_public.sql"
        ))),
        "rlsEnabledNoPolicy" => Some(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/",
            "vendor/security/rls_enabled_no_policy.sql"
        ))),
        "rlsReferencesUserMetadata" => Some(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/",
            "vendor/security/rls_references_user_metadata.sql"
        ))),
        "securityDefinerView" => Some(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/",
            "vendor/security/security_definer_view.sql"
        ))),
        "tableBloat" => Some(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/",
            "vendor/performance/table_bloat.sql"
        ))),
        "unindexedForeignKeys" => Some(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/",
            "vendor/performance/unindexed_foreign_keys.sql"
        ))),
        "unsupportedRegTypes" => Some(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/",
            "vendor/security/unsupported_reg_types.sql"
        ))),
        "unusedIndex" => Some(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/",
            "vendor/performance/unused_index.sql"
        ))),
        _ => None,
    }
}
#[doc = r" Map rule name from SQL result (snake_case) to diagnostic category"]
#[doc = r" Returns None if rule not found"]
#[doc = r""]
#[doc = r" This replaces the hardcoded match in convert.rs"]
pub fn get_rule_category(rule_name: &str) -> Option<&'static ::pgls_diagnostics::Category> {
    match rule_name {
        "auth_rls_initplan" => Some(::pgls_diagnostics::category!(
            "splinter/performance/authRlsInitplan"
        )),
        "auth_users_exposed" => Some(::pgls_diagnostics::category!(
            "splinter/security/authUsersExposed"
        )),
        "duplicate_index" => Some(::pgls_diagnostics::category!(
            "splinter/performance/duplicateIndex"
        )),
        "extension_in_public" => Some(::pgls_diagnostics::category!(
            "splinter/security/extensionInPublic"
        )),
        "extension_versions_outdated" => Some(::pgls_diagnostics::category!(
            "splinter/security/extensionVersionsOutdated"
        )),
        "fkey_to_auth_unique" => Some(::pgls_diagnostics::category!(
            "splinter/security/fkeyToAuthUnique"
        )),
        "foreign_table_in_api" => Some(::pgls_diagnostics::category!(
            "splinter/security/foreignTableInApi"
        )),
        "function_search_path_mutable" => Some(::pgls_diagnostics::category!(
            "splinter/security/functionSearchPathMutable"
        )),
        "insecure_queue_exposed_in_api" => Some(::pgls_diagnostics::category!(
            "splinter/security/insecureQueueExposedInApi"
        )),
        "materialized_view_in_api" => Some(::pgls_diagnostics::category!(
            "splinter/security/materializedViewInApi"
        )),
        "multiple_permissive_policies" => Some(::pgls_diagnostics::category!(
            "splinter/performance/multiplePermissivePolicies"
        )),
        "no_primary_key" => Some(::pgls_diagnostics::category!(
            "splinter/performance/noPrimaryKey"
        )),
        "policy_exists_rls_disabled" => Some(::pgls_diagnostics::category!(
            "splinter/security/policyExistsRlsDisabled"
        )),
        "rls_disabled_in_public" => Some(::pgls_diagnostics::category!(
            "splinter/security/rlsDisabledInPublic"
        )),
        "rls_enabled_no_policy" => Some(::pgls_diagnostics::category!(
            "splinter/security/rlsEnabledNoPolicy"
        )),
        "rls_references_user_metadata" => Some(::pgls_diagnostics::category!(
            "splinter/security/rlsReferencesUserMetadata"
        )),
        "security_definer_view" => Some(::pgls_diagnostics::category!(
            "splinter/security/securityDefinerView"
        )),
        "table_bloat" => Some(::pgls_diagnostics::category!(
            "splinter/performance/tableBloat"
        )),
        "unindexed_foreign_keys" => Some(::pgls_diagnostics::category!(
            "splinter/performance/unindexedForeignKeys"
        )),
        "unsupported_reg_types" => Some(::pgls_diagnostics::category!(
            "splinter/security/unsupportedRegTypes"
        )),
        "unused_index" => Some(::pgls_diagnostics::category!(
            "splinter/performance/unusedIndex"
        )),
        _ => None,
    }
}
#[doc = r" Check if a rule requires Supabase roles (anon, authenticated, service_role)"]
#[doc = r" Rules that require Supabase should be filtered out if these roles don't exist"]
pub fn rule_requires_supabase(rule_name: &str) -> bool {
    match rule_name {
        "authRlsInitplan" => true,
        "authUsersExposed" => true,
        "duplicateIndex" => false,
        "extensionInPublic" => false,
        "extensionVersionsOutdated" => false,
        "fkeyToAuthUnique" => true,
        "foreignTableInApi" => true,
        "functionSearchPathMutable" => false,
        "insecureQueueExposedInApi" => true,
        "materializedViewInApi" => true,
        "multiplePermissivePolicies" => false,
        "noPrimaryKey" => false,
        "policyExistsRlsDisabled" => false,
        "rlsDisabledInPublic" => true,
        "rlsEnabledNoPolicy" => false,
        "rlsReferencesUserMetadata" => true,
        "securityDefinerView" => true,
        "tableBloat" => false,
        "unindexedForeignKeys" => false,
        "unsupportedRegTypes" => false,
        "unusedIndex" => false,
        _ => false,
    }
}
