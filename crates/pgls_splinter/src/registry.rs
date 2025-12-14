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
        "authRlsInitplan" => Some(
            "/Users/psteinroe/Developer/postgres-language-server.git/refactor/rules/crates/pgls_splinter/vendor/performance/auth_rls_initplan.sql",
        ),
        "authUsersExposed" => Some(
            "/Users/psteinroe/Developer/postgres-language-server.git/refactor/rules/crates/pgls_splinter/vendor/security/auth_users_exposed.sql",
        ),
        "duplicateIndex" => Some(
            "/Users/psteinroe/Developer/postgres-language-server.git/refactor/rules/crates/pgls_splinter/vendor/performance/duplicate_index.sql",
        ),
        "extensionInPublic" => Some(
            "/Users/psteinroe/Developer/postgres-language-server.git/refactor/rules/crates/pgls_splinter/vendor/security/extension_in_public.sql",
        ),
        "extensionVersionsOutdated" => Some(
            "/Users/psteinroe/Developer/postgres-language-server.git/refactor/rules/crates/pgls_splinter/vendor/security/extension_versions_outdated.sql",
        ),
        "fkeyToAuthUnique" => Some(
            "/Users/psteinroe/Developer/postgres-language-server.git/refactor/rules/crates/pgls_splinter/vendor/security/fkey_to_auth_unique.sql",
        ),
        "foreignTableInApi" => Some(
            "/Users/psteinroe/Developer/postgres-language-server.git/refactor/rules/crates/pgls_splinter/vendor/security/foreign_table_in_api.sql",
        ),
        "functionSearchPathMutable" => Some(
            "/Users/psteinroe/Developer/postgres-language-server.git/refactor/rules/crates/pgls_splinter/vendor/security/function_search_path_mutable.sql",
        ),
        "insecureQueueExposedInApi" => Some(
            "/Users/psteinroe/Developer/postgres-language-server.git/refactor/rules/crates/pgls_splinter/vendor/security/insecure_queue_exposed_in_api.sql",
        ),
        "materializedViewInApi" => Some(
            "/Users/psteinroe/Developer/postgres-language-server.git/refactor/rules/crates/pgls_splinter/vendor/security/materialized_view_in_api.sql",
        ),
        "multiplePermissivePolicies" => Some(
            "/Users/psteinroe/Developer/postgres-language-server.git/refactor/rules/crates/pgls_splinter/vendor/performance/multiple_permissive_policies.sql",
        ),
        "noPrimaryKey" => Some(
            "/Users/psteinroe/Developer/postgres-language-server.git/refactor/rules/crates/pgls_splinter/vendor/performance/no_primary_key.sql",
        ),
        "policyExistsRlsDisabled" => Some(
            "/Users/psteinroe/Developer/postgres-language-server.git/refactor/rules/crates/pgls_splinter/vendor/security/policy_exists_rls_disabled.sql",
        ),
        "rlsDisabledInPublic" => Some(
            "/Users/psteinroe/Developer/postgres-language-server.git/refactor/rules/crates/pgls_splinter/vendor/security/rls_disabled_in_public.sql",
        ),
        "rlsEnabledNoPolicy" => Some(
            "/Users/psteinroe/Developer/postgres-language-server.git/refactor/rules/crates/pgls_splinter/vendor/security/rls_enabled_no_policy.sql",
        ),
        "rlsReferencesUserMetadata" => Some(
            "/Users/psteinroe/Developer/postgres-language-server.git/refactor/rules/crates/pgls_splinter/vendor/security/rls_references_user_metadata.sql",
        ),
        "securityDefinerView" => Some(
            "/Users/psteinroe/Developer/postgres-language-server.git/refactor/rules/crates/pgls_splinter/vendor/security/security_definer_view.sql",
        ),
        "tableBloat" => Some(
            "/Users/psteinroe/Developer/postgres-language-server.git/refactor/rules/crates/pgls_splinter/vendor/performance/table_bloat.sql",
        ),
        "unindexedForeignKeys" => Some(
            "/Users/psteinroe/Developer/postgres-language-server.git/refactor/rules/crates/pgls_splinter/vendor/performance/unindexed_foreign_keys.sql",
        ),
        "unsupportedRegTypes" => Some(
            "/Users/psteinroe/Developer/postgres-language-server.git/refactor/rules/crates/pgls_splinter/vendor/security/unsupported_reg_types.sql",
        ),
        "unusedIndex" => Some(
            "/Users/psteinroe/Developer/postgres-language-server.git/refactor/rules/crates/pgls_splinter/vendor/performance/unused_index.sql",
        ),
        _ => None,
    }
}
