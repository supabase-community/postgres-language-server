//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
use pgls_analyse::RegistryVisitor;
#[doc = r" Metadata for a splinter rule"]
#[derive(Debug, Clone, Copy)]
pub struct SplinterRuleMetadata {
    #[doc = r" Description of what the rule detects"]
    pub description: &'static str,
    #[doc = r" URL to documentation/remediation guide"]
    pub remediation: &'static str,
    #[doc = r" Whether this rule requires Supabase roles (anon, authenticated, service_role)"]
    pub requires_supabase: bool,
}
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
#[doc = r" Get metadata for a rule (camelCase name)"]
#[doc = r" Returns None if rule not found"]
#[doc = r""]
#[doc = r" This provides structured access to rule metadata without requiring SQL parsing"]
pub fn get_rule_metadata(rule_name: &str) -> Option<SplinterRuleMetadata> {
    match rule_name {
        "authRlsInitplan" => Some(SplinterRuleMetadata {
            description: "Detects if calls to \\`current_setting()\\` and \\`auth.<function>()\\` in RLS policies are being unnecessarily re-evaluated for each row",
            remediation: "https://supabase.com/docs/guides/database/database-linter?lint=0003_auth_rls_initplan",
            requires_supabase: true,
        }),
        "authUsersExposed" => Some(SplinterRuleMetadata {
            description: "Detects if auth.users is exposed to anon or authenticated roles via a view or materialized view in schemas exposed to PostgREST, potentially compromising user data security.",
            remediation: "https://supabase.com/docs/guides/database/database-linter?lint=0002_auth_users_exposed",
            requires_supabase: true,
        }),
        "duplicateIndex" => Some(SplinterRuleMetadata {
            description: "Detects cases where two ore more identical indexes exist.",
            remediation: "https://supabase.com/docs/guides/database/database-linter?lint=0009_duplicate_index",
            requires_supabase: false,
        }),
        "extensionInPublic" => Some(SplinterRuleMetadata {
            description: "Detects extensions installed in the \\`public\\` schema.",
            remediation: "https://supabase.com/docs/guides/database/database-linter?lint=0014_extension_in_public",
            requires_supabase: false,
        }),
        "extensionVersionsOutdated" => Some(SplinterRuleMetadata {
            description: "Detects extensions that are not using the default (recommended) version.",
            remediation: "https://supabase.com/docs/guides/database/database-linter?lint=0022_extension_versions_outdated",
            requires_supabase: false,
        }),
        "fkeyToAuthUnique" => Some(SplinterRuleMetadata {
            description: "Detects user defined foreign keys to unique constraints in the auth schema.",
            remediation: "Drop the foreign key constraint that references the auth schema.",
            requires_supabase: true,
        }),
        "foreignTableInApi" => Some(SplinterRuleMetadata {
            description: "Detects foreign tables that are accessible over APIs. Foreign tables do not respect row level security policies.",
            remediation: "https://supabase.com/docs/guides/database/database-linter?lint=0017_foreign_table_in_api",
            requires_supabase: true,
        }),
        "functionSearchPathMutable" => Some(SplinterRuleMetadata {
            description: "Detects functions where the search_path parameter is not set.",
            remediation: "https://supabase.com/docs/guides/database/database-linter?lint=0011_function_search_path_mutable",
            requires_supabase: false,
        }),
        "insecureQueueExposedInApi" => Some(SplinterRuleMetadata {
            description: "Detects cases where an insecure Queue is exposed over Data APIs",
            remediation: "https://supabase.com/docs/guides/database/database-linter?lint=0019_insecure_queue_exposed_in_api",
            requires_supabase: true,
        }),
        "materializedViewInApi" => Some(SplinterRuleMetadata {
            description: "Detects materialized views that are accessible over the Data APIs.",
            remediation: "https://supabase.com/docs/guides/database/database-linter?lint=0016_materialized_view_in_api",
            requires_supabase: true,
        }),
        "multiplePermissivePolicies" => Some(SplinterRuleMetadata {
            description: "Detects if multiple permissive row level security policies are present on a table for the same \\`role\\` and \\`action\\` (e.g. insert). Multiple permissive policies are suboptimal for performance as each policy must be executed for every relevant query.",
            remediation: "https://supabase.com/docs/guides/database/database-linter?lint=0006_multiple_permissive_policies",
            requires_supabase: false,
        }),
        "noPrimaryKey" => Some(SplinterRuleMetadata {
            description: "Detects if a table does not have a primary key. Tables without a primary key can be inefficient to interact with at scale.",
            remediation: "https://supabase.com/docs/guides/database/database-linter?lint=0004_no_primary_key",
            requires_supabase: false,
        }),
        "policyExistsRlsDisabled" => Some(SplinterRuleMetadata {
            description: "Detects cases where row level security (RLS) policies have been created, but RLS has not been enabled for the underlying table.",
            remediation: "https://supabase.com/docs/guides/database/database-linter?lint=0007_policy_exists_rls_disabled",
            requires_supabase: false,
        }),
        "rlsDisabledInPublic" => Some(SplinterRuleMetadata {
            description: "Detects cases where row level security (RLS) has not been enabled on tables in schemas exposed to PostgREST",
            remediation: "https://supabase.com/docs/guides/database/database-linter?lint=0013_rls_disabled_in_public",
            requires_supabase: true,
        }),
        "rlsEnabledNoPolicy" => Some(SplinterRuleMetadata {
            description: "Detects cases where row level security (RLS) has been enabled on a table but no RLS policies have been created.",
            remediation: "https://supabase.com/docs/guides/database/database-linter?lint=0008_rls_enabled_no_policy",
            requires_supabase: false,
        }),
        "rlsReferencesUserMetadata" => Some(SplinterRuleMetadata {
            description: "Detects when Supabase Auth user_metadata is referenced insecurely in a row level security (RLS) policy.",
            remediation: "https://supabase.com/docs/guides/database/database-linter?lint=0015_rls_references_user_metadata",
            requires_supabase: true,
        }),
        "securityDefinerView" => Some(SplinterRuleMetadata {
            description: "Detects views defined with the SECURITY DEFINER property. These views enforce Postgres permissions and row level security policies (RLS) of the view creator, rather than that of the querying user",
            remediation: "https://supabase.com/docs/guides/database/database-linter?lint=0010_security_definer_view",
            requires_supabase: true,
        }),
        "tableBloat" => Some(SplinterRuleMetadata {
            description: "Detects if a table has excess bloat and may benefit from maintenance operations like vacuum full or cluster.",
            remediation: "Consider running vacuum full (WARNING: incurs downtime) and tweaking autovacuum settings to reduce bloat.",
            requires_supabase: false,
        }),
        "unindexedForeignKeys" => Some(SplinterRuleMetadata {
            description: "Identifies foreign key constraints without a covering index, which can impact database performance.",
            remediation: "https://supabase.com/docs/guides/database/database-linter?lint=0001_unindexed_foreign_keys",
            requires_supabase: false,
        }),
        "unsupportedRegTypes" => Some(SplinterRuleMetadata {
            description: "Identifies columns using unsupported reg* types outside pg_catalog schema, which prevents database upgrades using pg_upgrade.",
            remediation: "https://supabase.com/docs/guides/database/database-linter?lint=unsupported_reg_types",
            requires_supabase: false,
        }),
        "unusedIndex" => Some(SplinterRuleMetadata {
            description: "Detects if an index has never been used and may be a candidate for removal.",
            remediation: "https://supabase.com/docs/guides/database/database-linter?lint=0005_unused_index",
            requires_supabase: false,
        }),
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
#[deprecated(note = "Use get_rule_metadata() instead")]
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
