# pgfence Gap: Remaining Rules & Infrastructure

Remaining gaps after the initial 11-rule PR. Based on source review of [pgfence](https://github.com/flvmnt/pgfence).

## Gaps

### 1. New Statement Rules (trivial)

#### 1a. `BanEnableDisableTrigger`
- **Match:** `AlterTableStmt` → `AlterTableCmd` with `subtype()` in `{AtEnableTrig, AtDisableTrig, AtEnableTrigAll, AtDisableTrigAll, AtEnableTrigUser, AtDisableTrigUser}`
- **Severity:** Warning, not recommended (same tier as `banCreateTrigger`)
- **Lock:** SHARE ROW EXCLUSIVE
- **pgfence ref:** `trigger.ts:48-62`, rule `enable-disable-trigger`

#### 1b. `BanUpdateWithoutWhere`
- **Match:** `UpdateStmt` where `where_clause.is_none()`
- **Severity:** Warning, recommended (same as `banDeleteWithoutWhere`)
- **pgfence ref:** `destructive.ts:83-99`, rule `update-in-migration`

#### 1c. `BanAddExclusionConstraint`
- **Match:** `AlterTableStmt` → `AlterTableCmd(AtAddConstraint)` → `Constraint` where `contype() == ConstrExclusion`
- **Severity:** Warning, recommended (no concurrent alternative exists — only mitigation is `lock_timeout`)
- **pgfence ref:** `add-constraint.ts:100-116`, rule `add-constraint-exclude`
- **Note:** Also check `CreateStmt` → `constraints` for exclusion constraints in `CREATE TABLE`

#### 1d. `WarnRefreshMatviewConcurrent`
- **Match:** `RefreshMatViewStmt` where `concurrent == true`
- **Severity:** Warning, not recommended (still takes EXCLUSIVE lock blocking DDL, but reads work)
- **pgfence ref:** `refresh-matview.ts:24-36`, rule `refresh-matview-concurrent`
- **Note:** Informational — concurrent refresh still blocks writes. Separate from `banBlockingRefreshMatview`.

### 2. Policy / Cross-Statement Rules (medium)

These require extending `TransactionState` in `linter_context.rs`.

#### 2a. `RequireStatementTimeout`
- **Condition:** File contains a dangerous lock statement (ALTER TABLE, CREATE INDEX non-concurrent) but no `SET statement_timeout` or `SET LOCAL statement_timeout` before it.
- **Pattern:** Same as existing `lockTimeoutWarning` — extend `TransactionState` with `has_statement_timeout()`.
- **Severity:** Warning, not recommended (opt-in policy)
- **pgfence ref:** `policy.ts:76-94`, rule `missing-statement-timeout`

#### 2b. `RequireIdleInTransactionTimeout`
- **Condition:** File contains dangerous statements but no `SET idle_in_transaction_session_timeout`.
- **Severity:** Warning, not recommended
- **pgfence ref:** `policy.ts:113-127`, rule `missing-idle-timeout`

#### 2c. `BanNotValidValidateSameTransaction`
- **Condition:** `ALTER TABLE ... ADD CONSTRAINT ... NOT VALID` followed by `ALTER TABLE ... VALIDATE CONSTRAINT` on the same constraint within the same file/transaction.
- **Detection:**
  1. On `AtAddConstraint` with `Constraint.skip_validation == true` → record constraint name in `TransactionState`
  2. On `AtValidateConstraint` → check if constraint name was recorded → emit diagnostic
  3. Reset on `COMMIT`/`ROLLBACK` (already tracked by `TransactionState`)
- **Severity:** Error, recommended (defeats the purpose of NOT VALID)
- **pgfence ref:** `policy.ts:134-172`, rule `not-valid-validate-same-tx`
- **Impl:** Extend `TransactionState` with `not_valid_constraints: HashSet<String>`. The constraint name comes from `AlterTableCmd.name` for validate, and from `Constraint.conname` for add.

#### 2d. `WarnWideLockWindow`
- **Condition:** Within a transaction, ACCESS EXCLUSIVE held on table A, then ACCESS EXCLUSIVE acquired on table B.
- **Detection:** Track `access_exclusive_tables: HashSet<String>` in `TransactionState`. On any statement that takes ACCESS EXCLUSIVE, check if set is non-empty with a different table.
- **Severity:** Warning, recommended
- **pgfence ref:** `policy.ts:174-203` + `transaction-state.ts:37-58`, rule `wide-lock-window`
- **Note:** Requires knowing which statements take ACCESS EXCLUSIVE. `AlterTableStmt` and non-concurrent `IndexStmt` are the main ones.

### 3. Infrastructure Improvements (larger)

#### 3a. `TransactionState` extensions
File: `crates/pgls_analyser/src/linter_context.rs`

Add fields:
- `statement_timeout_set: bool`
- `idle_in_transaction_timeout_set: bool`
- `not_valid_constraints: HashSet<String>` — constraint names added with NOT VALID in current tx
- `access_exclusive_tables: HashSet<(String, String)>` — (schema, table) pairs with ACCESS EXCLUSIVE in current tx

The existing `TransactionState` builder in `FileContext` already processes `VariableSetStmt` for `lock_timeout`. Extend it to also detect `statement_timeout` and `idle_in_transaction_session_timeout`. For NOT VALID tracking, process `AlterTableCmd` nodes as they're visited.

Check: `linter_context.rs` `FileContext::new()` — this is where `TransactionState` is built from `previous_stmts`. The constraint tracking needs to happen at statement-processing time, not just at context creation.

#### 3b. ALTER COLUMN TYPE widening (deferred)
pgfence distinguishes safe type changes (e.g., `varchar(N)` → `text`) from dangerous ones. Their approach: classify by **target type only** (source type not in the AST). Target `text` or `varchar` without length = LOW risk. Everything else = HIGH.

Our `changingColumnType` flags all type changes unconditionally. To match pgfence we'd need to inspect `ColumnDef.type_name` in the `AtAlterColumnType` command. This is doable but lower priority — our current rule is more conservative (fewer false negatives, more false positives).

**Defer** to a follow-up PR.

## Implementation Order

1. **1b, 1c** — `BanUpdateWithoutWhere`, `BanAddExclusionConstraint` (trivial, no infra changes)
2. **1a** — `BanEnableDisableTrigger` (trivial)
3. **3a** — `TransactionState` extensions (needed by 2a-2d)
4. **2c** — `BanNotValidValidateSameTransaction` (highest value cross-statement rule)
5. **2a, 2b** — `RequireStatementTimeout`, `RequireIdleInTransactionTimeout`
6. **2d** — `WarnWideLockWindow`
7. **1d** — `WarnRefreshMatviewConcurrent` (nice-to-have, informational)
8. **3b** — ALTER COLUMN TYPE widening (deferred)

## Out of Scope

- **DB stats risk adjustment** — pgfence adjusts risk based on row count from live DB. We do static analysis only.
- **ORM/framework SQL extraction** — pgfence extracts SQL from Prisma/Knex/Drizzle/etc. We handle raw SQL.
- **Plugin system** — pgfence supports custom rule plugins. Not needed.
- **Savepoint-level lock tracking** — pgfence tracks lock snapshots at savepoints. Our `TransactionState` resets on COMMIT/ROLLBACK but doesn't model savepoints. Low priority.
- **Cross-file table tracking** — pgfence tracks `CREATE TABLE` across files in a batch to skip false positives in later files. Our `lockTimeoutWarning` already does per-file tracking. Cross-file is a workspace-level concern, defer.
