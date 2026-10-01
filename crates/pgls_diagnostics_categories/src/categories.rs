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
    "lint/addSerialColumn": "https://pg-language-server.com/latest/reference/rules/add-serial-column/",
    "lint/addingFieldWithDefault": "https://pg-language-server.com/latest/reference/rules/adding-field-with-default/",
    "lint/addingForeignKeyConstraint": "https://pg-language-server.com/latest/reference/rules/adding-foreign-key-constraint/",
    "lint/addingNotNullField": "https://pg-language-server.com/latest/reference/rules/adding-not-null-field/",
    "lint/addingPrimaryKeyConstraint": "https://pg-language-server.com/latest/reference/rules/adding-primary-key-constraint/",
    "lint/addingRequiredField": "https://pg-language-server.com/latest/reference/rules/adding-required-field/",
    "lint/ambiguousColumn": "https://pg-language-server.com/latest/reference/rules/ambiguous-column/",
    "lint/avoidAddingExclusionConstraint": "https://pg-language-server.com/latest/reference/rules/avoid-adding-exclusion-constraint/",
    "lint/avoidAlterEnumAddValue": "https://pg-language-server.com/latest/reference/rules/avoid-alter-enum-add-value/",
    "lint/avoidAttachingPartition": "https://pg-language-server.com/latest/reference/rules/avoid-attaching-partition/",
    "lint/avoidCreateTrigger": "https://pg-language-server.com/latest/reference/rules/avoid-create-trigger/",
    "lint/avoidEnableDisableTrigger": "https://pg-language-server.com/latest/reference/rules/avoid-enable-disable-trigger/",
    "lint/avoidWideLockWindow": "https://pg-language-server.com/latest/reference/rules/avoid-wide-lock-window/",
    "lint/banCharField": "https://pg-language-server.com/latest/reference/rules/ban-char-field/",
    "lint/banConcurrentIndexCreationInTransaction": "https://pg-language-server.com/latest/reference/rules/ban-concurrent-index-creation-in-transaction/",
    "lint/banDeleteWithoutWhere": "https://pg-language-server.com/latest/reference/rules/ban-delete-without-where/",
    "lint/banDropColumn": "https://pg-language-server.com/latest/reference/rules/ban-drop-column/",
    "lint/banDropDatabase": "https://pg-language-server.com/latest/reference/rules/ban-drop-database/",
    "lint/banDropNotNull": "https://pg-language-server.com/latest/reference/rules/ban-drop-not-null/",
    "lint/banDropSchema": "https://pg-language-server.com/latest/reference/rules/ban-drop-schema/",
    "lint/banDropTable": "https://pg-language-server.com/latest/reference/rules/ban-drop-table/",
    "lint/banDropTrigger": "https://pg-language-server.com/latest/reference/rules/ban-drop-trigger/",
    "lint/banTruncate": "https://pg-language-server.com/latest/reference/rules/ban-truncate/",
    "lint/banTruncateCascade": "https://pg-language-server.com/latest/reference/rules/ban-truncate-cascade/",
    "lint/banUpdateWithoutWhere": "https://pg-language-server.com/latest/reference/rules/ban-update-without-where/",
    "lint/banVacuumFull": "https://pg-language-server.com/latest/reference/rules/ban-vacuum-full/",
    "lint/changingColumnType": "https://pg-language-server.com/latest/reference/rules/changing-column-type/",
    "lint/constraintMissingNotValid": "https://pg-language-server.com/latest/reference/rules/constraint-missing-not-valid/",
    "lint/creatingEnum": "https://pg-language-server.com/latest/reference/rules/creating-enum/",
    "lint/disallowUniqueConstraint": "https://pg-language-server.com/latest/reference/rules/disallow-unique-constraint/",
    "lint/functionReturnTypeMismatch": "https://pg-language-server.com/latest/reference/rules/function-return-type-mismatch/",
    "lint/insertColumnMismatch": "https://pg-language-server.com/latest/reference/rules/insert-column-mismatch/",
    "lint/invalidDropTypeSignature": "https://pg-language-server.com/latest/reference/rules/invalid-drop-type-signature/",
    "lint/lockTimeoutWarning": "https://pg-language-server.com/latest/reference/rules/lock-timeout-warning/",
    "lint/missingFromClauseEntry": "https://pg-language-server.com/latest/reference/rules/missing-from-clause-entry/",
    "lint/multipleAlterTable": "https://pg-language-server.com/latest/reference/rules/multiple-alter-table/",
    "lint/preferBigInt": "https://pg-language-server.com/latest/reference/rules/prefer-big-int/",
    "lint/preferIdentity": "https://pg-language-server.com/latest/reference/rules/prefer-identity/",
    "lint/preferJsonb": "https://pg-language-server.com/latest/reference/rules/prefer-jsonb/",
    "lint/preferRobustStmts": "https://pg-language-server.com/latest/reference/rules/prefer-robust-stmts/",
    "lint/preferTextField": "https://pg-language-server.com/latest/reference/rules/prefer-text-field/",
    "lint/preferTimestamptz": "https://pg-language-server.com/latest/reference/rules/prefer-timestamptz/",
    "lint/renamingColumn": "https://pg-language-server.com/latest/reference/rules/renaming-column/",
    "lint/renamingTable": "https://pg-language-server.com/latest/reference/rules/renaming-table/",
    "lint/requireConcurrentDetachPartition": "https://pg-language-server.com/latest/reference/rules/require-concurrent-detach-partition/",
    "lint/requireConcurrentIndexCreation": "https://pg-language-server.com/latest/reference/rules/require-concurrent-index-creation/",
    "lint/requireConcurrentIndexDeletion": "https://pg-language-server.com/latest/reference/rules/require-concurrent-index-deletion/",
    "lint/requireConcurrentRefreshMatview": "https://pg-language-server.com/latest/reference/rules/require-concurrent-refresh-matview/",
    "lint/requireConcurrentReindex": "https://pg-language-server.com/latest/reference/rules/require-concurrent-reindex/",
    "lint/requireIdleInTransactionTimeout": "https://pg-language-server.com/latest/reference/rules/require-idle-in-transaction-timeout/",
    "lint/requireSeparateConstraintValidation": "https://pg-language-server.com/latest/reference/rules/require-separate-constraint-validation/",
    "lint/requireStatementTimeout": "https://pg-language-server.com/latest/reference/rules/require-statement-timeout/",
    "lint/runningStatementWhileHoldingAccessExclusive": "https://pg-language-server.com/latest/reference/rules/running-statement-while-holding-access-exclusive/",
    "lint/transactionNesting": "https://pg-language-server.com/latest/reference/rules/transaction-nesting/",
    "lint/unknownColumn": "https://pg-language-server.com/latest/reference/rules/unknown-column/",
    "lint/unknownFunction": "https://pg-language-server.com/latest/reference/rules/unknown-function/",
    "lint/unknownRelation": "https://pg-language-server.com/latest/reference/rules/unknown-relation/",
    "lint/unknownSchema": "https://pg-language-server.com/latest/reference/rules/unknown-schema/",
    "lint/unknownType": "https://pg-language-server.com/latest/reference/rules/unknown-type/",
    // end lint rules
    // pglinter rules start
    // Meta diagnostics
    "pglinter/extensionNotInstalled": "Install the pglinter extension with: CREATE EXTENSION pglinter",
    "pglinter/ruleDisabledInExtension": "Enable the rule in the extension with: UPDATE pglinter.rules SET enable = true WHERE code = '<code>'",
    // Base rules (B-series)
    "pglinter/base/compositePrimaryKeyTooManyColumns": "https://github.com/pmpetit/pglinter#b012",
    "pglinter/base/howManyObjectsWithUppercase": "https://github.com/pmpetit/pglinter#b005",
    "pglinter/base/howManyRedudantIndex": "https://github.com/pmpetit/pglinter#b002",
    "pglinter/base/howManyTableWithoutIndexOnFk": "https://github.com/pmpetit/pglinter#b003",
    "pglinter/base/howManyTableWithoutPrimaryKey": "https://github.com/pmpetit/pglinter#b001",
    "pglinter/base/howManyTablesNeverSelected": "https://github.com/pmpetit/pglinter#b006",
    "pglinter/base/howManyTablesWithFkMismatch": "https://github.com/pmpetit/pglinter#b008",
    "pglinter/base/howManyTablesWithFkOutsideSchema": "https://github.com/pmpetit/pglinter#b007",
    "pglinter/base/howManyTablesWithReservedKeywords": "https://github.com/pmpetit/pglinter#b010",
    "pglinter/base/howManyTablesWithSameTrigger": "https://github.com/pmpetit/pglinter#b009",
    "pglinter/base/howManyUnusedIndex": "https://github.com/pmpetit/pglinter#b004",
    "pglinter/base/severalTableOwnerInSchema": "https://github.com/pmpetit/pglinter#b011",
    // Cluster rules (C-series)
    "pglinter/cluster/passwordEncryptionIsMd5": "https://github.com/pmpetit/pglinter#c003",
    "pglinter/cluster/pgHbaEntriesWithMethodTrustOrPasswordShouldNotExists": "https://github.com/pmpetit/pglinter#c002",
    "pglinter/cluster/pgHbaEntriesWithMethodTrustShouldNotExists": "https://github.com/pmpetit/pglinter#c001",
    // Schema rules (S-series)
    "pglinter/schema/ownerSchemaIsInternalRole": "https://github.com/pmpetit/pglinter#s004",
    "pglinter/schema/schemaOwnerDoNotMatchTableOwner": "https://github.com/pmpetit/pglinter#s005",
    "pglinter/schema/schemaPrefixedOrSuffixedWithEnvt": "https://github.com/pmpetit/pglinter#s002",
    "pglinter/schema/schemaWithDefaultRoleNotGranted": "https://github.com/pmpetit/pglinter#s001",
    "pglinter/schema/unsecuredPublicSchema": "https://github.com/pmpetit/pglinter#s003",
    // pglinter rules end

    // splinter rules start
    "splinter/performance/authRlsInitplan": "https://supabase.com/docs/guides/database/database-linter?lint=0003_auth_rls_initplan",
    "splinter/performance/duplicateIndex": "https://supabase.com/docs/guides/database/database-linter?lint=0009_duplicate_index",
    "splinter/performance/multiplePermissivePolicies": "https://supabase.com/docs/guides/database/database-linter?lint=0006_multiple_permissive_policies",
    "splinter/performance/noPrimaryKey": "https://supabase.com/docs/guides/database/database-linter?lint=0004_no_primary_key",
    "splinter/performance/tableBloat": "Consider running vacuum full (WARNING: incurs downtime) and tweaking autovacuum settings to reduce bloat.",
    "splinter/performance/unindexedForeignKeys": "https://supabase.com/docs/guides/database/database-linter?lint=0001_unindexed_foreign_keys",
    "splinter/performance/unusedIndex": "https://supabase.com/docs/guides/database/database-linter?lint=0005_unused_index",
    "splinter/security/authUsersExposed": "https://supabase.com/docs/guides/database/database-linter?lint=0002_auth_users_exposed",
    "splinter/security/extensionInPublic": "https://supabase.com/docs/guides/database/database-linter?lint=0014_extension_in_public",
    "splinter/security/extensionVersionsOutdated": "https://supabase.com/docs/guides/database/database-linter?lint=0022_extension_versions_outdated",
    "splinter/security/fkeyToAuthUnique": "Drop the foreign key constraint that references the auth schema.",
    "splinter/security/foreignTableInApi": "https://supabase.com/docs/guides/database/database-linter?lint=0017_foreign_table_in_api",
    "splinter/security/functionSearchPathMutable": "https://supabase.com/docs/guides/database/database-linter?lint=0011_function_search_path_mutable",
    "splinter/security/insecureQueueExposedInApi": "https://supabase.com/docs/guides/database/database-linter?lint=0019_insecure_queue_exposed_in_api",
    "splinter/security/materializedViewInApi": "https://supabase.com/docs/guides/database/database-linter?lint=0016_materialized_view_in_api",
    "splinter/security/policyExistsRlsDisabled": "https://supabase.com/docs/guides/database/database-linter?lint=0007_policy_exists_rls_disabled",
    "splinter/security/rlsDisabledInPublic": "https://supabase.com/docs/guides/database/database-linter?lint=0013_rls_disabled_in_public",
    "splinter/security/rlsEnabledNoPolicy": "https://supabase.com/docs/guides/database/database-linter?lint=0008_rls_enabled_no_policy",
    "splinter/security/rlsPolicyAlwaysTrue": "https://supabase.com/docs/guides/database/database-linter?lint=0024_permissive_rls_policy",
    "splinter/security/rlsReferencesUserMetadata": "https://supabase.com/docs/guides/database/database-linter?lint=0015_rls_references_user_metadata",
    "splinter/security/securityDefinerView": "https://supabase.com/docs/guides/database/database-linter?lint=0010_security_definer_view",
    "splinter/security/sensitiveColumnsExposed": "https://supabase.com/docs/guides/database/database-linter?lint=0023_sensitive_columns_exposed",
    "splinter/security/unsupportedRegTypes": "https://supabase.com/docs/guides/database/database-linter?lint=unsupported_reg_types",
    // splinter rules end
    ;
    // General categories
    "stdin",
    "check",
    "format",
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
    // Lint groups end

    // Splinter groups start
    "splinter",
    "splinter/performance",
    "splinter/security",
    // Splinter groups end

    // Pglinter groups start
    "pglinter",
    "pglinter/base",
    "pglinter/cluster",
    "pglinter/schema",
    // Pglinter groups end
}
