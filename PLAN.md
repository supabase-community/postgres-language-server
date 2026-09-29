# Plan: Next Linter and Typecheck

This plan restructures the linter and replaces the `EXPLAIN`-based typechecker with typecheck rules that run inside the linter against an in-memory catalog.

## Decisions

1. **Flat rule IDs.** A rule is identified by its name alone (`banDropColumn`), in config, suppressions, and docs. Groups are metadata used for bulk configuration, presets, and docs, not part of the rule's identity.
2. **Typecheck requires a database.** There is no database-free typecheck. We do not bundle builtin catalogs; builtins (types, functions, operators, casts) are introspected from the connected database.
3. **No replay.** We never execute DDL against the database, not even in a rolled-back transaction, and we never replay migrations across files. The connected database is assumed to reflect the state *before* the current file. Each file applies only its own statements, in memory.
4. **Typecheck becomes linter rules.** They run through the same registry, severity, and suppression machinery as lint rules. They stay separately switchable: `typecheck.enabled` toggles all typecheck rules independently of `linter.enabled`.
5. **Temporary `EXPLAIN` fallback.** Our own checker owns name resolution from day one. `EXPLAIN` keeps running only for statements that reference nothing created or changed in the current file, where the database's answer is still correct. The fallback is deleted once our expression typing reaches parity.

## Current State and Problems

### Linter

