# oneOfKeywords Implementation Plan

This plan implements ambiguous keyword matching using `oneOfKeywords` to enable keyword suggestions when a user types an ambiguous prefix (e.g., "se" could be SELECT or SET).

## How It Works

The `oneOfKeywords(keywords)` function:
1. Computes all ambiguous prefixes (prefixes matching 2+ keywords)
2. For each ambiguous prefix, creates a regex pattern aliased to `any_keyword:keyword1:keyword2:...`
3. Also matches `REPLACED_TOKEN` with all possible keywords (injected by LSP at cursor)
4. Uses `prec(-10)` so it only matches when specific rules don't

## Key Distinction

- **Unambiguous prefix** (e.g., "i" for INSERT) → `partial_keyword:insert`
- **Ambiguous prefix** (e.g., "se" for SELECT/SET) → `any_keyword:select:set`
- **REPLACED_TOKEN** (cursor position) → `any_keyword:all:possible:keywords`

---

## All Completed Tasks ✅

### Phase 1: Top-Level Statement Keywords

Added to `statement` rule for statement-starting keywords.

| Prefix | Result |
|--------|--------|
| i, e, g, m, t, v, w | `partial_keyword:X` (unambiguous) |
| s | `any_keyword:select:set:show` |
| se | `any_keyword:select:set` |
| c | `any_keyword:comment:copy:create` |
| co | `any_keyword:comment:copy` |
| d | `any_keyword:delete:drop` |
| r, re | `any_keyword:reset:revoke` |
| u | `any_keyword:unload:update` |
| REPLACED_TOKEN | `any_keyword:alter:comment:copy:create:delete:drop:grant:insert:merge:reset:revoke:select:set:show:truncate:unload:update:vacuum:with` |

### Phase 2: After FROM Clause

Added to `from` rule for post-FROM keywords (WHERE, JOIN types, ORDER, etc.).

| Prefix | Result |
|--------|--------|
| j | `partial_keyword:join` (unambiguous) |
| l | `any_keyword:lateral:left:limit` |
| o | `any_keyword:offset:order` |
| w | `any_keyword:where:window` |
| REPLACED_TOKEN | All post-FROM keywords |

### Phase 3: CREATE/ALTER/DROP Subtypes

Added to `_create_statement`, `_alter_statement`, `_drop_statement` for object type keywords.

| Context | Prefix | Result |
|---------|--------|--------|
| CREATE | t | `any_keyword:table:trigger:type` |
| CREATE | s | `any_keyword:schema:sequence` |
| ALTER | t | `any_keyword:table:type` |
| DROP | t | `any_keyword:table:type` |
| All | REPLACED_TOKEN | All subtypes for that command |

### Phase 4: JOIN Types

Handled by Phase 2's `from` rule additions.

### Phase 5: INSERT Context

Added to `insert` rule for keywords after table reference.

| Input | Result |
|-------|--------|
| `INSERT INTO users REPLACED_TOKEN` | `any_keyword:default:on:select:values` |

### Phase 6: UPDATE Context

Added to `update` rule for keywords after SET clause.

| Input | Result |
|-------|--------|
| `UPDATE t SET x=1 REPLACED_TOKEN` | `any_keyword:from:returning:where` |

---

## Testing

Run `just tree-print test.sql` to verify:
- 38 total `partial_keyword` and `any_keyword` nodes in comprehensive test
- Unambiguous single letters match specific `partial_keyword:X`
- Ambiguous prefixes match `any_keyword:X:Y:Z` with correct subset
- REPLACED_TOKEN matches all possible keywords for context
- Full SQL statements still parse correctly

---

## Status: ✅ COMPLETE

All phases implemented and tested successfully.

Last updated: 2025-12-19
