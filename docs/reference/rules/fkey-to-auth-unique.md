# fkeyToAuthUnique

**Diagnostic Category: `splinter/security/fkeyToAuthUnique`**

**Severity**: Error

> [!NOTE]
> This rule requires a Supabase database/project and will be automatically skipped if not detected.

## Description

Detects user defined foreign keys to unique constraints in the auth schema.

[Learn More](Drop the foreign key constraint that references the auth schema.)

## SQL Query

```sql
(
select
    'fkey_to_auth_unique' as "name!",
    'Foreign Key to Auth Unique Constraint' as "title!",
    'ERROR' as "level!",
    'EXTERNAL' as "facing!",
    array['SECURITY'] as "categories!",
    'Detects user defined foreign keys to unique constraints in the auth schema.' as "description!",
    format(
        'Table `%s`.`%s` has a foreign key `%s` referencing an auth unique constraint',
        n.nspname, -- referencing schema
        c_rel.relname, -- referencing table
        c.conname -- fkey name
    ) as "detail!",
    'Drop the foreign key constraint that references the auth schema.' as "remediation!",
    jsonb_build_object(
        'schema', n.nspname,
        'name', c_rel.relname,
        'foreign_key', c.conname
    ) as "metadata!",
    format(
        'fkey_to_auth_unique_%s_%s_%s',
        n.nspname, -- referencing schema
        c_rel.relname, -- referencing table
        c.conname
    ) as "cache_key!"
from
    pg_catalog.pg_constraint c
    join pg_catalog.pg_class c_rel
        on c.conrelid = c_rel.oid
    join pg_catalog.pg_namespace n
        on c_rel.relnamespace = n.oid
    join pg_catalog.pg_class ref_rel
        on c.confrelid = ref_rel.oid
    join pg_catalog.pg_namespace cn
        on ref_rel.relnamespace = cn.oid
    join pg_catalog.pg_index i
        on c.conindid = i.indexrelid
where c.contype = 'f'
    and cn.nspname = 'auth'
    and i.indisunique
    and not i.indisprimary)
```

## How to configure

```json
{
  "splinter": {
    "rules": {
      "security": {
        "fkeyToAuthUnique": "error"
      }
    }
  }
}
```
