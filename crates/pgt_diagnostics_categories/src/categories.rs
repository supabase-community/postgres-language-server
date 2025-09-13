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
    "lint/safety/addingFieldWithDefault": "https://pgtools.dev/latest/rules/adding-field-with-default",
    "lint/safety/addingForeignKeyConstraint": "https://pgtools.dev/latest/rules/adding-foreign-key-constraint",
    "lint/safety/addingNotNullField": "https://pgtools.dev/latest/rules/adding-not-null-field",
    "lint/safety/addingPrimaryKeyConstraint": "https://pgtools.dev/latest/rules/adding-primary-key-constraint",
    "lint/safety/addingRequiredField": "https://pgtools.dev/latest/rules/adding-required-field",
    "lint/safety/banCharField": "https://pgtools.dev/latest/rules/ban-char-field",
    "lint/safety/banConcurrentIndexCreationInTransaction": "https://pgtools.dev/latest/rules/ban-concurrent-index-creation-in-transaction",
    "lint/safety/banDropColumn": "https://pgtools.dev/latest/rules/ban-drop-column",
    "lint/safety/banDropDatabase": "https://pgtools.dev/latest/rules/ban-drop-database",
    "lint/safety/banDropNotNull": "https://pgtools.dev/latest/rules/ban-drop-not-null",
    "lint/safety/banDropTable": "https://pgtools.dev/latest/rules/ban-drop-table",
    "lint/safety/banTruncateCascade": "https://pgtools.dev/latest/rules/ban-truncate-cascade",
    "lint/safety/changingColumnType": "https://pgtools.dev/latest/rules/changing-column-type",
    "lint/safety/constraintMissingNotValid": "https://pgtools.dev/latest/rules/constraint-missing-not-valid",
    "lint/safety/disallowUniqueConstraint": "https://pgtools.dev/latest/rules/disallow-unique-constraint",
    "lint/safety/preferBigInt": "https://pgtools.dev/latest/rules/prefer-big-int",
    "lint/safety/preferBigintOverInt": "https://pgtools.dev/latest/rules/prefer-bigint-over-int",
    "lint/safety/preferBigintOverSmallint": "https://pgtools.dev/latest/rules/prefer-bigint-over-smallint",
    "lint/safety/preferIdentity": "https://pgtools.dev/latest/rules/prefer-identity",
    "lint/safety/preferRobustStmts": "https://pgtools.dev/latest/rules/prefer-robust-stmts",
    "lint/safety/preferTextField": "https://pgtools.dev/latest/rules/prefer-text-field",
    "lint/safety/preferTimestamptz": "https://pgtools.dev/latest/rules/prefer-timestamptz",
    "lint/safety/renamingColumn": "https://pgtools.dev/latest/rules/renaming-column",
    "lint/safety/renamingTable": "https://pgtools.dev/latest/rules/renaming-table",
    "lint/safety/requireConcurrentIndexCreation": "https://pgtools.dev/latest/rules/require-concurrent-index-creation",
    "lint/safety/requireConcurrentIndexDeletion": "https://pgtools.dev/latest/rules/require-concurrent-index-deletion",
    "lint/safety/transactionNesting": "https://pgtools.dev/latest/rules/transaction-nesting",
    // end lint rules
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
}
