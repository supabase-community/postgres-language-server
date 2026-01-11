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
| [banCharField](./rules/ban-char-field.md) | Using CHAR(n) or CHARACTER(n) types is discouraged. |  |
| [banConcurrentIndexCreationInTransaction](./rules/ban-concurrent-index-creation-in-transaction.md) | Concurrent index creation is not allowed within a transaction. | ✅ |
| [banDropColumn](./rules/ban-drop-column.md) | Dropping a column may break existing clients. | ✅ |
| [banDropDatabase](./rules/ban-drop-database.md) | Dropping a database may break existing clients (and everything else, really). |  |
| [banDropNotNull](./rules/ban-drop-not-null.md) | Dropping a NOT NULL constraint may break existing clients. | ✅ |
| [banDropTable](./rules/ban-drop-table.md) | Dropping a table may break existing clients. | ✅ |
| [banTruncateCascade](./rules/ban-truncate-cascade.md) | Using `TRUNCATE`'s `CASCADE` option will truncate any tables that are also foreign-keyed to the specified tables. |  |
| [changingColumnType](./rules/changing-column-type.md) | Changing a column type may break existing clients. |  |
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
| [requireConcurrentIndexCreation](./rules/require-concurrent-index-creation.md) | Creating indexes non-concurrently can lock the table for writes. |  |
| [requireConcurrentIndexDeletion](./rules/require-concurrent-index-deletion.md) | Dropping indexes non-concurrently can lock the table for reads. |  |
| [runningStatementWhileHoldingAccessExclusive](./rules/running-statement-while-holding-access-exclusive.md) | Running additional statements while holding an ACCESS EXCLUSIVE lock blocks all table access. | ✅ |
| [transactionNesting](./rules/transaction-nesting.md) | Detects problematic transaction nesting that could lead to unexpected behavior. |  |

[//]: # (END RULES_INDEX)

