# Unambiguous First Keywords

This document lists statements and clauses where the **first keyword unambiguously identifies** the statement/clause type. These are candidates for simpler grammar handling since no lookahead is needed to determine the clause type.

## Top-Level Statements

These keywords at the start of a statement uniquely identify the statement type:

| Keyword | Statement | Notes |
|---------|-----------|-------|
| `SELECT` | select statement | |
| `INSERT` | insert statement | |
| `UPDATE` | update statement | |
| `DELETE` | delete statement | |
| `TRUNCATE` | truncate statement | |
| `VACUUM` | vacuum statement | |
| `GRANT` | grant statement | |
| `REVOKE` | revoke statement | |
| `SET` | set statement | |
| `RESET` | reset statement | |
| `SHOW` | show statement | |
| `COPY` | copy statement | |
| `MERGE` | merge statement | |
| `COMMENT` | comment statement | requires `ON` after |
| `EXPLAIN` | explain prefix | followed by another statement |
| `WITH` | CTE | followed by CTE definition |
| `BEGIN` | transaction/block | |

### Statements Requiring Second Keyword for Subtype

These are unambiguous as a category but need a second keyword to determine the specific subtype:

| First Keyword | Second Keyword | Statement |
|---------------|----------------|-----------|
| `CREATE` | `TABLE` | create_table |
| `CREATE` | `VIEW` | create_view |
| `CREATE` | `MATERIALIZED` | create_materialized_view |
| `CREATE` | `INDEX` | create_index |
| `CREATE` | `FUNCTION` | create_function |
| `CREATE` | `TYPE` | create_type |
| `CREATE` | `DATABASE` | create_database |
| `CREATE` | `ROLE`/`USER`/`GROUP` | create_role |
| `CREATE` | `SEQUENCE` | create_sequence |
| `CREATE` | `EXTENSION` | create_extension |
| `CREATE` | `TRIGGER` | create_trigger |
| `CREATE` | `POLICY` | create_policy |
| `CREATE` | `SCHEMA` | create_schema |
| `ALTER` | `TABLE` | alter_table |
| `ALTER` | `VIEW` | alter_view |
| `ALTER` | `SCHEMA` | alter_schema |
| `ALTER` | `TYPE` | alter_type |
| `ALTER` | `INDEX` | alter_index |
| `ALTER` | `DATABASE` | alter_database |
| `ALTER` | `ROLE` | alter_role |
| `ALTER` | `SEQUENCE` | alter_sequence |
| `ALTER` | `POLICY` | alter_policy |
| `DROP` | `TABLE` | drop_table |
| `DROP` | `VIEW` | drop_view |
| `DROP` | `INDEX` | drop_index |
| `DROP` | `TYPE` | drop_type |
| `DROP` | `SCHEMA` | drop_schema |
| `DROP` | `DATABASE` | drop_database |
| `DROP` | `ROLE`/`USER`/`GROUP` | drop_role |
| `DROP` | `SEQUENCE` | drop_sequence |
| `DROP` | `EXTENSION` | drop_extension |
| `DROP` | `FUNCTION` | drop_function |
| `DROP` | `POLICY` | drop_policy |

---

## Subclauses (Unambiguous First Keyword)

These keywords unambiguously start a subclause within their context:

| Keyword | Clause | Context |
|---------|--------|---------|
| `WHERE` | where clause | SELECT, UPDATE, DELETE, etc. |
| `FROM` | from clause | SELECT, DELETE |
| `HAVING` | having clause | after GROUP BY |
| `LIMIT` | limit clause | SELECT |
| `OFFSET` | offset clause | after LIMIT |
| `RETURNING` | returning clause | INSERT, UPDATE, DELETE |
| `VALUES` | values clause | INSERT |
| `WINDOW` | window clause | SELECT |
| `USING` | using clause | JOIN, CREATE INDEX |
| `ON` | on clause | JOIN condition |
| `INTO` | into clause | SELECT INTO, INSERT INTO |
| `CASE` | case expression | |
| `WHEN` | when clause | CASE, MERGE |
| `THEN` | then clause | CASE, MERGE |
| `ELSE` | else clause | CASE |
| `END` | end keyword | CASE, BEGIN block |
| `EXISTS` | exists expression | |
| `BETWEEN` | between expression | |
| `FILTER` | filter clause | aggregate functions |
| `OVER` | window function | |

---

## Ambiguous Keywords (Require Additional Tokens)

These keywords do NOT unambiguously identify their clause because they need additional tokens:

| Keyword | Possible Continuations | Issue |
|---------|----------------------|-------|
| `LEFT` | `JOIN`, `OUTER JOIN` | incomplete without `JOIN` |
| `RIGHT` | `JOIN`, `OUTER JOIN` | incomplete without `JOIN` |
| `FULL` | `JOIN`, `OUTER JOIN` | incomplete without `JOIN` |
| `INNER` | `JOIN` | incomplete without `JOIN` |
| `CROSS` | `JOIN` | incomplete without `JOIN` |
| `NATURAL` | `JOIN`, `LEFT JOIN`, etc. | incomplete without `JOIN` |
| `OUTER` | must follow LEFT/RIGHT/FULL | never appears first |
| `GROUP` | `BY` | incomplete without `BY` |
| `ORDER` | `BY` | incomplete without `BY` |
| `PARTITION` | `BY` (window), or partition spec | incomplete without `BY` |
| `AS` | alias, CTE body, type cast | highly context-dependent |
| `JOIN` | can follow modifiers | valid alone but also after LEFT/RIGHT/etc. |

---

## Implications for Grammar Design

### Currently Working Well
- Top-level statement keywords work well because they're at statement boundaries
- Clauses like `WHERE`, `LIMIT`, `OFFSET`, `RETURNING` work well

### Need `partialSeq` Refactoring
The following should be wrapped in `partialSeq` from the start to allow partial input:

1. **JOIN clauses**: `LEFT`, `RIGHT`, `FULL`, `INNER`, `NATURAL`, `CROSS` should be inside `partialSeq` so typing just `LEFT` is valid
2. **GROUP BY**: Should be `partialSeq(keyword_group, keyword_by, ...)`
3. **ORDER BY**: Should be `partialSeq(keyword_order, keyword_by, ...)`
4. **PARTITION BY**: Should be `partialSeq(keyword_partition, keyword_by, ...)`

### Example Fix for JOIN

Current (problematic):
```js
join: ($) =>
  seq(
    optional($.keyword_natural),
    optional(choice($.keyword_left, ...)),
    partialSeq($.keyword_join, ...)  // JOIN is required here
  ),
```

Proposed (allows partial):
```js
join: ($) =>
  partialSeq(
    optional($.keyword_natural),
    optional(choice($.keyword_left, ...)),
    $.keyword_join,
    $.relation,
    ...
  ),
```

This allows `LEFT` alone to parse as a valid partial join, with `JOIN` suggested as the next keyword.