- All 52 rules live in one group, `safety`, under one category, `Lint`. The Biome-derived category → group → rule scaffolding is mostly unused (`RuleCategory::Action` and `RuleCategory::Transformation` are never used, and `Analyser.metadata` is dead code).
- The nested ID shape `lint/safety/banDropColumn` leaks into suppressions and confuses users (#747).
- Some rules duplicate or contradict each other:
  - `preferBigInt` covers both `preferBigintOverInt` and `preferBigintOverSmallint`.
  - `concurrentRefreshMatviewLock` flags `CONCURRENTLY`, while `requireConcurrentRefreshMatview` demands it.
  - `runningStatementWhileHoldingAccessExclusive`, `avoidWideLockWindow`, and `multipleAlterTable` all target "holding ACCESS EXCLUSIVE too long".
  - `lockTimeoutWarning`, `requireStatementTimeout`, and `requireIdleInTransactionTimeout` could be one rule with options.
- `addingNotNullField` (recommended) returns early on PG ≥ 11 when a database is connected (`crates/pgls_analyser/src/lint/safety/adding_not_null_field.rs:54`). `SET NOT NULL` still scans the table under ACCESS EXCLUSIVE, so the rule is effectively off for every connected user.
- The analyser overwrites every rule's span with the statement range (`crates/pgls_analyser/src/lib.rs:101`), and all 63 diagnostic sites pass `None` anyway. Every lint underlines the whole statement.
- Rules don't know which kind of file they run on. Migration-only rules fire in query files (#365, #367).
- Rules can't provide fixes (#523).
- State shared across statements (`AnalysedFileContext`, `TransactionState` in `crates/pgls_analyser/src/linter_context.rs`) is a small hand-built catalog. `created_objects` is a `Vec` searched linearly, and it treats an empty schema as `public` regardless of `search_path`.

### Typecheck

- Each statement is prepared on its own on a fresh connection (`crates/pgls_typecheck/src/lib.rs:58,76`), so nothing created earlier in the file is visible. This causes the temp-table and `pg_temp` false positives (#624, #692). The docs even document the workaround (`docs/guides/suppressions.md:44`).
- SQL function bodies are checked by swapping arguments for literal defaults. This cannot work for row-type arguments (#705), and return types are never checked (#431).
- DDL is never checked (#369).
- `SET search_path` inside the file is ignored; only the configured `typecheck.searchPath` applies.
- Only the first error per statement is reported, and typecheck can only be suppressed as a whole (`typecheck`), not per error kind.

### Other

- `pgls_pglinter` has configuration and settings (`crates/pgls_workspace/src/settings.rs:231`) but is never called.
- `pgls_type_resolver` has no dependents, and it panics on names with three parts.
- Each database-backed checker (typecheck, plpgsql_check, splinter) has its own pipeline, config section, and diagnostic category.

## Target Architecture

### Catalog: Database Snapshot Plus Changes From the Current File

We rework `pgls_schema_cache` into a catalog (working name `pgls_catalog`).

- **Base.** The schema cache loaded from the connected database. It is never written back to the database.
- **Extended introspection.** In addition to today's queries, load everything type resolution needs, builtins included:
  - `pg_type` in full: arrays, domains, pseudo-types, composite types, and categories/preferred flags.
  - `pg_proc` in full, including builtins, polymorphism, variadics, defaults, and return sets.
  - `pg_operator`.
  - `pg_cast`, including cast context (implicit, assignment, explicit).
  - The session's default `search_path`.
- **Indexed and cheap to snapshot.** Look up by `(schema, name)` through hash maps (by OID where needed), and share structure between copies (e.g. `im` or `Arc` copy-on-write) so taking a snapshot before each statement is cheap.
- **Overlay.** `catalog.apply(&stmt)` applies the file's DDL in memory:
  - Tables, including `CREATE TEMP TABLE` and `pg_temp.*`, and `CREATE TABLE AS`.
  - `ALTER TABLE` (columns, types, constraints, renames), `DROP`, and `RENAME`.
  - Views and materialized views, whose column types come from the typechecker's output types for the defining query.
  - Types, enums, domains, and composite types.
  - Functions and procedures (signature and return type).
  - Schemas.
- **Tolerant applying.** Applying `CREATE` for an existing object replaces it, so an already-applied migration that is open in the editor does not produce "already exists" noise.
- **Unknown objects.** Statements the catalog can't model (`DO` blocks, dynamic SQL, `CREATE EXTENSION`, unsupported DDL) mark the objects they affect as *unknown* instead of guessing. The overlay also records every object it created, changed, or dropped, which the `EXPLAIN` fallback needs (see below).

### Session State

`Session` replaces `TransactionState` and tracks, statement by statement:

- `search_path`, including in-file `SET search_path`
- the current role
- transaction depth, savepoints, and `BEGIN`/`COMMIT`/`ROLLBACK`
- `lock_timeout`, `statement_timeout`, and `idle_in_transaction_session_timeout`
- locks held and NOT VALID constraints (what the safety rules use today)

### Analyser Loop

```
for stmt in file:
    ctx = RuleContext { stmt, catalog: &catalog_before_stmt, session: &session, file_kind, db }
    run enabled rules (lint rules if linter.enabled, typecheck rules if typecheck.enabled && db connected)
    catalog.apply(stmt)
    session.apply(stmt)
```

- `file_kind` is `migration` when the file is inside `migrations.migrationsDir`, and `any` otherwise.
- Rules keep the spans they set. The analyser only falls back to the statement range when a rule gives none. Add span helpers that use libpg_query's `location` fields.
- Rules can provide fixes through a `fix` hook, which produces text edits via `pgls_pretty_print` or targeted replacements (#523). Examples: adding `CONCURRENTLY`, `IF NOT EXISTS`, or `NOT VALID`.

### Typecheck Rules

- **Group.** Typecheck rules live in a `typecheck` group.
- **Switches.** `typecheck.enabled` switches the whole group on or off, independently of `linter.enabled`. Per-rule severity uses the normal flat config (`linter.rules.unknownColumn`).
- **Database required.** The rules only run when a database is connected.
- **Only report when certain.** Anything that touches an unknown object, or can't be resolved with confidence, stays silent.

**Step 1: name resolution.** Builds scopes for FROM items, joins, CTEs (including recursive), subqueries, LATERAL, SQL function parameters (by name, `$n`, and `fn_name.param`), and trigger `NEW`/`OLD`.

| Rule | Checks |
| --- | --- |
| `unknownRelation` | table, view, or materialized view does not exist |
| `unknownColumn` | column does not exist on the resolved relation or record type |
| `unknownSchema` | schema does not exist (with `pg_temp` handled) |
| `ambiguousColumn` | unqualified column matches more than one relation in scope |
| `unknownFunction` | no function with that name and arity |
| `insertColumnMismatch` | INSERT target columns and values/SELECT list differ in count |
| `unknownType` | type name in casts, column definitions, or signatures does not exist |

**Step 2: expression typing.** This follows Postgres's type conversion rules ("Type Conversion" chapter of the Postgres docs), using the introspected catalog.

| Rule | Checks |
| --- | --- |
| `operatorTypeMismatch` | no operator matches the operand types |
| `functionArgumentMismatch` | no overload matches the argument types, or the call is ambiguous |
| `invalidCast` | no cast path exists between the types |
| `assignmentTypeMismatch` | INSERT/UPDATE value types not assignable to the target columns |
| `functionReturnTypeMismatch` | SQL function's final statement does not match the declared return type (#431) |
| `invalidDropTypeSignature` | `DROP TYPE` with a parameter list (#369) |

The difficult parts are operator and function overload resolution, implicit and assignment casts, polymorphic types (`anyelement`, `anyarray`, `anycompatible*`), `unknown` literals, domains, record and row types, and set-returning functions in FROM.

**Output types.** Expression typing also yields the output column types of a query. The overlay uses them for views, `CREATE TABLE AS`, and set-returning SQL functions, and hover can show them.

### Temporary `EXPLAIN` Fallback

This keeps full Postgres checking for statements whose objects the database still describes correctly, until our own expression typing reaches parity.

A statement is sent to the existing `EXPLAIN`/`PREPARE` path only if all of these hold:

1. `typecheck.enabled` is true and a database is connected.
2. The statement is a type the current path supports (DML, CTEs, `CREATE TABLE AS`, views, materialized views).
3. None of the objects it references (as resolved by the name resolution step) were created, changed, or dropped by the overlay, and none of them are unknown.
4. It is not a SQL function body with parameters that can't be replaced by literals (row or composite types, #705).

When the fallback runs, the `search_path` sent to Postgres comes from the tracked `Session`, not only from config.

**Deduplication.** Errors from the fallback whose SQLSTATE the name resolution rules already cover are dropped:

- `42P01` undefined table
- `42703` undefined column
- `3F000` invalid schema name
- `42704` undefined object, for types

Other errors (for example `42883` and `42804`) are reported under a `typecheck`-group diagnostic, so `pgls-ignore typecheck` keeps working.

**Exit criteria.** Delete the fallback and `pgls_typecheck`'s `EXPLAIN` path when the step 2 rules produce the same diagnostics as `EXPLAIN` on:

- the typecheck test suite, and
- a corpus of real-world SQL, such as the PrairieLearn repo from #482 and the ETL case from #786,

with no new false positives.

### plpgsql_check and Database Linters

- plpgsql_check stays a database-backed rule for now. It moves into the `typecheck` group and keeps its options, which move from 13 separate booleans to rule options. Once the PL/pgSQL parser (#639) lands, the name resolution and typing steps can cover PL/pgSQL bodies natively (#495, #179).
- `dblint` stays a separate command, because it works on database objects, not files. Splinter uses the same flat-ID config and suppression conventions.

## Rule Taxonomy

### IDs, Config, and Suppressions

- IDs are flat and unique across all rules: `banDropColumn`, `unknownColumn`.
- Config:

  ```jsonc
  {
    "linter": {
      "enabled": true,
      "rules": {
        "recommended": true,
        "banDropColumn": "off",
        "preferBigInt": { "level": "warn", "options": { "minimumType": "int" } },
      },
      "groups": {
        "style": "off",
      },
    },
    "typecheck": {
      "enabled": true,
      "searchPath": ["public", "app_*"],
    },
  }
  ```

- Suppressions: `-- pgls-ignore banDropColumn`, or with a group name `-- pgls-ignore style` / `-- pgls-ignore typecheck`.
- Rule docs live at `/rules/<rule-name>`.
- Each rule declares `appliesTo: migration | any`. Migration-only rules run only on files inside `migrations.migrationsDir`. If `migrationsDir` is not configured, every rule runs everywhere, as today.

### Backward Compatibility

- `linter.rules.safety.<rule>` keeps working and emits a deprecation diagnostic that points to `linter.rules.<rule>`.
- `pgls-ignore lint/safety/<rule>`, `pgls-ignore lint/safety`, and `pgls-ignore lint` keep working with a deprecation hint. The legacy `pgt-ignore` prefix keeps working.
- Removed or merged rule names map to their replacement with a deprecation diagnostic (`preferBigintOverInt` → `preferBigInt`).

### Groups

| Group | Default | Purpose |
| --- | --- | --- |
| `typecheck` | error, requires DB, switched by `typecheck.enabled` | code that fails at runtime because of names or types |
| `correctness` | error | code that fails at runtime for other reasons |
| `safety` | warn, migrations only | valid, but dangerous against a live database: locks, rewrites, blocking |
| `destructive` | warn, migrations only | loses data or breaks existing clients |
| `style` | warn, not recommended | schema design preferences |
| `security` | warn | static checks for security issues in new DDL |
| `nursery` | off | new rules until they stabilize |

### Assignment of Existing Rules

- **`correctness`**
  - `banConcurrentIndexCreationInTransaction`
  - `transactionNesting`
  - `avoidAlterEnumAddValue` (gated by the server version from the catalog)
- **`safety`**
  - `addingFieldWithDefault`, `addSerialColumn`
  - `addingForeignKeyConstraint`, `addingNotNullField`, `addingPrimaryKeyConstraint`
  - `avoidAddingExclusionConstraint`, `avoidAttachingPartition`, `avoidCreateTrigger`, `avoidEnableDisableTrigger`, `avoidWideLockWindow`
  - `banDropTrigger`, `banVacuumFull`
  - `constraintMissingNotValid`, `disallowUniqueConstraint`
  - `lockTimeoutWarning`, `requireStatementTimeout`, `requireIdleInTransactionTimeout`
  - `multipleAlterTable`, `runningStatementWhileHoldingAccessExclusive`
  - `requireConcurrentDetachPartition`, `requireConcurrentIndexCreation`, `requireConcurrentIndexDeletion`, `requireConcurrentRefreshMatview`, `requireConcurrentReindex`
  - `requireSeparateConstraintValidation`
  - `preferRobustStmts`
- **`destructive`**
  - `banDropColumn`, `banDropDatabase`, `banDropNotNull`, `banDropSchema`, `banDropTable`
  - `renamingColumn`, `renamingTable`
  - `changingColumnType`, `addingRequiredField`
  - `banTruncate`, `banTruncateCascade`
  - `banDeleteWithoutWhere`, `banUpdateWithoutWhere` (`appliesTo: any`)
- **`style`**
  - `preferBigInt` (merged)
  - `preferIdentity`, `preferJsonb`, `preferTextField`, `preferTimestamptz`
  - `banCharField`, `creatingEnum`
- **`security`**: new rules only, made possible by the catalog. Candidates:
  - a new table in an exposed schema without RLS enabled
  - a `SECURITY DEFINER` function without a fixed `search_path`
  - a policy that is always true

### Rule Consolidation

- Merge `preferBigintOverInt` and `preferBigintOverSmallint` into `preferBigInt`, with an option for the minimum flagged type.
- Delete `concurrentRefreshMatviewLock`. It contradicts `requireConcurrentRefreshMatview`.
- Fix `addingNotNullField`: remove the PG ≥ 11 early return. Instead, skip only when a validated `CHECK (col IS NOT NULL)` exists (PG 12+).
- Review `runningStatementWhileHoldingAccessExclusive`, `avoidWideLockWindow`, and `multipleAlterTable` for double reporting. Merge them or narrow each one's scope.
- Consider merging the three timeout rules into one `requireTimeouts` rule with options.

## Phases

### Phase 0: Cleanup

- Fix `addingNotNullField`.
- Merge the bigint rules and delete `concurrentRefreshMatviewLock`.
- Keep rule-provided spans in the analyser, and add span helpers.
- Delete `pgls_type_resolver`, the unused `RuleCategory` variants, and dead analyser metadata.
- Wire up `pgls_pglinter` or delete it (see Open Questions).

### Phase 1: Catalog, Session, Analyser Loop

- Rework `pgls_schema_cache` into the indexed catalog with extended introspection (`pg_operator`, `pg_cast`, full `pg_proc` and `pg_type`, default `search_path`).
- Implement `catalog.apply` for the DDL listed above, plus unknown-object tracking and a record of what the overlay touched.
- Implement `Session`, and replace `AnalysedFileContext` and `TransactionState` with it. Port the safety rules that read transaction state.
- Pass the catalog snapshot, session, `file_kind`, and database availability into `RuleContext`.
- Follow-up: completions and hover use the snapshot at the cursor, so objects created earlier in the file show up.

### Phase 2: Flat IDs and Regrouping

- Update `pgls_analyse` metadata so rules declare a group, `appliesTo`, and requirements such as the database.
- Update codegen (`just gen-lint`, `just new-lintrule`), configuration, the JSON schema, and docs for flat IDs and `linter.groups`.
- Update `pgls_suppressions` for flat IDs.
- Implement the backward-compatibility mapping.
- Move rules into their new groups.

### Phase 3: Name Resolution Rules + `EXPLAIN` Fallback

- Implement scopes and the step 1 rules in the `typecheck` group.
- Move the `EXPLAIN` path behind the fallback conditions, with deduplication and `search_path` from `Session`.
- Remove the old typecheck pipeline from `pull_file_diagnostics`. Diagnostics now come out of the analyser.
- This closes #624, #692, and #705.

### Phase 4: Expression Typing

- Implement the step 2 rules. This closes #431 and #369.
- Run both checkers on the corpus and track differences until the exit criteria are met.
- Delete the `EXPLAIN` fallback and the remaining `pgls_typecheck` `EXPLAIN` code.

### Phase 5: Fixes and Security Rules

- Add the `fix` hook, code actions in the LSP, and `check --fix` in the CLI (#523).
- Add the `security` rules.

## Issues Addressed

| Issue | Addressed by |
| --- | --- |
| #624 temp table false positive | overlay + name resolution (Phase 3) |
| #692 `pg_temp` false positive | overlay + `pg_temp` handling (Phase 3) |
| #705 row-type argument in SQL function | parameter scopes; fallback skipped for these bodies (Phase 3) |
| #431 SQL function return type mismatch | `functionReturnTypeMismatch` (Phase 4) |
| #369 `DROP TYPE` with parameters | `invalidDropTypeSignature` (Phase 4) |
| #365 / #367 drop table as error in non-migration files | `appliesTo`, `destructive` group defaults (Phase 2) |
| #747 suppression ID confusion | flat IDs (Phase 2) |
| #523 autofix | `fix` hook (Phase 5) |
| #495, #179 PL/pgSQL | plpgsql_check stays; native coverage after #639 |

## Open Questions

- `pgls_pglinter`: wire it into `dblint` or delete it?
- Group names: are `destructive` and `security` the right names?
- Should `banDeleteWithoutWhere` and `banUpdateWithoutWhere` move to `correctness` or a `suspicious` group, since they apply to query files too?
- How should the overlay treat `CREATE EXTENSION` when the extension is already installed in the connected database? Keep its objects known, or mark them unknown?
