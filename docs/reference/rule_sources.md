# Rule Sources
Many rules are inspired by or directly ported from other tools. This page lists the sources of each rule.

## Exclusive rules

- [ambiguousColumn](./rules/ambiguous-column.md) 
- [assignmentTypeMismatch](./rules/assignment-type-mismatch.md) 
- [banNonTransactionalStatementInTransaction](./rules/ban-non-transactional-statement-in-transaction.md) 
- [functionArgumentMismatch](./rules/function-argument-mismatch.md) 
- [functionReturnTypeMismatch](./rules/function-return-type-mismatch.md) 
- [insertColumnMismatch](./rules/insert-column-mismatch.md) 
- [invalidCast](./rules/invalid-cast.md) 
- [invalidDropTypeSignature](./rules/invalid-drop-type-signature.md) 
- [missingFromClauseEntry](./rules/missing-from-clause-entry.md) 
- [operatorTypeMismatch](./rules/operator-type-mismatch.md) 
- [unknownColumn](./rules/unknown-column.md) 
- [unknownFunction](./rules/unknown-function.md) 
- [unknownRelation](./rules/unknown-relation.md) 
- [unknownSchema](./rules/unknown-schema.md) 
- [unknownType](./rules/unknown-type.md) 

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
| [ban-alter-domain-with-add-constraint](https://squawkhq.com/docs/ban-alter-domain-with-add-constraint) |[banAlterDomainWithAddConstraint](./rules/ban-alter-domain-with-add-constraint.md) |
| [ban-alter-identity](https://squawkhq.com/docs/ban-alter-identity) |[banAlterIdentity](./rules/ban-alter-identity.md) |
| [ban-char-field](https://squawkhq.com/docs/ban-char-field) |[banCharField](./rules/ban-char-field.md) |
| [ban-concurrent-index-creation-in-transaction](https://squawkhq.com/docs/ban-concurrent-index-creation-in-transaction) |[banConcurrentIndexCreationInTransaction](./rules/ban-concurrent-index-creation-in-transaction.md) |
| [ban-create-domain-with-constraint](https://squawkhq.com/docs/ban-create-domain-with-constraint) |[banCreateDomainWithConstraint](./rules/ban-create-domain-with-constraint.md) |
| [ban-drop-column](https://squawkhq.com/docs/ban-drop-column) |[banDropColumn](./rules/ban-drop-column.md) |
| [ban-drop-constraint](https://squawkhq.com/docs/ban-drop-constraint) |[banDropConstraint](./rules/ban-drop-constraint.md) |
| [ban-drop-database](https://squawkhq.com/docs/ban-drop-database) |[banDropDatabase](./rules/ban-drop-database.md) |
| [ban-drop-default](https://squawkhq.com/docs/ban-drop-default) |[banDropDefault](./rules/ban-drop-default.md) |
| [ban-drop-domain](https://squawkhq.com/docs/ban-drop-domain) |[banDropDomain](./rules/ban-drop-domain.md) |
| [ban-drop-extension](https://squawkhq.com/docs/ban-drop-extension) |[banDropExtension](./rules/ban-drop-extension.md) |
| [ban-drop-function](https://squawkhq.com/docs/ban-drop-function) |[banDropFunction](./rules/ban-drop-function.md) |
| [ban-drop-generated-expression](https://squawkhq.com/docs/ban-drop-generated-expression) |[banDropGeneratedExpression](./rules/ban-drop-generated-expression.md) |
| [ban-drop-not-null](https://squawkhq.com/docs/ban-drop-not-null) |[banDropNotNull](./rules/ban-drop-not-null.md) |
| [ban-drop-sequence](https://squawkhq.com/docs/ban-drop-sequence) |[banDropSequence](./rules/ban-drop-sequence.md) |
| [ban-drop-table](https://squawkhq.com/docs/ban-drop-table) |[banDropTable](./rules/ban-drop-table.md) |
| [ban-drop-type](https://squawkhq.com/docs/ban-drop-type) |[banDropType](./rules/ban-drop-type.md) |
| [ban-drop-view](https://squawkhq.com/docs/ban-drop-view) |[banDropView](./rules/ban-drop-view.md) |
| [ban-duplicate-column-assignments](https://squawkhq.com/docs/ban-duplicate-column-assignments) |[banDuplicateColumnAssignments](./rules/ban-duplicate-column-assignments.md) |
| [ban-set-schema](https://squawkhq.com/docs/ban-set-schema) |[banSetSchema](./rules/ban-set-schema.md) |
| [ban-truncate-cascade](https://squawkhq.com/docs/ban-truncate-cascade) |[banTruncateCascade](./rules/ban-truncate-cascade.md) |
| [ban-uncommitted-transaction](https://squawkhq.com/docs/ban-uncommitted-transaction) |[banUncommittedTransaction](./rules/ban-uncommitted-transaction.md) |
| [changing-column-type](https://squawkhq.com/docs/changing-column-type) |[changingColumnType](./rules/changing-column-type.md) |
| [constraint-missing-not-valid](https://squawkhq.com/docs/constraint-missing-not-valid) |[constraintMissingNotValid](./rules/constraint-missing-not-valid.md) |
| [disallow-unique-constraint](https://squawkhq.com/docs/disallow-unique-constraint) |[disallowUniqueConstraint](./rules/disallow-unique-constraint.md) |
| [identifier-too-long](https://squawkhq.com/docs/identifier-too-long) |[identifierTooLong](./rules/identifier-too-long.md) |
| [prefer-big-int](https://squawkhq.com/docs/prefer-big-int) |[preferBigInt](./rules/prefer-big-int.md) |
| [prefer-identity](https://squawkhq.com/docs/prefer-identity) |[preferIdentity](./rules/prefer-identity.md) |
| [prefer-robust-stmts](https://squawkhq.com/docs/prefer-robust-stmts) |[preferRobustStmts](./rules/prefer-robust-stmts.md) |
| [prefer-text-field](https://squawkhq.com/docs/prefer-text-field) |[preferTextField](./rules/prefer-text-field.md) |
| [prefer-timestamptz](https://squawkhq.com/docs/prefer-timestamptz) |[preferTimestamptz](./rules/prefer-timestamptz.md) |
| [renaming-column](https://squawkhq.com/docs/renaming-column) |[renamingColumn](./rules/renaming-column.md) |
| [renaming-object](https://squawkhq.com/docs/renaming-object) |[renamingObject](./rules/renaming-object.md) |
| [renaming-table](https://squawkhq.com/docs/renaming-table) |[renamingTable](./rules/renaming-table.md) |
| [require-concurrent-index-creation](https://squawkhq.com/docs/require-concurrent-index-creation) |[requireConcurrentIndexCreation](./rules/require-concurrent-index-creation.md) |
| [require-concurrent-index-deletion](https://squawkhq.com/docs/require-concurrent-index-deletion) |[requireConcurrentIndexDeletion](./rules/require-concurrent-index-deletion.md) |
| [require-enum-value-ordering](https://squawkhq.com/docs/require-enum-value-ordering) |[requireEnumValueOrdering](./rules/require-enum-value-ordering.md) |
| [require-table-schema](https://squawkhq.com/docs/require-table-schema) |[requireTableSchema](./rules/require-table-schema.md) |
| [transaction-nesting](https://squawkhq.com/docs/transaction-nesting) |[transactionNesting](./rules/transaction-nesting.md) |

### pgfence

| pgfence Rule Name | Rule Name |
| ---- | ---- |
| [add-constraint-exclude](https://github.com/flvmnt/pgfence) |[avoidAddingExclusionConstraint](./rules/avoid-adding-exclusion-constraint.md) |
| [alter-enum-add-value](https://github.com/flvmnt/pgfence) |[avoidAlterEnumAddValue](./rules/avoid-alter-enum-add-value.md) |
| [attach-partition](https://github.com/flvmnt/pgfence) |[avoidAttachingPartition](./rules/avoid-attaching-partition.md) |
| [create-trigger](https://github.com/flvmnt/pgfence) |[avoidCreateTrigger](./rules/avoid-create-trigger.md) |
| [delete-without-where](https://github.com/flvmnt/pgfence) |[banDeleteWithoutWhere](./rules/ban-delete-without-where.md) |
| [detach-partition](https://github.com/flvmnt/pgfence) |[requireConcurrentDetachPartition](./rules/require-concurrent-detach-partition.md) |
| [drop-schema](https://github.com/flvmnt/pgfence) |[banDropSchema](./rules/ban-drop-schema.md) |
| [drop-trigger](https://github.com/flvmnt/pgfence) |[banDropTrigger](./rules/ban-drop-trigger.md) |
| [enable-disable-trigger](https://github.com/flvmnt/pgfence) |[avoidEnableDisableTrigger](./rules/avoid-enable-disable-trigger.md) |
| [missing-idle-timeout](https://github.com/flvmnt/pgfence) |[requireIdleInTransactionTimeout](./rules/require-idle-in-transaction-timeout.md) |
| [missing-statement-timeout](https://github.com/flvmnt/pgfence) |[requireStatementTimeout](./rules/require-statement-timeout.md) |
| [not-valid-validate-same-tx](https://github.com/flvmnt/pgfence) |[requireSeparateConstraintValidation](./rules/require-separate-constraint-validation.md) |
| [refresh-matview-blocking](https://github.com/flvmnt/pgfence) |[requireConcurrentRefreshMatview](./rules/require-concurrent-refresh-matview.md) |
| [reindex-non-concurrent](https://github.com/flvmnt/pgfence) |[requireConcurrentReindex](./rules/require-concurrent-reindex.md) |
| [truncate](https://github.com/flvmnt/pgfence) |[banTruncate](./rules/ban-truncate.md) |
| [update-in-migration](https://github.com/flvmnt/pgfence) |[banUpdateWithoutWhere](./rules/ban-update-without-where.md) |
| [vacuum-full](https://github.com/flvmnt/pgfence) |[banVacuumFull](./rules/ban-vacuum-full.md) |
| [wide-lock-window](https://github.com/flvmnt/pgfence) |[avoidWideLockWindow](./rules/avoid-wide-lock-window.md) |
