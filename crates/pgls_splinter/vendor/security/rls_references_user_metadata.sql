-- meta: name = rlsReferencesUserMetadata
-- meta: title = RLS references user metadata
-- meta: severity = ERROR
-- meta: category = SECURITY
-- meta: description = Detects when Supabase Auth user_metadata is referenced insecurely in a row level security (RLS) policy.
-- meta: remediation = https://supabase.com/docs/guides/database/database-linter?lint=0015_rls_references_user_metadata
-- meta: requires_supabase = true

(
with policies as (
    select
        nsp.nspname as schema_name,
        pb.tablename as table_name,
        polname as policy_name,
        qual,
        with_check
    from
        pg_catalog.pg_policy pa
        join pg_catalog.pg_class pc
            on pa.polrelid = pc.oid
        join pg_catalog.pg_namespace nsp
            on pc.relnamespace = nsp.oid
        join pg_catalog.pg_policies pb
            on pc.relname = pb.tablename
            and nsp.nspname = pb.schemaname
            and pa.polname = pb.policyname
)
select
    'rls_references_user_metadata' as "name!",
    'RLS references user metadata' as "title!",
    'ERROR' as "level!",
    'EXTERNAL' as "facing!",
    array['SECURITY'] as "categories!",
    'Detects when Supabase Auth user_metadata is referenced insecurely in a row level security (RLS) policy.' as "description!",
    format(
        'Table \`%s.%s\` has a row level security policy \`%s\` that references Supabase Auth \`user_metadata\`. \`user_metadata\` is editable by end users and should never be used in a security context.',
        schema_name,
        table_name,
        policy_name
    ) as "detail!",
    'https://supabase.com/docs/guides/database/database-linter?lint=0015_rls_references_user_metadata' as "remediation!",
    jsonb_build_object(
        'schema', schema_name,
        'name', table_name,
        'type', 'table'
    ) as "metadata!",
    format('rls_references_user_metadata_%s_%s_%s', schema_name, table_name, policy_name) as "cache_key!"
from
    policies
where
    schema_name not in (
        '_timescaledb_cache', '_timescaledb_catalog', '_timescaledb_config', '_timescaledb_internal', 'auth', 'cron', 'extensions', 'graphql', 'graphql_public', 'information_schema', 'net', 'pgmq', 'pgroonga', 'pgsodium', 'pgsodium_masks', 'pgtle', 'pgbouncer', 'pg_catalog', 'pgtle', 'realtime', 'repack', 'storage', 'supabase_functions', 'supabase_migrations', 'tiger', 'topology', 'vault'
    )
    and (
        -- Example: auth.jwt() -> 'user_metadata'
        -- False positives are possible, but it isn't practical to string match
        -- If false positive rate is too high, this expression can iterate
        qual like '%auth.jwt()%user_metadata%'
        or qual like '%current_setting(%request.jwt.claims%)%user_metadata%'
        or with_check like '%auth.jwt()%user_metadata%'
        or with_check like '%current_setting(%request.jwt.claims%)%user_metadata%'
    ))
