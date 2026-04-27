# Rule Sources
Many rules are inspired by or directly ported from other tools. This page lists the sources of each rule.

## Exclusive rules

_No exclusive rules available._

## Rules from other sources

### Eugene

| Eugene Rule Name | Rule Name |
| ---- | ---- |
| [E11](https://kaveland.no/eugene/hints/E11/index.html) |[addSerialColumn](./rules/add-serial-column.md) |
| [E3](https://kaveland.no/eugene/hints/E3/index.html) |[preferJsonb](./rules/prefer-jsonb.md) |
| [E4](https://kaveland.no/eugene/hints/E4/index.html) |[runningStatementWhileHoldingAccessExclusive](./rules/running-statement-while-holding-access-exclusive.md) |
| [E9](https://kaveland.no/eugene/hints/E9/index.html) |[lockTimeoutWarning](./rules/lock-timeout-warning.md) |
| [W12](https://kaveland.no/eugene/hints/W12/index.html) |[multipleAlterTable](./rules/multiple-alter-table.md) |
| [W13](https://kaveland.no/eugene/hints/W13/index.html) |[creatingEnum](./rules/creating-enum.md) |

### Squawk

| Squawk Rule Name | Rule Name |
| ---- | ---- |
| [adding-field-with-default](https://squawkhq.com/docs/adding-field-with-default) |[addingFieldWithDefault](./rules/adding-field-with-default.md) |
| [adding-foreign-key-constraint](https://squawkhq.com/docs/adding-foreign-key-constraint) |[addingForeignKeyConstraint](./rules/adding-foreign-key-constraint.md) |
| [adding-not-null-field](https://squawkhq.com/docs/adding-not-null-field) |[addingNotNullField](./rules/adding-not-null-field.md) |
| [adding-required-field](https://squawkhq.com/docs/adding-required-field) |[addingRequiredField](./rules/adding-required-field.md) |
| [adding-serial-primary-key-field](https://squawkhq.com/docs/adding-serial-primary-key-field) |[addingPrimaryKeyConstraint](./rules/adding-primary-key-constraint.md) |
| [ban-char-field](https://squawkhq.com/docs/ban-char-field) |[banCharField](./rules/ban-char-field.md) |
| [ban-concurrent-index-creation-in-transaction](https://squawkhq.com/docs/ban-concurrent-index-creation-in-transaction) |[banConcurrentIndexCreationInTransaction](./rules/ban-concurrent-index-creation-in-transaction.md) |
| [ban-drop-column](https://squawkhq.com/docs/ban-drop-column) |[banDropColumn](./rules/ban-drop-column.md) |
| [ban-drop-database](https://squawkhq.com/docs/ban-drop-database) |[banDropDatabase](./rules/ban-drop-database.md) |
| [ban-drop-not-null](https://squawkhq.com/docs/ban-drop-not-null) |[banDropNotNull](./rules/ban-drop-not-null.md) |
| [ban-drop-table](https://squawkhq.com/docs/ban-drop-table) |[banDropTable](./rules/ban-drop-table.md) |
| [ban-truncate-cascade](https://squawkhq.com/docs/ban-truncate-cascade) |[banTruncateCascade](./rules/ban-truncate-cascade.md) |
| [changing-column-type](https://squawkhq.com/docs/changing-column-type) |[changingColumnType](./rules/changing-column-type.md) |
| [constraint-missing-not-valid](https://squawkhq.com/docs/constraint-missing-not-valid) |[constraintMissingNotValid](./rules/constraint-missing-not-valid.md) |
| [disallow-unique-constraint](https://squawkhq.com/docs/disallow-unique-constraint) |[disallowUniqueConstraint](./rules/disallow-unique-constraint.md) |
| [prefer-big-int](https://squawkhq.com/docs/prefer-big-int) |[preferBigInt](./rules/prefer-big-int.md) |
| [prefer-bigint-over-int](https://squawkhq.com/docs/prefer-bigint-over-int) |[preferBigintOverInt](./rules/prefer-bigint-over-int.md) |
| [prefer-bigint-over-smallint](https://squawkhq.com/docs/prefer-bigint-over-smallint) |[preferBigintOverSmallint](./rules/prefer-bigint-over-smallint.md) |
| [prefer-identity](https://squawkhq.com/docs/prefer-identity) |[preferIdentity](./rules/prefer-identity.md) |
| [prefer-robust-stmts](https://squawkhq.com/docs/prefer-robust-stmts) |[preferRobustStmts](./rules/prefer-robust-stmts.md) |
| [prefer-text-field](https://squawkhq.com/docs/prefer-text-field) |[preferTextField](./rules/prefer-text-field.md) |
| [prefer-timestamptz](https://squawkhq.com/docs/prefer-timestamptz) |[preferTimestamptz](./rules/prefer-timestamptz.md) |
| [renaming-column](https://squawkhq.com/docs/renaming-column) |[renamingColumn](./rules/renaming-column.md) |
| [renaming-table](https://squawkhq.com/docs/renaming-table) |[renamingTable](./rules/renaming-table.md) |
| [require-concurrent-index-creation](https://squawkhq.com/docs/require-concurrent-index-creation) |[requireConcurrentIndexCreation](./rules/require-concurrent-index-creation.md) |
| [require-concurrent-index-deletion](https://squawkhq.com/docs/require-concurrent-index-deletion) |[requireConcurrentIndexDeletion](./rules/require-concurrent-index-deletion.md) |
| [transaction-nesting](https://squawkhq.com/docs/transaction-nesting) |[transactionNesting](./rules/transaction-nesting.md) |

### pgfence

| pgfence Rule Name | Rule Name |
| ---- | ---- |
| [add-constraint-exclude](https://github.com/flvmnt/pgfence) |[banAddExclusionConstraint](./rules/ban-add-exclusion-constraint.md) |
| [alter-enum-add-value](https://github.com/flvmnt/pgfence) |[banAlterEnumAddValue](./rules/ban-alter-enum-add-value.md) |
| [attach-partition](https://github.com/flvmnt/pgfence) |[banAttachPartition](./rules/ban-attach-partition.md) |
| [create-trigger](https://github.com/flvmnt/pgfence) |[banCreateTrigger](./rules/ban-create-trigger.md) |
| [delete-without-where](https://github.com/flvmnt/pgfence) |[banDeleteWithoutWhere](./rules/ban-delete-without-where.md) |
| [detach-partition](https://github.com/flvmnt/pgfence) |[requireConcurrentDetachPartition](./rules/require-concurrent-detach-partition.md) |
| [drop-schema](https://github.com/flvmnt/pgfence) |[banDropSchema](./rules/ban-drop-schema.md) |
| [drop-trigger](https://github.com/flvmnt/pgfence) |[banDropTrigger](./rules/ban-drop-trigger.md) |
| [enable-disable-trigger](https://github.com/flvmnt/pgfence) |[banEnableDisableTrigger](./rules/ban-enable-disable-trigger.md) |
| [missing-idle-timeout](https://github.com/flvmnt/pgfence) |[requireIdleInTransactionTimeout](./rules/require-idle-in-transaction-timeout.md) |
| [missing-statement-timeout](https://github.com/flvmnt/pgfence) |[requireStatementTimeout](./rules/require-statement-timeout.md) |
| [not-valid-validate-same-tx](https://github.com/flvmnt/pgfence) |[banNotValidValidateSameTransaction](./rules/ban-not-valid-validate-same-transaction.md) |
| [refresh-matview-blocking](https://github.com/flvmnt/pgfence) |[banBlockingRefreshMatview](./rules/ban-blocking-refresh-matview.md) |
| [refresh-matview-concurrent](https://github.com/flvmnt/pgfence) |[warnRefreshMatviewConcurrent](./rules/warn-refresh-matview-concurrent.md) |
| [reindex-non-concurrent](https://github.com/flvmnt/pgfence) |[requireConcurrentReindex](./rules/require-concurrent-reindex.md) |
| [truncate](https://github.com/flvmnt/pgfence) |[banTruncate](./rules/ban-truncate.md) |
| [update-in-migration](https://github.com/flvmnt/pgfence) |[banUpdateWithoutWhere](./rules/ban-update-without-where.md) |
| [vacuum-full](https://github.com/flvmnt/pgfence) |[banVacuumFull](./rules/ban-vacuum-full.md) |
| [wide-lock-window](https://github.com/flvmnt/pgfence) |[warnWideLockWindow](./rules/warn-wide-lock-window.md) |
