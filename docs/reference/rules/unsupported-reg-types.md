# unsupportedRegTypes

**Diagnostic Category: `splinter/security/unsupportedRegTypes`**

**Severity**: Warning

## Description

Identifies columns using unsupported reg* types outside pg_catalog schema, which prevents database upgrades using pg_upgrade.

## Remediation

https://supabase.com/docs/guides/database/database-linter?lint=unsupported_reg_types

## SQL Query

```sql
(
select
    'unsupported_reg_types' as "name!",
    'Unsupported reg types' as "title!",
    'WARN' as "level!",
    'EXTERNAL' as "facing!",
    array['SECURITY'] as "categories!",
    'Identifies columns using unsupported reg* types outside pg_catalog schema, which prevents database upgrades using pg_upgrade.' as "description!",
    format(
        'Table \`%s.%s\` has a column \`%s\` with unsupported reg* type \`%s\`.',
        n.nspname,
        c.relname,
        a.attname,
        t.typname
    ) as "detail!",
    'https://supabase.com/docs/guides/database/database-linter?lint=unsupported_reg_types' as "remediation!",
    jsonb_build_object(
        'schema', n.nspname,
        'name', c.relname,
        'column', a.attname,
        'type', 'table'
    ) as "metadata!",
    format(
        'unsupported_reg_types_%s_%s_%s',
        n.nspname,
        c.relname,
        a.attname
    ) AS cache_key
from
    pg_catalog.pg_attribute a
    join pg_catalog.pg_class c
        on a.attrelid = c.oid
    join pg_catalog.pg_namespace n
        on c.relnamespace = n.oid
    join pg_catalog.pg_type t
        on a.atttypid = t.oid
    join pg_catalog.pg_namespace tn
        on t.typnamespace = tn.oid
where
    tn.nspname = 'pg_catalog'
    and t.typname in ('regcollation', 'regconfig', 'regdictionary', 'regnamespace', 'regoper', 'regoperator', 'regproc', 'regprocedure')
    and n.nspname not in ('pg_catalog', 'information_schema', 'pgsodium'))
```

## How to configure

```json
{
  "splinter": {
    "rules": {
      "security": {
        "unsupportedRegTypes": "error"
      }
    }
  }
}
```
