# addingRequiredField
**Diagnostic Category: `lint/safety/addingRequiredField`**

**Since**: `vnext`


**Sources**: 
- Inspired from: <a href="https://squawkhq.com/docs/adding-required-field" target="_blank"><code>squawk/adding-required-field</code></a>

## Description
Adding a new column that is NOT NULL and has no default value to an existing table effectively makes it required.

This will fail immediately upon running for any populated table. Furthermore, old application code that is unaware of this column will fail to INSERT to this table.

Make new columns optional initially by omitting the NOT NULL constraint until all existing data and application code has been updated. Once no NULL values are written to or persisted in the database, set it to NOT NULL.
Alternatively, if using Postgres version 11 or later, add a DEFAULT value that is not volatile. This allows the column to keep its NOT NULL constraint.

## Invalid

alter table test add column count int not null;

## Valid in Postgres >= 11

alter table test add column count int not null default 0;

## How to configure
```json

{
  "linter": {
    "rules": {
      "safety": {
        "addingRequiredField": "error"
      }
    }
  }
}

```
