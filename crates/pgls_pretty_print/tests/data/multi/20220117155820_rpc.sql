/*
create function public.create_organization(
    handle app.valid_name,
    display_name text = null,
    bio text = null,
    contact_email citext = null,
    avatar_id uuid = null
)
    returns public.organizations
    language plpgsql
as $$
begin
    -- Register the requested handle
    insert into app.handle_registry(handle, is_organization) values ($1, true);

    -- Create the organization
    insert into app.organizations(handle, display_name, bio, contact_email, avatar_id)
    values ($1, $2, $3, $4, $5);

    -- Return the org
    return org from public.organizations org where org.handle = $1;
end;
$$;

create function public.publish_package_version(
    handle app.valid_name,
    package_partial_name app.valid_name,
    version text,
    description_md text,
    sql text
)
    returns public.package_versions
    language plpgsql
as $$
declare
    acc app.accounts = acc from app.accounts acc where id = auth.uid();
    package_id uuid;
    package_version_id uuid;
begin
    -- Upsert package
    -- TODO add description or markdown object
    insert into app.packages(partial_name, handle, description_md)
        values (package_partial_name, package_handle)
        on conflict do update
        set description_md = excluded.description_md
        returning id
        into package_id;

    -- Insert package_version
    insert into app.package_versions(package_id, version_struct, sql)
        values (
            package_id,
            app.text_to_semver(version),
            sql
        )
        returning id
        into package_version_id;

    -- Return the package version
    return pv from public.package_versions pv where pv.id = package_version_id;
end;
$$;

create function public.publish_package_upgrade(
    handle app.valid_name,
    package_partial_name app.valid_name,
    from_version text,
    to_version text,
    sql text
)
    returns public.package_versions
    language plpgsql
as $$
declare
    acc app.accounts = acc from app.accounts acc where id = auth.uid();
    package_id uuid;
    package_version_id uuid;
begin
    select
        ap.id
    from
        app.packages ap
    where
        ap.handle = $1
        and ap.partial_name = $2
    into
        package_id;

    if package_id is null then
        perform app.exception('Unknown package' || handle || '-' || package_partial_name);
    end if;

    insert into app.packages(partial_name, handle, description_md)
        values (package_partial_name, package_handle)
        on conflict do update
        set description_md = excluded.description_md
        returning id
        into package_id;

    -- Insert package_version
    insert into app.package_versions(package_id, version_struct, sql)
        values (
            package_id,
            app.text_to_semver(version),
            sql
        )
        returning id
        into package_version_id;

    -- Return the package version
    return pv from public.package_versions pv where pv.id = package_version_id;
end;
$$;

create function public.is_handle_available(handle app.valid_name)
    returns boolean
    stable
    language sql
as $$
    select
        not exists(
            select
                1
            from
                app.handle_registry hr
            where
                hr.handle = $1
        )
$$;
*/

create or replace function public.search_packages(
    handle citext default null,
    partial_name citext default null
)
    returns setof public.packages
    stable
    language sql
as $$
    select *
    from public.packages
    where
        ($1 is null or handle <% $1 or handle ~ $1)
        and
        ($2 is null or partial_name <% $2 or partial_name ~ $2)
    order by
        coalesce(extensions.similarity($1, handle), 0) + coalesce(extensions.similarity($2, partial_name), 0) desc,
        created_at desc;
$$;
