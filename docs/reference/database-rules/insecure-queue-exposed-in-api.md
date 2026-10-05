# insecureQueueExposedInApi

**Group** [`security`](../database_rules.md#security) · **Severity** Error · **Recommended** · **Requires Supabase**  
**Diagnostic** `splinter/security/insecureQueueExposedInApi`

!!! note
    This rule needs a Supabase database. It is skipped when the Supabase roles don't exist.

## Description

Detects cases where an insecure Queue is exposed over Data APIs

## Remediation

See the [Supabase database linter docs](https://supabase.com/docs/guides/database/database-linter?lint=0019_insecure_queue_exposed_in_api).

## How to configure

```json
{
  "splinter": {
    "rules": {
      "security": {
        "insecureQueueExposedInApi": "error"
      }
    }
  }
}
```

## SQL query

??? note "Show the query"

    ```sql
    (
    select
        'insecure_queue_exposed_in_api' as "name!",
        'Insecure Queue Exposed in API' as "title!",
        'ERROR' as "level!",
        'EXTERNAL' as "facing!",
        array['SECURITY'] as "categories!",
        'Detects cases where an insecure Queue is exposed over Data APIs' as "description!",
        format(
            'Table \`%s.%s\` is public, but RLS has not been enabled.',
            n.nspname,
            c.relname
        ) as "detail!",
        'https://supabase.com/docs/guides/database/database-linter?lint=0019_insecure_queue_exposed_in_api' as "remediation!",
        jsonb_build_object(
            'schema', n.nspname,
            'name', c.relname,
            'type', 'table'
        ) as "metadata!",
        format(
            'rls_disabled_in_public_%s_%s',
            n.nspname,
            c.relname
        ) as "cache_key!"
    from
        pg_catalog.pg_class c
        join pg_catalog.pg_namespace n
            on c.relnamespace = n.oid
    where
        c.relkind in ('r', 'I') -- regular or partitioned tables
        and not c.relrowsecurity -- RLS is disabled
        and (
            pg_catalog.has_table_privilege('anon', c.oid, 'SELECT')
            or pg_catalog.has_table_privilege('authenticated', c.oid, 'SELECT')
        )
        and n.nspname = 'pgmq' -- tables in the pgmq schema
        and c.relname like 'q_%' -- only queue tables
        -- Constant requirements
        and 'pgmq_public' = any(array(select trim(unnest(string_to_array(current_setting('pgrst.db_schemas', 't'), ','))))))
    ```
