insert into app.package_versions(package_id, version_struct, sql, description_md)
values (
(select id from app.packages where package_name = 'supabase-dbdev'),
(0,0,4),
$pkg$

create schema dbdev;

-- base_url and api_key have been added as arguments with default values to help test locally
create or replace function dbdev.install(
    package_name text,
    base_url text default 'https://api.database.dev/rest/v1/',
    api_key text default 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6InhtdXB0cHBsZnZpaWZyYndtbXR2Iiwicm9sZSI6ImFub24iLCJpYXQiOjE2ODAxMDczNzIsImV4cCI6MTk5NTY4MzM3Mn0.z2CN0mvO2No8wSi46Gw59DFGCTJrzM0AQKsu_5k134s'
)
    returns bool
    language plpgsql
as $$
declare
    http_ext_schema regnamespace = extnamespace::regnamespace from pg_catalog.pg_extension where extname = 'http' limit 1;
    pgtle_is_available bool = true from pg_catalog.pg_extension where extname = 'pg_tle' limit 1;
    -- HTTP respones
    rec jsonb;
    status int;
    contents json;

    -- Install Record
    rec_sql text;
    rec_ver text;
    rec_from_ver text;
    rec_to_ver text;
    rec_package_name text;
    rec_description text;
    rec_requires text[];
    rec_default_ver text;
begin

    if http_ext_schema is null then
        raise exception using errcode='22000', message=format('dbdev requires the http extension and it is not available');
    end if;

    if pgtle_is_available is null then
        raise exception using errcode='22000', message=format('dbdev requires the pgtle extension and it is not available');
    end if;

    -------------------
    -- Base Versions --
    -------------------
    execute  $stmt$select row_to_json(x)
    from $stmt$ || pg_catalog.quote_ident(http_ext_schema::text) || $stmt$.http(
        (
            'GET',
            format(
                '%spackage_versions?select=package_name,version,sql,control_description,control_requires&limit=50&package_name=eq.%s',
                $stmt$ || pg_catalog.quote_literal(base_url) || $stmt$,
                $stmt$ || pg_catalog.quote_literal($1) || $stmt$
            ),
            array[
                ('apiKey', $stmt$ || pg_catalog.quote_literal(api_key) || $stmt$)::http_header
            ],
            null,
            null
        )
    ) x
    limit 1; $stmt$
    into rec;

    status = (rec ->> 'status')::int;
    contents = to_json(rec ->> 'content') #>> '{}';

    if status <> 200 then
        raise notice using errcode='22000', message=format('DBDEV INFO: %s', contents);
        raise exception using errcode='22000', message=format('Non-200 response code while loading versions from dbdev');
    end if;

    if contents is null or json_typeof(contents) <> 'array' or json_array_length(contents) = 0 then
        raise exception using errcode='22000', message=format('No versions found for package named %s', package_name);
    end if;

    for rec_package_name, rec_ver, rec_sql, rec_description, rec_requires in select
            (r ->> 'package_name'),
            (r ->> 'version'),
            (r ->> 'sql'),
            (r ->> 'control_description'),
            array(select json_array_elements_text((r -> 'control_requires')))
        from
            json_array_elements(contents) as r
        loop

        -- Install the primary version
        if not exists (
            select true
            from pgtle.available_extensions()
            where
                name = rec_package_name
        ) then
            perform pgtle.install_extension(rec_package_name, rec_ver, rec_description, rec_sql, rec_requires);
        end if;

        -- Install other available versions
        if not exists (
            select true
            from pgtle.available_extension_versions()
            where
                name = rec_package_name
                and version = rec_ver
        ) then
            perform pgtle.install_extension_version_sql(rec_package_name, rec_ver, rec_sql);
        end if;

    end loop;

    ----------------------
    -- Upgrade Versions --
    ----------------------
    execute  $stmt$select row_to_json(x)
    from $stmt$ || pg_catalog.quote_ident(http_ext_schema::text) || $stmt$.http(
        (
            'GET',
            format(
                '%spackage_upgrades?select=package_name,from_version,to_version,sql&limit=50&package_name=eq.%s',
                $stmt$ || pg_catalog.quote_literal(base_url) || $stmt$,
                $stmt$ || pg_catalog.quote_literal($1) || $stmt$
            ),
            array[
                ('apiKey', $stmt$ || pg_catalog.quote_literal(api_key) || $stmt$)::http_header
            ],
            null,
            null
        )
    ) x
    limit 1; $stmt$
    into rec;

    status = (rec ->> 'status')::int;
    contents = to_json(rec ->> 'content') #>> '{}';

    if status <> 200 then
        raise notice using errcode='22000', message=format('DBDEV INFO: %s', contents);
        raise exception using errcode='22000', message=format('Non-200 response code while loading upgrade paths from dbdev');
    end if;

    if json_typeof(contents) <> 'array' then
        raise exception using errcode='22000', message=format('Invalid response from dbdev upgrade paths');
    end if;

    for rec_package_name, rec_from_ver, rec_to_ver, rec_sql in select
            (r ->> 'package_name'),
            (r ->> 'from_version'),
            (r ->> 'to_version'),
            (r ->> 'sql')
        from
            json_array_elements(contents) as r
        loop

        if not exists (
            select true
            from pgtle.extension_update_paths(rec_package_name)
            where
                source = rec_from_ver
                and target = rec_to_ver
                and path is not null
        ) then
            perform pgtle.install_update_path(rec_package_name, rec_from_ver, rec_to_ver, rec_sql);
        end if;
    end loop;

    -------------------------
    -- Set Default Version --
    -------------------------
    execute  $stmt$select row_to_json(x)
    from $stmt$ || pg_catalog.quote_ident(http_ext_schema::text) || $stmt$.http(
        (
            'GET',
            format(
                '%spackages?select=package_name,default_version&limit=1&package_name=eq.%s',
                $stmt$ || pg_catalog.quote_literal(base_url) || $stmt$,
                $stmt$ || pg_catalog.quote_literal($1) || $stmt$
            ),
            array[
                ('apiKey', $stmt$ || pg_catalog.quote_literal(api_key) || $stmt$)::http_header
            ],
            null,
            null
        )
    ) x
    limit 1; $stmt$
    into rec;

    status = (rec ->> 'status')::int;
    contents = to_json(rec ->> 'content') #>> '{}';

    if status <> 200 then
        raise notice using errcode='22000', message=format('DBDEV INFO: %s', contents);
        raise exception using errcode='22000', message=format('Non-200 response code while loading packages from dbdev');
    end if;

    if contents is null or json_typeof(contents) <> 'array' or json_array_length(contents) = 0 then
        raise exception using errcode='22000', message=format('No package named %s found', package_name);
    end if;

    for rec_package_name, rec_default_ver in select
            (r ->> 'package_name'),
            (r ->> 'default_version')
        from
            json_array_elements(contents) as r
        loop

        if rec_default_ver is not null then
            perform pgtle.set_default_version(rec_package_name, rec_default_ver);
        else
            raise notice using errcode='22000', message=format('DBDEV INFO: missing default version');
        end if;

    end loop;

    --------------------------
    -- Send Download Notice --
    --------------------------
    -- Notifies dbdev that a package has been downloaded and records IP + user agent so we can compute unique download counts
    execute  $stmt$select row_to_json(x)
    from $stmt$ || pg_catalog.quote_ident(http_ext_schema::text) || $stmt$.http(
        (
            'POST',
            format(
                '%srpc/register_download',
                $stmt$ || pg_catalog.quote_literal(base_url) || $stmt$
            ),
            array[
                ('apiKey', $stmt$ || pg_catalog.quote_literal(api_key) || $stmt$)::http_header,
                ('x-client-info', 'dbdev/0.0.4')::http_header
            ],
            'application/json',
            json_build_object('package_name', $stmt$ || pg_catalog.quote_literal($1) || $stmt$)::text
        )
    ) x
    limit 1; $stmt$
    into rec;

    return true;
end;
$$;

$pkg$,
$description$
# dbdev

dbdev is the SQL client for database.new and is the primary way end users interact with the package (pglet) registry.

dbdev can be used to load packages from the registry. For example:

```sql
-- Load the package from the package index
select dbdev.install('olirice-index_advisor');
```
Where `olirice` is the handle of the author and `index_advisor` is the name of the pglet.

Once installed, pglets are visible in PostgreSQL as extensions. At that point they can be enabled with standard Postgres commands i.e. the `create extension`

To improve reproducibility, we recommend __always__ specifying the package version in your `create extension` statements.

For example:
```sql
-- Enable the extension
create extension "olirice-index_advisor"
    schema 'public'
    version '0.1.0';
```

Which creates all tables/indexes/functions/etc specified by the extension.

## How to Install

The in-database SQL client for the package registry is named `dbdev`. You can bootstrap the client with:

```sql
/*---------------------
---- install dbdev ----
----------------------
Requires:
  - pg_tle: https://github.com/aws/pg_tle
  - pgsql-http: https://github.com/pramsey/pgsql-http
*/
create extension if not exists http with schema extensions;
create extension if not exists pg_tle;
drop extension if exists "supabase-dbdev";
select pgtle.uninstall_extension_if_exists('supabase-dbdev');
select
    pgtle.install_extension(
        'supabase-dbdev',
        resp.contents ->> 'version',
        'PostgreSQL package manager',
        resp.contents ->> 'sql'
    )
from http(
    (
        'GET',
        'https://api.database.dev/rest/v1/'
        || 'package_versions?select=sql,version'
        || '&package_name=eq.supabase-dbdev'
        || '&order=version.desc'
        || '&limit=1',
        array[
            (
                'apiKey',
                'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJp'
                || 'c3MiOiJzdXBhYmFzZSIsInJlZiI6InhtdXB0cHBsZnZpaWZyY'
                || 'ndtbXR2Iiwicm9sZSI6ImFub24iLCJpYXQiOjE2ODAxMDczNzI'
                || 'sImV4cCI6MTk5NTY4MzM3Mn0.z2CN0mvO2No8wSi46Gw59DFGCTJ'
                || 'rzM0AQKsu_5k134s'
            )::http_header
        ],
        null,
        null
    )
) x,
lateral (
    select
        ((row_to_json(x) -> 'content') #>> '{}')::json -> 0
) resp(contents);
create extension "supabase-dbdev";
select dbdev.install('supabase-dbdev');
drop extension if exists "supabase-dbdev";
create extension "supabase-dbdev";
```

With the client ready, search for packages on [database.dev](database.dev) and install them with

```sql
select dbdev.install('handle-package_name');
create extension "handle-package_name"
    schema 'public'
    version '1.2.3';
```
$description$
);

-- set supabase-dbdev package's default_version to 0.0.4
update app.packages
set default_version_struct = app.text_to_semver('0.0.4')
where package_name = 'supabase-dbdev';
