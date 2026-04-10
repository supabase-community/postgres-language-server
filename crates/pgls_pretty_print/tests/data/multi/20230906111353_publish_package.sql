grant insert (partial_name, handle, control_description)
    on app.packages
    to authenticated;

grant update (control_description)
    on app.packages
    to authenticated;

create policy packages_update_policy
    on app.packages
    as permissive
    for update
    to authenticated
    using ( app.is_package_maintainer(auth.uid(), id) );

create or replace function public.publish_package(
    package_name app.valid_name,
    package_description varchar(1000)
)
    returns void
    language plpgsql
as $$
declare
    account app.accounts = account from app.accounts account where id = auth.uid();
begin
    if account.handle is null then
        raise exception 'user not logged in';
    end if;

    insert into app.packages(handle, partial_name, control_description)
    values (account.handle, package_name, package_description)
    on conflict on constraint packages_handle_partial_name_key
    do update
    set control_description = excluded.control_description;
end;
$$;

create or replace function public.publish_package_version(
    package_name app.valid_name,
    version_source text,
    version_description text,
    version text
)
    returns uuid
    language plpgsql
as $$
declare
    account app.accounts = account from app.accounts account where id = auth.uid();
    package_id uuid;
    version_id uuid;
begin
    if account.handle is null then
        raise exception 'user not logged in';
    end if;

    select ap.id
    from app.packages ap
    where ap.handle = account.handle and ap.partial_name = publish_package_version.package_name
    into package_id;

    begin
        insert into app.package_versions(package_id, version_struct, sql, description_md)
        values (package_id, app.text_to_semver(version), version_source, version_description)
        returning id into version_id;

        return version_id;
    exception when unique_violation then
        return null;
    end;
end;
$$;

create or replace function public.publish_package_upgrade(
    package_name app.valid_name,
    upgrade_source text,
    from_version text,
    to_version text
)
    returns uuid
    language plpgsql
as $$
declare
    account app.accounts = account from app.accounts account where id = auth.uid();
    package_id uuid;
    upgrade_id uuid;
begin
    if account.handle is null then
        raise exception 'user not logged in';
    end if;

    select ap.id
    from app.packages ap
    where ap.handle = account.handle and ap.partial_name = publish_package_upgrade.package_name
    into package_id;

    begin
        insert into app.package_upgrades(package_id, from_version_struct, to_version_struct, sql)
        values (package_id, app.text_to_semver(from_version), app.text_to_semver(to_version), upgrade_source)
        returning id into upgrade_id;

        return upgrade_id;
    exception when unique_violation then
        return null;
    end;
end;
$$;
