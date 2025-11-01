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
    "dblint/splinter/authRlsInitplan": "https://supabase.com/docs/guides/database/database-linter?lint=0003_auth_rls_initplan",
    "dblint/splinter/authUsersExposed": "https://supabase.com/docs/guides/database/database-linter?lint=0002_auth_users_exposed",
    "dblint/splinter/duplicateIndex": "https://supabase.com/docs/guides/database/database-linter?lint=0009_duplicate_index",
    "dblint/splinter/extensionInPublic": "https://supabase.com/docs/guides/database/database-linter?lint=0014_extension_in_public",
    "dblint/splinter/extensionVersionsOutdated": "https://supabase.com/docs/guides/database/database-linter?lint=0022_extension_versions_outdated",
    "dblint/splinter/fkeyToAuthUnique": "https://supabase.com/docs/guides/database/database-linter?lint=0021_fkey_to_auth_unique",
    "dblint/splinter/foreignTableInApi": "https://supabase.com/docs/guides/database/database-linter?lint=0017_foreign_table_in_api",
    "dblint/splinter/functionSearchPathMutable": "https://supabase.com/docs/guides/database/database-linter?lint=0011_function_search_path_mutable",
    "dblint/splinter/insecureQueueExposedInApi": "https://supabase.com/docs/guides/database/database-linter?lint=0019_insecure_queue_exposed_in_api",
    "dblint/splinter/materializedViewInApi": "https://supabase.com/docs/guides/database/database-linter?lint=0016_materialized_view_in_api",
    "dblint/splinter/multiplePermissivePolicies": "https://supabase.com/docs/guides/database/database-linter?lint=0006_multiple_permissive_policies",
    "dblint/splinter/noPrimaryKey": "https://supabase.com/docs/guides/database/database-linter?lint=0004_no_primary_key",
    "dblint/splinter/policyExistsRlsDisabled": "https://supabase.com/docs/guides/database/database-linter?lint=0007_policy_exists_rls_disabled",
    "dblint/splinter/rlsDisabledInPublic": "https://supabase.com/docs/guides/database/database-linter?lint=0013_rls_disabled_in_public",
    "dblint/splinter/rlsEnabledNoPolicy": "https://supabase.com/docs/guides/database/database-linter?lint=0008_rls_enabled_no_policy",
    "dblint/splinter/rlsReferencesUserMetadata": "https://supabase.com/docs/guides/database/database-linter?lint=0015_rls_references_user_metadata",
    "dblint/splinter/securityDefinerView": "https://supabase.com/docs/guides/database/database-linter?lint=0010_security_definer_view",
    "dblint/splinter/tableBloat": "https://supabase.com/docs/guides/database/database-linter?lint=0020_table_bloat",
    "dblint/splinter/unindexedForeignKeys": "https://supabase.com/docs/guides/database/database-linter?lint=0001_unindexed_foreign_keys",
    "dblint/splinter/unknown": "https://pg-language-server.com/latest",
    "dblint/splinter/unsupportedRegTypes": "https://supabase.com/docs/guides/database/database-linter?lint=unsupported_reg_types",
    "dblint/splinter/unusedIndex": "https://supabase.com/docs/guides/database/database-linter?lint=0005_unused_index",
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
    // splinter groups start
    "dblint",
    "dblint/splinter",
    // splinter groups end
    // Lint groups end
}