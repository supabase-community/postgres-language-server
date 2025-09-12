# Rules

Below the list of rules supported by Postgres Language Tools, divided by group. Here's a legend of the emojis:

- The icon ✅ indicates that the rule is part of the recommended rules.

[//]: # (BEGIN RULES_INDEX)

## Safety

Rules that detect potential safety issues in your code.

| Rule name | Description | Properties |
| --- | --- | --- |
| [addingFieldWithDefault](./adding-field-with-default) | Adding a column with a DEFAULT value may lead to a table rewrite while holding an ACCESS EXCLUSIVE lock. | ✅ |
| [addingForeignKeyConstraint](./adding-foreign-key-constraint) | Adding a foreign key constraint requires a table scan and a SHARE ROW EXCLUSIVE lock on both tables, which blocks writes. | ✅ |
| [addingNotNullField](./adding-not-null-field) | Setting a column NOT NULL blocks reads while the table is scanned. | ✅ |
| [addingPrimaryKeyConstraint](./adding-primary-key-constraint) | Adding a primary key constraint results in locks and table rewrites. | ✅ |
| [addingRequiredField](./adding-required-field) | Adding a new column that is NOT NULL and has no default value to an existing table effectively makes it required. |  |
| [banCharField](./ban-char-field) | Using CHAR(n) or CHARACTER(n) types is discouraged. |  |
| [banConcurrentIndexCreationInTransaction](./ban-concurrent-index-creation-in-transaction) | Concurrent index creation is not allowed within a transaction. | ✅ |
| [banDropColumn](./ban-drop-column) | Dropping a column may break existing clients. | ✅ |
| [banDropDatabase](./ban-drop-database) | Dropping a database may break existing clients (and everything else, really). |  |
| [banDropNotNull](./ban-drop-not-null) | Dropping a NOT NULL constraint may break existing clients. | ✅ |
| [banDropTable](./ban-drop-table) | Dropping a table may break existing clients. | ✅ |
| [banTruncateCascade](./ban-truncate-cascade) | Using `TRUNCATE`'s `CASCADE` option will truncate any tables that are also foreign-keyed to the specified tables. |  |
| [changingColumnType](./changing-column-type) | Changing a column type may break existing clients. |  |
| [constraintMissingNotValid](./constraint-missing-not-valid) | Adding constraints without NOT VALID blocks all reads and writes. |  |
| [disallowUniqueConstraint](./disallow-unique-constraint) | Disallow adding a UNIQUE constraint without using an existing index. |  |
| [preferBigInt](./prefer-big-int) | Prefer BIGINT over smaller integer types. |  |
| [preferBigintOverInt](./prefer-bigint-over-int) | Prefer BIGINT over INT/INTEGER types. |  |
| [preferBigintOverSmallint](./prefer-bigint-over-smallint) | Prefer BIGINT over SMALLINT types. |  |
| [preferIdentity](./prefer-identity) | Prefer using IDENTITY columns over serial columns. |  |
| [preferTextField](./prefer-text-field) | Prefer using TEXT over VARCHAR(n) types. |  |
| [preferTimestamptz](./prefer-timestamptz) | Prefer TIMESTAMPTZ over TIMESTAMP types. |  |

[//]: # (END RULES_INDEX)


