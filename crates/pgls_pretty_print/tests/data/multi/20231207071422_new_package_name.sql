create or replace function app.to_package_name(handle app.valid_name, partial_name app.valid_name)
    returns text
    immutable
    language sql
as $$
    select format('%s@%s', $1, $2)
$$;

alter table app.packages
add column package_alias text null;

update app.packages
set package_alias = format('%s@%s', handle, partial_name);

-- add package_alias column to the views
create or replace view public.packages as
    select
        pa.id,
        pa.package_name,
        pa.handle,
        pa.partial_name,
        newest_ver.version as latest_version,
        newest_ver.description_md,
        pa.control_description,
        pa.control_requires,
        pa.created_at,
        pa.default_version,
        pa.package_alias
    from
        app.packages pa,
        lateral (
            select *
            from app.package_versions pv
            where pv.package_id = pa.id
            order by pv.version_struct desc
            limit 1
        ) newest_ver;

create or replace view public.package_versions as
    select
        pv.id,
        pv.package_id,
        pa.package_name,
        pv.version,
        pv.sql,
        pv.description_md,
        pa.control_description,
        pa.control_requires,
        pv.created_at,
        pa.package_alias
    from
        app.packages pa
        join app.package_versions pv
            on pa.id = pv.package_id;

create or replace view public.package_upgrades
    as
    select
        pu.id,
        pu.package_id,
        pa.package_name,
        pu.from_version,
        pu.to_version,
        pu.sql,
        pu.created_at,
        pa.package_alias
    from
        app.packages pa
        join app.package_upgrades pu
            on pa.id = pu.package_id;

create or replace function public.register_download(package_name text)
    returns void
    language sql
    security definer
    as
$$
    insert into app.downloads(package_id)
    select id
    from app.packages ap
    where ap.package_name = $1 or ap.package_alias = $1
$$;
