# Rules

Below the list of rules supported by the Postgres Language Server, divided by group. Here's a legend of the emojis:

- The icon ✅ indicates that the rule is part of the recommended rules.  

[//]: # (BEGIN RULES_INDEX)

## Safety

Rules that detect potential safety issues in your code.

| Rule name | Description | Properties |
| --- | --- | --- |
| [addSerialColumn](./rules/add-serial-column.md) | Adding a column with a SERIAL type or GENERATED ALWAYS AS ... STORED causes a full table rewrite. | ✅ |
| [addingFieldWithDefault](./rules/adding-field-with-default.md) | Adding a column with a DEFAULT value may lead to a table rewrite while holding an ACCESS EXCLUSIVE lock. | ✅ |
| [addingForeignKeyConstraint](./rules/adding-foreign-key-constraint.md) | Adding a foreign key constraint requires a table scan and a SHARE ROW EXCLUSIVE lock on both tables, which blocks writes. | ✅ |
| [addingNotNullField](./rules/adding-not-null-field.md) | Setting a column NOT NULL blocks reads while the table is scanned. | ✅ |
| [addingPrimaryKeyConstraint](./rules/adding-primary-key-constraint.md) | Adding a primary key constraint results in locks and table rewrites. | ✅ |
| [addingRequiredField](./rules/adding-required-field.md) | Adding a new column that is NOT NULL and has no default value to an existing table effectively makes it required. |  |
| [avoidAddingExclusionConstraint](./rules/avoid-adding-exclusion-constraint.md) | Adding an exclusion constraint acquires an `ACCESS EXCLUSIVE` lock. | ✅ |
| [avoidAlterEnumAddValue](./rules/avoid-alter-enum-add-value.md) | `ALTER TYPE ... ADD VALUE` cannot run inside a transaction block in older Postgres versions. |  |
| [avoidAttachingPartition](./rules/avoid-attaching-partition.md) | Attaching a partition acquires an `ACCESS EXCLUSIVE` lock on the parent table. | ✅ |
| [avoidCreateTrigger](./rules/avoid-create-trigger.md) | Creating a trigger acquires a `SHARE ROW EXCLUSIVE` lock on the table. |  |
| [avoidEnableDisableTrigger](./rules/avoid-enable-disable-trigger.md) | Enabling or disabling a trigger acquires a `SHARE ROW EXCLUSIVE` lock. |  |
| [avoidWideLockWindow](./rules/avoid-wide-lock-window.md) | Acquiring ACCESS EXCLUSIVE locks on multiple tables widens the lock window. | ✅ |
| [banCharField](./rules/ban-char-field.md) | Using CHAR(n) or CHARACTER(n) types is discouraged. |  |
| [banConcurrentIndexCreationInTransaction](./rules/ban-concurrent-index-creation-in-transaction.md) | Concurrent index creation is not allowed within a transaction. | ✅ |
| [banDeleteWithoutWhere](./rules/ban-delete-without-where.md) | A `DELETE` statement without a `WHERE` clause will remove all rows from the table. | ✅ |
| [banDropColumn](./rules/ban-drop-column.md) | Dropping a column may break existing clients. | ✅ |
| [banDropDatabase](./rules/ban-drop-database.md) | Dropping a database may break existing clients (and everything else, really). |  |
| [banDropNotNull](./rules/ban-drop-not-null.md) | Dropping a NOT NULL constraint may break existing clients. | ✅ |
| [banDropSchema](./rules/ban-drop-schema.md) | Dropping a schema will remove all objects within it and may break existing clients. | ✅ |
| [banDropTable](./rules/ban-drop-table.md) | Dropping a table may break existing clients. | ✅ |
| [banDropTrigger](./rules/ban-drop-trigger.md) | Dropping a trigger acquires an `ACCESS EXCLUSIVE` lock on the table. |  |
| [banTruncate](./rules/ban-truncate.md) | Truncating a table removes all rows and can cause data loss in production. | ✅ |
| [banTruncateCascade](./rules/ban-truncate-cascade.md) | Using `TRUNCATE`'s `CASCADE` option will truncate any tables that are also foreign-keyed to the specified tables. |  |
| [banUpdateWithoutWhere](./rules/ban-update-without-where.md) | An `UPDATE` statement without a `WHERE` clause will modify all rows in the table. | ✅ |
| [banVacuumFull](./rules/ban-vacuum-full.md) | `VACUUM FULL` rewrites the entire table and acquires an `ACCESS EXCLUSIVE` lock. | ✅ |
| [changingColumnType](./rules/changing-column-type.md) | Changing a column type may require a table rewrite and break existing clients. |  |
| [concurrentRefreshMatviewLock](./rules/concurrent-refresh-matview-lock.md) | `REFRESH MATERIALIZED VIEW CONCURRENTLY` still acquires an `EXCLUSIVE` lock. |  |
| [constraintMissingNotValid](./rules/constraint-missing-not-valid.md) | Adding constraints without NOT VALID blocks all reads and writes. |  |
| [creatingEnum](./rules/creating-enum.md) | Creating enum types is not recommended for new applications. |  |
| [disallowUniqueConstraint](./rules/disallow-unique-constraint.md) | Disallow adding a UNIQUE constraint without using an existing index. |  |
| [lockTimeoutWarning](./rules/lock-timeout-warning.md) | Taking a dangerous lock without setting a lock timeout can cause indefinite blocking. | ✅ |
| [multipleAlterTable](./rules/multiple-alter-table.md) | Multiple ALTER TABLE statements on the same table should be combined into a single statement. | ✅ |
| [preferBigInt](./rules/prefer-big-int.md) | Prefer BIGINT over smaller integer types. |  |
| [preferBigintOverInt](./rules/prefer-bigint-over-int.md) | Prefer BIGINT over INT/INTEGER types. |  |
| [preferBigintOverSmallint](./rules/prefer-bigint-over-smallint.md) | Prefer BIGINT over SMALLINT types. |  |
| [preferIdentity](./rules/prefer-identity.md) | Prefer using IDENTITY columns over serial columns. |  |
| [preferJsonb](./rules/prefer-jsonb.md) | Prefer JSONB over JSON types. |  |
| [preferRobustStmts](./rules/prefer-robust-stmts.md) | Prefer statements with guards for robustness in migrations. |  |
| [preferTextField](./rules/prefer-text-field.md) | Prefer using TEXT over VARCHAR(n) types. |  |
| [preferTimestamptz](./rules/prefer-timestamptz.md) | Prefer TIMESTAMPTZ over TIMESTAMP types. |  |
| [renamingColumn](./rules/renaming-column.md) | Renaming columns may break existing queries and application code. |  |
| [renamingTable](./rules/renaming-table.md) | Renaming tables may break existing queries and application code. |  |
| [requireConcurrentDetachPartition](./rules/require-concurrent-detach-partition.md) | Detaching a partition without `CONCURRENTLY` acquires an `ACCESS EXCLUSIVE` lock. | ✅ |
| [requireConcurrentIndexCreation](./rules/require-concurrent-index-creation.md) | Creating indexes non-concurrently can lock the table for writes. |  |
| [requireConcurrentIndexDeletion](./rules/require-concurrent-index-deletion.md) | Dropping indexes non-concurrently can lock the table for reads. |  |
| [requireConcurrentRefreshMatview](./rules/require-concurrent-refresh-matview.md) | `REFRESH MATERIALIZED VIEW` without `CONCURRENTLY` acquires an `ACCESS EXCLUSIVE` lock. | ✅ |
| [requireConcurrentReindex](./rules/require-concurrent-reindex.md) | `REINDEX` without `CONCURRENTLY` acquires an `ACCESS EXCLUSIVE` lock on the table. | ✅ |
| [requireIdleInTransactionTimeout](./rules/require-idle-in-transaction-timeout.md) | Dangerous lock statements should be preceded by `SET idle_in_transaction_session_timeout`. |  |
| [requireSeparateConstraintValidation](./rules/require-separate-constraint-validation.md) | Validating a constraint in the same transaction it was added as `NOT VALID` defeats the purpose. | ✅ |
| [requireStatementTimeout](./rules/require-statement-timeout.md) | Dangerous lock statements should be preceded by `SET statement_timeout`. |  |
| [runningStatementWhileHoldingAccessExclusive](./rules/running-statement-while-holding-access-exclusive.md) | Running additional statements while holding an ACCESS EXCLUSIVE lock blocks all table access. | ✅ |
| [transactionNesting](./rules/transaction-nesting.md) | Detects problematic transaction nesting that could lead to unexpected behavior. |  |

[//]: # (END RULES_INDEX)

