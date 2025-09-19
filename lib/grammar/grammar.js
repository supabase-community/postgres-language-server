/**
 * @file A grammar specifically designed for use with the Postgres Language Server by Supabase-Community. It is tailored to provide autocompletions and other LSP features.
 * @author juleswritescode
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

module.exports = grammar({
  name: "pgls",

  rules: {
    // TODO: add the actual grammar rules
    source_file: $ => "hello"
  }
});
