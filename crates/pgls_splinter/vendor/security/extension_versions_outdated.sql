-- meta: name = extensionVersionsOutdated
-- meta: title = Extension Versions Outdated
-- meta: severity = WARN
-- meta: category = SECURITY
-- meta: description = Detects extensions that are not using the default (recommended) version.
-- meta: remediation = https://supabase.com/docs/guides/database/database-linter?lint=0022_extension_versions_outdated

(
select
    'extension_versions_outdated' as "name!",
    'Extension Versions Outdated' as "title!",
    'WARN' as "level!",
    'EXTERNAL' as "facing!",
    array['SECURITY'] as "categories!",
    'Detects extensions that are not using the default (recommended) version.' as "description!",
    format(
        'Extension `%s` is using version `%s` but version `%s` is available. Using outdated extension versions may expose the database to security vulnerabilities.',
        ext.name,
        ext.installed_version,
        ext.default_version
    ) as "detail!",
    'https://supabase.com/docs/guides/database/database-linter?lint=0022_extension_versions_outdated' as "remediation!",
    jsonb_build_object(
        'extension_name', ext.name,
        'installed_version', ext.installed_version,
        'default_version', ext.default_version
    ) as "metadata!",
    format(
        'extension_versions_outdated_%s_%s',
        ext.name,
        ext.installed_version
    ) as "cache_key!"
from
    pg_catalog.pg_available_extensions ext
join
    -- ignore versions not in pg_available_extension_versions
    -- e.g. residue of pg_upgrade
    pg_catalog.pg_available_extension_versions extv
    on extv.name = ext.name and extv.installed
where
    ext.installed_version is not null
    and ext.default_version is not null
    and ext.installed_version != ext.default_version
order by
    ext.name)
