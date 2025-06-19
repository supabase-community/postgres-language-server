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
    "lint/safety/addingRequiredField": "https://pglt.dev/linter/rules/adding-required-field",
    "lint/safety/banDropColumn": "https://pglt.dev/linter/rules/ban-drop-column",
    "lint/safety/banDropDatabase": "https://pgtools.dev/linter/rules/ban-drop-database",
    "lint/safety/banDropNotNull": "https://pglt.dev/linter/rules/ban-drop-not-null",
    "lint/safety/banDropTable": "https://pglt.dev/linter/rules/ban-drop-table",
    "lint/safety/banTruncateCascade": "https://pgtools.dev/linter/rules/ban-truncate-cascade",
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
    "internalError/panic",
    "syntax",
    "dummy",

    // Lint groups start
    "lint",
    "lint/performance",
    "lint/safety",
    // Lint groups end
}
