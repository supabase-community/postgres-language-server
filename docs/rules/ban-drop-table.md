# banDropTable
**Diagnostic Category: `lint/safety/banDropTable`**

**Since**: `vnext`

> [!NOTE]
> This rule is recommended. A diagnostic error will appear when linting your code.

**Sources**: 
- Inspired from: <a href="https://squawkhq.com/docs/ban-drop-table" target="_blank"><code>squawk/ban-drop-table</code></a>

## Description
Dropping a table may break existing clients.

Update your application code to no longer read or write the table.

Once the table is no longer needed, you can delete it by running the command "DROP TABLE mytable;".

This command will permanently remove the table from the database and all its contents.
Be sure to back up the table before deleting it, just in case you need to restore it in the future.

## Examples

```sql
drop table some_table;
```

```sh
code-block.sql lint/safety/banDropTable ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Dropping a table may break existing clients.
  
  i Update your application code to no longer read or write the table, and only then delete the table. Be sure to create a backup.
  

```

## How to configure
```json

{
  "linter": {
    "rules": {
      "safety": {
        "banDropTable": "error"
      }
    }
  }
}

```
