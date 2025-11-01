// This file contains the list of all diagnostic categories for the pg
// toolchain
//
// The `define_categories` macro is preprocessed in the build script for the
// crate in order to generate the static registry. The body of the macro
// consists of a list of key-value pairs defining the categories that have an
// associated hyperlink, then a list of string literals defining the remaining
// categories without a link.

// PLEASE, DON'T EDIT THIS FILE BY HAND.
// Use `just new-lintrule` to create a new rule.
// lint rules are lexicographically sorted and
// must be between `define_categories! {\n` and `\n    ;\n`.

define_categories! {
    "lint/safety/addSerialColumn": "https://pg-language-server.com/latest/rules/add-serial-column",
    "lint/safety/addingFieldWithDefault": "https://pg-language-server.com/latest/rules/adding-field-with-default",
    "lint/safety/addingForeignKeyConstraint": "https://pg-language-server.com/latest/rules/adding-foreign-key-constraint",
    "lint/safety/addingNotNullField": "https://pg-language-server.com/latest/rules/adding-not-null-field",
    "lint/safety/addingPrimaryKeyConstraint": "https://pg-language-server.com/latest/rules/adding-primary-key-constraint",
    "lint/safety/addingRequiredField": "https://pg-language-server.com/latest/rules/adding-required-field",
    "lint/safety/banCharField": "https://pg-language-server.com/latest/rules/ban-char-field",
    "lint/safety/banConcurrentIndexCreationInTransaction": "https://pg-language-server.com/latest/rules/ban-concurrent-index-creation-in-transaction",
    "lint/safety/banDropColumn": "https://pg-language-server.com/latest/rules/ban-drop-column",
    "lint/safety/banDropDatabase": "https://pg-language-server.com/latest/rules/ban-drop-database",
    "lint/safety/banDropNotNull": "https://pg-language-server.com/latest/rules/ban-drop-not-null",
    "lint/safety/banDropTable": "https://pg-language-server.com/latest/rules/ban-drop-table",
    "lint/safety/banTruncateCascade": "https://pg-language-server.com/latest/rules/ban-truncate-cascade",
    "lint/safety/changingColumnType": "https://pg-language-server.com/latest/rules/changing-column-type",
    "lint/safety/constraintMissingNotValid": "https://pg-language-server.com/latest/rules/constraint-missing-not-valid",
    "lint/safety/creatingEnum": "https://pg-language-server.com/latest/rules/creating-enum",
    "lint/safety/disallowUniqueConstraint": "https://pg-language-server.com/latest/rules/disallow-unique-constraint",
    "lint/safety/lockTimeoutWarning": "https://pg-language-server.com/latest/rules/lock-timeout-warning",
    "lint/safety/multipleAlterTable": "https://pg-language-server.com/latest/rules/multiple-alter-table",
    "lint/safety/preferBigInt": "https://pg-language-server.com/latest/rules/prefer-big-int",
    "lint/safety/preferBigintOverInt": "https://pg-language-server.com/latest/rules/prefer-bigint-over-int",
    "lint/safety/preferBigintOverSmallint": "https://pg-language-server.com/latest/rules/prefer-bigint-over-smallint",
    "lint/safety/preferIdentity": "https://pg-language-server.com/latest/rules/prefer-identity",
    "lint/safety/preferJsonb": "https://pg-language-server.com/latest/rules/prefer-jsonb",
    "lint/safety/preferRobustStmts": "https://pg-language-server.com/latest/rules/prefer-robust-stmts",
    "lint/safety/preferTextField": "https://pg-language-server.com/latest/rules/prefer-text-field",
    "lint/safety/preferTimestamptz": "https://pg-language-server.com/latest/rules/prefer-timestamptz",
    "lint/safety/renamingColumn": "https://pg-language-server.com/latest/rules/renaming-column",
    "lint/safety/renamingTable": "https://pg-language-server.com/latest/rules/renaming-table",
    "lint/safety/requireConcurrentIndexCreation": "https://pg-language-server.com/latest/rules/require-concurrent-index-creation",
    "lint/safety/requireConcurrentIndexDeletion": "https://pg-language-server.com/latest/rules/require-concurrent-index-deletion",
    "lint/safety/runningStatementWhileHoldingAccessExclusive": "https://pg-language-server.com/latest/rules/running-statement-while-holding-access-exclusive",
    "lint/safety/transactionNesting": "https://pg-language-server.com/latest/rules/transaction-nesting",
    // end lint rules
    // splinter rules start
    "splinter/performance/authRlsInitplan": "https://supabase.com/docs/guides/database/database-linter?lint=0003_auth_rls_initplan",
    "splinter/performance/duplicateIndex": "https://supabase.com/docs/guides/database/database-linter?lint=0009_duplicate_index",
    "splinter/performance/multiplePermissivePolicies": "https://supabase.com/docs/guides/database/database-linter?lint=0006_multiple_permissive_policies",
    "splinter/performance/noPrimaryKey": "https://supabase.com/docs/guides/database/database-linter?lint=0004_no_primary_key",
    "splinter/performance/tableBloat": "https://supabase.com/docs/guides/database/database-linter?lint=0020_table_bloat",
    "splinter/performance/unindexedForeignKeys": "https://supabase.com/docs/guides/database/database-linter?lint=0001_unindexed_foreign_keys",
    "splinter/performance/unusedIndex": "https://supabase.com/docs/guides/database/database-linter?lint=0005_unused_index",
    "splinter/security/authUsersExposed": "https://supabase.com/docs/guides/database/database-linter?lint=0002_auth_users_exposed",
    "splinter/security/extensionInPublic": "https://supabase.com/docs/guides/database/database-linter?lint=0014_extension_in_public",
    "splinter/security/extensionVersionsOutdated": "https://supabase.com/docs/guides/database/database-linter?lint=0022_extension_versions_outdated",
    "splinter/security/fkeyToAuthUnique": "https://supabase.com/docs/guides/database/database-linter?lint=0021_fkey_to_auth_unique",
    "splinter/security/foreignTableInApi": "https://supabase.com/docs/guides/database/database-linter?lint=0017_foreign_table_in_api",
    "splinter/security/functionSearchPathMutable": "https://supabase.com/docs/guides/database/database-linter?lint=0011_function_search_path_mutable",
    "splinter/security/insecureQueueExposedInApi": "https://supabase.com/docs/guides/database/database-linter?lint=0019_insecure_queue_exposed_in_api",
    "splinter/security/materializedViewInApi": "https://supabase.com/docs/guides/database/database-linter?lint=0016_materialized_view_in_api",
    "splinter/security/policyExistsRlsDisabled": "https://supabase.com/docs/guides/database/database-linter?lint=0007_policy_exists_rls_disabled",
    "splinter/security/rlsDisabledInPublic": "https://supabase.com/docs/guides/database/database-linter?lint=0013_rls_disabled_in_public",
    "splinter/security/rlsEnabledNoPolicy": "https://supabase.com/docs/guides/database/database-linter?lint=0008_rls_enabled_no_policy",
    "splinter/security/rlsReferencesUserMetadata": "https://supabase.com/docs/guides/database/database-linter?lint=0015_rls_references_user_metadata",
    "splinter/security/securityDefinerView": "https://supabase.com/docs/guides/database/database-linter?lint=0010_security_definer_view",
    "splinter/security/unsupportedRegTypes": "https://supabase.com/docs/guides/database/database-linter?lint=unsupported_reg_types",
    "splinter/unknown/unknown": "https://pg-language-server.com/latest",
    // splinter rules end
    ;
    // General categories
    "stdin",
    "check",
    "configuration",
    "database/connection",
    "internalError/io",
    "internalError/runtime",
    "internalError/fs",
    "flags/invalid",
    "project",
    "typecheck",
    "plpgsql_check",
    "internalError/panic",
    "syntax",
    "dummy",

    // Lint groups start
    "lint",
    "lint/performance",
    "lint/safety",
    // Lint groups end

    // Splinter groups start
    "splinter",
    "splinter/performance",
    "splinter/security",
    "splinter/unknown",
    // Splinter groups end
}
