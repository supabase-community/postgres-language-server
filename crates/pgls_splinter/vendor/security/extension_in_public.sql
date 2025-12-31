-- meta: name = extensionInPublic
-- meta: title = Extension in Public
-- meta: severity = WARN
-- meta: category = SECURITY
-- meta: description = Detects extensions installed in the \`public\` schema.
-- meta: remediation = https://supabase.com/docs/guides/database/database-linter?lint=0014_extension_in_public

(
select
    'extension_in_public' as "name!",
    'Extension in Public' as "title!",
    'WARN' as "level!",
    'EXTERNAL' as "facing!",
    array['SECURITY'] as "categories!",
    'Detects extensions installed in the \`public\` schema.' as "description!",
    format(
        'Extension \`%s\` is installed in the public schema. Move it to another schema.',
        pe.extname
    ) as "detail!",
    'https://supabase.com/docs/guides/database/database-linter?lint=0014_extension_in_public' as "remediation!",
    jsonb_build_object(
        'schema', pe.extnamespace::regnamespace,
        'name', pe.extname,
        'type', 'extension'
    ) as "metadata!",
    format(
        'extension_in_public_%s',
        pe.extname
    ) as "cache_key!"
from
    pg_catalog.pg_extension pe
where
    -- plpgsql is installed by default in public and outside user control
    -- confirmed safe
    pe.extname not in ('plpgsql')
    -- Scoping this to public is not optimal. Ideally we would use the postgres
    -- search path. That currently isn't available via SQL. In other lints
    -- we have used has_schema_privilege('anon', 'extensions', 'USAGE') but that
    -- is not appropriate here as it would evaluate true for the extensions schema
    and pe.extnamespace::regnamespace::text = 'public')
