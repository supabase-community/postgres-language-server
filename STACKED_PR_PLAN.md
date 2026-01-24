# Stacked PR Plan for PR 629

## Overview
PR 629 "feat: keyword completion in SELECT clauses" will be split into the following stacked PRs.

## PRs

### PR 1: refactor(grammar): update conflicts and remove unused keywords
**Files:**
- `crates/pgls_treesitter_grammar/grammar.js`
- `crates/pgls_treesitter_grammar/src/grammar.json` (generated)
- `crates/pgls_treesitter_grammar/src/node-types.json` (generated)
- `crates/pgls_treesitter_grammar/src/parser.c` (generated)

**Changes:**
- Remove unused keywords (`unload`, `overwrite`, `change`, `modify`)
- Update grammar conflicts (join rules, object_reference)
- Regenerate parser files

**Status:** [x] complete - https://github.com/supabase-community/postgres-language-server/pull/654

---

### PR 2: docs(grammar): add grammar guidelines
**Files:**
- `crates/pgls_treesitter_grammar/GRAMMAR_GUIDELINES.md`

**Changes:**
- Add documentation for grammar contributions
- Explain identifier types, partial grammars, and `@end` markers

**Status:** [x] complete - https://github.com/supabase-community/postgres-language-server/pull/655

---

### PR 3: feat(grammar): add benchmarks
**Files:**
- `crates/pgls_treesitter_grammar/Cargo.toml`
- `crates/pgls_treesitter_grammar/benches/lookahead_iter.rs`
- `crates/pgls_treesitter_grammar/benches/parsing.rs`
- `crates/pgls_treesitter_grammar/benches/parsing_with_existing_tree.rs`
- `Cargo.lock`

**Changes:**
- Add criterion and clap dependencies
- Add parsing benchmarks
- Add lookahead iterator benchmark

**Status:** [x] complete - https://github.com/supabase-community/postgres-language-server/pull/656

---

### PR 4: feat(treesitter): add helper module with navigation utilities
**Files:**
- `crates/pgls_treesitter/src/helper.rs`
- `crates/pgls_treesitter/src/lib.rs`

**Changes:**
- Add `goto_node_at_position`, `goto_previous_leaf`, `goto_closest_unfinished_parent_clause`
- Add `previous_sibling_completed`, `last_children_completed`
- Add `SINGLE_TOKEN_RULES` constant
- Export functions from lib.rs

**Status:** [x] complete - https://github.com/supabase-community/postgres-language-server/pull/657

---

### PR 5: feat(treesitter): add keyword and clause tracking to context
**Files:**
- `crates/pgls_treesitter/src/context/mod.rs`

**Changes:**
- Add `possible_keywords_at_position`, `previous_clause`, `current_clause` fields
- Add methods to gather keyword possibilities using lookahead iterator
- Fix `GrantStatement` enum mapping (was incorrectly `RevokeStatement`)

**Status:** [x] complete - https://github.com/supabase-community/postgres-language-server/pull/658

---

### PR 6: fix(completions): sanitization edge cases
**Files:**
- `crates/pgls_completions/src/sanitization.rs`

**Changes:**
- Use `saturating_sub` to avoid panic on position 0
- Prevent underflow in cursor position checks

**Status:** [x] complete - https://github.com/supabase-community/postgres-language-server/pull/659

---

### PR 7: feat(completions): add keyword completion provider
**Files:**
- `crates/pgls_completions/src/providers/keywords.rs`
- `crates/pgls_completions/src/providers/mod.rs`
- `crates/pgls_completions/src/complete.rs`
- `crates/pgls_completions/src/item.rs`
- `crates/pgls_completions/src/relevance.rs`
- `crates/pgls_completions/src/relevance/filtering.rs`
- `crates/pgls_completions/src/relevance/scoring.rs`
- `crates/pgls_completions/src/builder.rs`
- `crates/pgls_lsp/src/handlers/completions.rs`

**Changes:**
- Add `SqlKeyword` struct with keyword list
- Add `complete_keywords` function
- Add `Keyword` variant to `CompletionItemKind`
- Add filtering logic with speculative parsing
- Add keyword scoring (-10 to deprioritize)
- Update builder to pass shared tree for filtering

**Status:** [x] complete - https://github.com/supabase-community/postgres-language-server/pull/661

---

### PR 8: style(grammar): formatting and trailing commas
**Files:**
- `crates/pgls_treesitter_grammar/grammar.js`
- `crates/pgls_treesitter_grammar/src/grammar.json` (generated)
- `crates/pgls_treesitter_grammar/src/node-types.json` (generated)
- `crates/pgls_treesitter_grammar/src/parser.c` (generated)
- `crates/pgls_treesitter_grammar/Cargo.toml`

**Changes:**
- Add trailing commas to grammar.js
- Reorder dependencies in Cargo.toml

**Status:** [ ] pending

---

### PR 9: refactor(test_utils): add field names to tree printing
**Files:**
- `crates/pgls_test_utils/src/lib.rs`
- `crates/pgls_test_utils/src/bin/tree_print.rs`
- `crates/pgls_treesitter_grammar/tests/grammar_tests.rs`

**Changes:**
- Update print_ts_tree signature to include field names
- Update grammar tests to use new signature

**Status:** [ ] pending

---

### PR 10: fix(treesitter): fix helper and context edge cases
**Files:**
- `crates/pgls_treesitter/src/helper.rs`
- `crates/pgls_treesitter/src/context/mod.rs`
- `crates/pgls_completions/src/providers/columns.rs` (typo fix only)

**Changes:**
- Fix `previous_sibling_completed` to return true when no sibling
- Fix `delete` -> `delete_statement` clause mapping
- Fix typo "speciifcation" -> "specification"

**Status:** [ ] pending

---

### PR 11: test(completions): update snapshots and enable tests
**Files:**
- All snapshot files in `crates/pgls_completions/src/snapshots/`
- All snapshot files in `crates/pgls_treesitter_grammar/tests/snapshots/`
- `crates/pgls_completions/src/providers/columns.rs` (remove #[ignore])
- `crates/pgls_completions/src/providers/keywords.rs` (remove #[ignore])
- `crates/pgls_completions/src/providers/roles.rs`
- `crates/pgls_completions/src/providers/tables.rs`
- `crates/pgls_completions/src/relevance/filtering.rs` (remove #[ignore])
- `crates/pgls_treesitter/src/context/mod.rs` (remove #[ignore])
- `crates/pgls_completions/src/test_helper.rs`

**Changes:**
- Update all snapshots for keyword completions and field names
- Remove #[ignore] attributes from tests
- Update test helper

**Status:** [x] complete - https://github.com/supabase-community/postgres-language-server/pull/660 (needs splitting)

---

## Notes
- PRs 1-6 are independent infrastructure changes
- PR 7 is the main feature PR
- PRs 8-11 split the original PR 8 into smaller chunks
