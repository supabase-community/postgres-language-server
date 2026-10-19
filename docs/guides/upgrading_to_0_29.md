# Upgrading to 0.29

Version 0.29 removes the names that earlier releases kept working with deprecation warnings. If your setup prints no deprecation warnings on the latest 0.28 release, these removals don't affect you. It also renames the schema cache to snapshot in the editor and WASM APIs, and removes the `EXPLAIN`-based type check.

## Postgres Language Server names

The project was renamed from PostgresTools. The old names are gone.

| Removed | Replacement |
| --- | --- |
| The `postgrestools` binary | `postgres-language-server` |
| The `@postgrestools/postgrestools` npm package | `@postgres-language-server/cli` |
| The `@postgrestools/backend-jsonrpc` npm package | `@postgres-language-server/backend-jsonrpc` |
| The `postgrestools.jsonc` configuration file | `postgres-language-server.jsonc` |
| `PGT_LOG_PATH` | `PGLS_LOG_PATH` |
| `PGT_LOG_LEVEL` | `PGLS_LOG_LEVEL` |
| `PGT_LOG_PREFIX_NAME` | `PGLS_LOG_PREFIX_NAME` |
| `PGT_LOG_KIND` | `PGLS_LOG_KIND` |
| `PGT_CONFIG_PATH` | `PGLS_CONFIG_PATH` |
| `-- pgt-ignore`, `-- pgt-ignore-all`, `-- pgt-ignore-start`, `-- pgt-ignore-end` | `-- pgls-ignore`, `-- pgls-ignore-all`, `-- pgls-ignore-start`, `-- pgls-ignore-end` |

The language server no longer reads `postgrestools.jsonc`, so rename the file. A `-- pgt-ignore` comment is now an ordinary comment and suppresses nothing.

## Rule names

Version 0.27 gave every rule a flat name. The former names are no longer accepted. See [Upgrading to 0.27](upgrading_to_0_27.md) for the details of the flat names.

### Configuration

| Removed | Replacement |
| --- | --- |
| `linter.rules.safety.<rule>` | `linter.rules.<rule>` |
| `linter.rules.safety.recommended` | `linter.rules.recommended`, or `linter.groups.<group>` |
| `linter.rules.safety.all` | `linter.rules.all`, or `linter.groups.<group>` |
| `linter.rules.safety.preferBigintOverInt` | `linter.rules.preferBigInt` with `{ "checkInt": true }` |
| `linter.rules.safety.preferBigintOverSmallint` | `linter.rules.preferBigInt` with `{ "checkSmallint": true }` |
| `linter.rules.safety.concurrentRefreshMatviewLock` | `linter.rules.requireConcurrentRefreshMatview` |

A configuration that still contains `linter.rules.safety` is rejected: `safety` is an unknown key in `linter.rules`.

### Suppression comments and `--only` / `--skip`

| Removed | Replacement |
| --- | --- |
| `lint/<group>/<rule>`, e.g. `lint/safety/banDropColumn` | `<rule>`, e.g. `banDropColumn` |
| `<group>/<rule>` in `--only` / `--skip`, e.g. `safety/banDropColumn` | `<rule>` |
| `lint/<group>`, e.g. `lint/destructive` | `<group>`, e.g. `destructive` |
| `lint/safety`, meaning every lint rule | `lint` |
| `safety` in `--only` / `--skip`, meaning every lint rule | No replacement: `safety` now selects the `safety` group. Leave out `--only` to run every rule. |
| `preferBigintOverInt`, `preferBigintOverSmallint` | `preferBigInt` |
| `concurrentRefreshMatviewLock` | `requireConcurrentRefreshMatview` |

A suppression comment with a removed specifier reports `Invalid rule.` and suppresses nothing. `--only` and `--skip` reject them.

## Schema cache is now snapshot

The schema the language server loads from the database is called the snapshot. The remaining schema cache names are renamed, without aliases.

| Removed | Replacement |
| --- | --- |
| The `pgls.invalidateSchemaCache` LSP command | `pgls.invalidateSnapshot` |
| The `pgls/invalidate_schema_cache` workspace method | `pgls/invalidate_snapshot` |
| The `SchemaCache` type of `@postgres-language-server/wasm` | `Snapshot` |

Editor extensions and scripts that call the command or the method by name need the new names. The code action is now titled "Invalidate Database Snapshot".

## `EXPLAIN`-based type check

Before the [`typecheck` rules](../features/type_checking.md), the language server checked statements by running `EXPLAIN` on them. Version 0.28 ran this check next to the rules, for statements that only reference objects that exist unchanged in the database. Version 0.29 removes it: the `typecheck` rules now report all type errors, and the language server no longer runs `EXPLAIN` on your statements. `plpgsql_check` still runs against the database.

`typecheck.enabled` and `typecheck.searchPath` keep working and now only affect the `typecheck` rules. Suppression comments with the `typecheck` specifier keep suppressing the `typecheck` rules.
