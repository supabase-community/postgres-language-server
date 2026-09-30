# Diagnostic Suppressions

Suppress diagnostics with comments in SQL files. Rule IDs are flat and unique (for example, `banDropTable`), while groups are metadata used to suppress related rules.

## Suppressing a Rule

Place a comment above the statement that causes the diagnostic:

```sql
-- pgls-ignore banDropTable
drop table users;
```

The accepted specifiers are:

- `lint` — all lint rules.
- `banDropTable` or `lint/banDropTable` — one rule.
- `destructive` or `lint/destructive` — all rules in a group.
- `typecheck` — typecheck diagnostics and lint rules in the `typecheck` group.
- Other non-lint diagnostic categories, such as `syntax`, continue to work as category suppressions.

A colon adds an optional explanation:

```sql
-- pgls-ignore banDropTable: My startup never had any users.
drop table users;
```

### Blocks and Files

Use matching start and end specifiers to suppress a block. Equivalent forms such as `banDropTable` and `lint/banDropTable` match as the same rule.

```sql
-- pgls-ignore-start typecheck: created in this migration
alter table users drop constraint users_pkey;
alter table users add primary key (user_id);
-- pgls-ignore-end typecheck
```

Nesting is allowed, and each start needs an end with the same normalized specifier. Use `pgls-ignore-all` at the top of a file to suppress a rule/group/category throughout that file:

```sql
-- pgls-ignore-all banDropTable

drop table tasks;
drop table projects;
```

Multiple comments can suppress multiple rules for a statement:

```sql
-- pgls-ignore banDropColumn
-- pgls-ignore typecheck
alter table tasks drop column created_at;
```

## Backward Compatibility

Legacy `lint/<group>/<rule>` and `lint/<group>` forms still work, but report a deprecation warning that names the flat form. `lint/safety` keeps its former meaning: it suppresses every lint rule, like `lint`, because all rules used to be in that group. Use `safety` for today's safety group. Removed rule names `preferBigintOverInt` and `preferBigintOverSmallint` map to `preferBigInt`, also with a warning. The legacy `pgt-ignore` comment prefix still works.

## Notes

- Suppressing diagnostics disabled in your [configuration](../configuration.md) reports a warning because the suppression has no effect.
- Suppressions that match no diagnostic also report a warning.
