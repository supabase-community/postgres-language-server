create view public.accounts as
    select
        acc.id,
        acc.handle,
        obj.name as avatar_path,
        acc.display_name,
        acc.bio,
        acc.contact_email,
        acc.created_at
    from
        app.accounts acc
        left join storage.objects obj
            on acc.avatar_id = obj.id;

create view public.organizations as
    select
        org.id,
        org.handle,
        obj.name as avatar_path,
        org.display_name,
        org.bio,
        org.contact_email,
        org.created_at
    from
        app.organizations org
        left join storage.objects obj
            on org.avatar_id = obj.id;

create view public.members as
    select
        aio.organization_id,
        aio.account_id,
        aio.role,
        aio.created_at
    from
        app.members aio;

create view public.packages as
    select
        pa.id,
        pa.package_name,
        pa.handle,
        pa.partial_name,
        newest_ver.version as latest_version,
        newest_ver.description_md,
        pa.control_description,
        pa.control_requires,
        pa.created_at
    from
        app.packages pa,
        lateral (
            select *
            from app.package_versions pv
            where pv.package_id = pa.id
            order by pv.version_struct
            limit 1
        ) newest_ver;

create view public.package_versions as
    select
        pv.id,
        pv.package_id,
        pa.package_name,
        pv.version,
        pv.sql,
        pv.description_md,
        pa.control_description,
        pa.control_requires,
        pv.created_at
    from
        app.packages pa
        join app.package_versions pv
            on pa.id = pv.package_id;

create view public.package_upgrades
    as
    select
        pu.id,
        pu.package_id,
        pa.package_name,
        pu.from_version,
        pu.to_version,
        pu.sql,
        pu.created_at
    from
        app.packages pa
        join app.package_upgrades pu
            on pa.id = pu.package_id;
