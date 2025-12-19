# Keyword Completion Implementation Plan

This plan implements partial keyword matching in the Tree-sitter grammar to enable SQL keyword autocompletion in the LSP.

## How It Works

The `completableKeyword($, "keyword", { minLength: N })` function creates a grammar rule that matches:
1. The full keyword (e.g., `keyword_select`)
2. Partial prefixes via regex (e.g., `sel|sele|selec` for SELECT with minLength: 3)
3. Partial matches are aliased as `partial_keyword:<keyword>` for identification by the LSP

## Implementation Summary

**Total keywords implemented: 40+**

### Statement-Starting Keywords (21)

| minLength | Keywords |
|-----------|----------|
| 1 | EXPLAIN, GRANT, INSERT, MERGE, TRUNCATE, VACUUM, WITH |
| 2 | ALTER, ANALYZE, CREATE, DELETE, DROP, SET, SHOW, UNLOAD, UPDATE |
| 3 | COPY, RESET, REVOKE, SELECT |
| 4 | COMMENT |

### Clause Keywords (8)

| minLength | Keywords |
|-----------|----------|
| 1 | FROM, WHERE, GROUP, HAVING, ORDER, LIMIT |
| 2 | OFFSET |
| 3 | RETURNING |

### CREATE/ALTER/DROP Subtype Keywords (12)

| minLength | Keywords |
|-----------|----------|
| 1 | VIEW, INDEX, FUNCTION, DATABASE, EXTENSION, POLICY |
| 2 | TABLE, TYPE, TRIGGER, SCHEMA, SEQUENCE |

### Other Secondary Keywords

| minLength | Keywords |
|-----------|----------|
| 1 | JOIN, INTO, VALUES |

---

## All Completed Tasks ✅

### Phase 1: High-Priority Statement Keywords
- [x] SELECT (minLength: 3)
- [x] INSERT (minLength: 1)
- [x] UPDATE (minLength: 2)
- [x] DELETE (minLength: 2)
- [x] CREATE (minLength: 2)
- [x] ALTER (minLength: 2)
- [x] DROP (minLength: 2)

### Phase 2: Medium-Priority Statement Keywords
- [x] SET (minLength: 2)
- [x] GRANT (minLength: 1)
- [x] WITH (minLength: 1)
- [x] SHOW (minLength: 2)
- [x] TRUNCATE (minLength: 1)
- [x] COPY (minLength: 3)
- [x] COMMENT (minLength: 4)
- [x] RESET (minLength: 3)
- [x] REVOKE (minLength: 3)

### Phase 3: Lower-Priority Statement Keywords
- [x] EXPLAIN (minLength: 1)
- [x] ANALYZE (minLength: 2)
- [x] VACUUM (minLength: 1)
- [x] MERGE (minLength: 1)
- [x] UNLOAD (minLength: 2)

### Phase 4a: Clause Keywords
- [x] FROM (minLength: 1)
- [x] WHERE (minLength: 1)
- [x] ORDER (minLength: 1)
- [x] GROUP (minLength: 1)
- [x] HAVING (minLength: 1)
- [x] LIMIT (minLength: 1)
- [x] OFFSET (minLength: 2)
- [x] RETURNING (minLength: 3)

### Phase 4b: CREATE/ALTER/DROP Subtypes
- [x] TABLE (minLength: 2)
- [x] VIEW (minLength: 1)
- [x] INDEX (minLength: 1)
- [x] FUNCTION (minLength: 1)
- [x] TYPE (minLength: 2)
- [x] DATABASE (minLength: 1)
- [x] SCHEMA (minLength: 2)
- [x] SEQUENCE (minLength: 2)
- [x] EXTENSION (minLength: 1)
- [x] TRIGGER (minLength: 2)
- [x] POLICY (minLength: 1)

### Phase 4c: JOIN Keywords
- [x] JOIN (minLength: 1)

### Phase 4d: Other Secondary Keywords
- [x] INTO (minLength: 1)
- [x] VALUES (minLength: 1)

---

## Testing Protocol

For each keyword change:
1. Add test SQL to `test.sql` with partial keywords
2. Run `just tree-print test.sql`
3. Verify:
   - Partial keywords parse as `partial_keyword:<name>`
   - Full keywords still parse as `keyword_<name>`
   - No grammar conflicts or ambiguity errors
4. Only proceed to next keyword if tests pass

---

## Status: ✅ COMPLETE

All phases implemented and tested.

Last updated: 2025-12-19
