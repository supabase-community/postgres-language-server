-- meta: name = unindexedForeignKeys
-- meta: title = Unindexed foreign keys
-- meta: severity = INFO
-- meta: category = PERFORMANCE
-- meta: description = Identifies foreign key constraints without a covering index, which can impact database performance.
-- meta: remediation = https://supabase.com/docs/guides/database/database-linter?lint=0001_unindexed_foreign_keys

with foreign_keys as (
    select
        cl.relnamespace::regnamespace::text as schema_name,
        cl.relname as table_name,
        cl.oid as table_oid,
        ct.conname as fkey_name,
        ct.conkey as col_attnums
    from
        pg_catalog.pg_constraint ct
        join pg_catalog.pg_class cl -- fkey owning table
            on ct.conrelid = cl.oid
        left join pg_catalog.pg_depend d
            on d.objid = cl.oid
            and d.deptype = 'e'
    where
        ct.contype = 'f' -- foreign key constraints
        and d.objid is null -- exclude tables that are dependencies of extensions
        and cl.relnamespace::regnamespace::text not in (
            'pg_catalog', 'information_schema', 'auth', 'storage', 'vault', 'extensions'
        )
),
index_ as (
    select
        pi.indrelid as table_oid,
        indexrelid::regclass as index_,
        string_to_array(indkey::text, ' ')::smallint[] as col_attnums
    from
        pg_catalog.pg_index pi
    where
        indisvalid
)
select
    'unindexed_foreign_keys' as "name!",
    'Unindexed foreign keys' as "title!",
    'INFO' as "level!",
    'EXTERNAL' as "facing!",
    array['PERFORMANCE'] as "categories!",
    'Identifies foreign key constraints without a covering index, which can impact database performance.' as "description!",
    format(
        'Table `%s.%s` has a foreign key `%s` without a covering index. This can lead to suboptimal query performance.',
        fk.schema_name,
        fk.table_name,
        fk.fkey_name
    ) as "detail!",
    'https://supabase.com/docs/guides/database/database-linter?lint=0001_unindexed_foreign_keys' as "remediation!",
    jsonb_build_object(
        'schema', fk.schema_name,
        'name', fk.table_name,
        'type', 'table',
        'fkey_name', fk.fkey_name,
        'fkey_columns', fk.col_attnums
    ) as "metadata!",
    format('unindexed_foreign_keys_%s_%s_%s', fk.schema_name, fk.table_name, fk.fkey_name) as "cache_key!"
from
    foreign_keys fk
    left join index_ idx
        on fk.table_oid = idx.table_oid
        and fk.col_attnums = idx.col_attnums[1:array_length(fk.col_attnums, 1)]
    left join pg_catalog.pg_depend dep
        on idx.table_oid = dep.objid
        and dep.deptype = 'e'
where
    idx.index_ is null
    and fk.schema_name not in (
        '_timescaledb_cache', '_timescaledb_catalog', '_timescaledb_config', '_timescaledb_internal', 'auth', 'cron', 'extensions', 'graphql', 'graphql_public', 'information_schema', 'net', 'pgmq', 'pgroonga', 'pgsodium', 'pgsodium_masks', 'pgtle', 'pgbouncer', 'pg_catalog', 'pgtle', 'realtime', 'repack', 'storage', 'supabase_functions', 'supabase_migrations', 'tiger', 'topology', 'vault'
    )
    and dep.objid is null -- exclude tables owned by extensions
order by
    fk.schema_name,
    fk.table_name,
    fk.fkey_name
