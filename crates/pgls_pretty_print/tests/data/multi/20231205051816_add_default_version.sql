-- default_version column has a default value '0.0.0' only temporarily because the column is not null.
-- It will be removed below.
alter table app.packages
add column default_version_struct app.semver not null default app.text_to_semver('0.0.0'),
add column default_version text generated always as (app.semver_to_text(default_version_struct)) stored;

-- for now we set the default version to current latest version
-- new client will allow users to set a specific default version in the control file
update app.packages
set default_version_struct = app.text_to_semver(pp.latest_version)
from public.packages pp
where packages.id = pp.id;

-- now that every row has a valid default_version, remove the default value of '0.0.0'
alter table app.packages
alter column default_version_struct drop default;

-- add new default_version column to the view
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
        pa.default_version
    from
        app.packages pa,
        lateral (
            select *
            from app.package_versions pv
            where pv.package_id = pa.id
            order by pv.version_struct desc
            limit 1
        ) newest_ver;

-- grant insert and update permissions to authenticated users on the new default_version_struct column
grant insert (partial_name, handle, control_description, control_relocatable, control_requires, default_version_struct)
    on app.packages
    to authenticated;

grant update (control_description, control_relocatable, control_requires, default_version_struct)
    on app.packages
    to authenticated;

-- publish_package accepts an additional `default_version` argument
drop function public.publish_package(app.valid_name, varchar, bool, text[]);
create or replace function public.publish_package(
    package_name app.valid_name,
    package_description varchar(1000),
    relocatable bool default false,
    requires text[] default '{}',
    default_version text default null
)
    returns void
    language plpgsql
as $$
declare
    account app.accounts = account from app.accounts account where id = auth.uid();
    require text;
begin
    if account.handle is null then
        raise exception 'user not logged in';
    end if;

    if default_version is null then
        raise exception 'default_version is required. If you are on `dbdev` CLI version 0.1.5 or older upgrade to the latest version.';
    end if;

    foreach require in array requires
    loop
        if not exists (
            select true
            from app.allowed_extensions
            where
                name = require
        ) then
            raise exception '`requires` in the control file can''t have `%` in it', require;
        end if;
    end loop;

    insert into app.packages(handle, partial_name, control_description, control_relocatable, control_requires, default_version_struct)
    values (account.handle, package_name, package_description, relocatable, requires, app.text_to_semver(default_version))
    on conflict on constraint packages_handle_partial_name_key
    do update
    set control_description = excluded.control_description,
        control_relocatable = excluded.control_relocatable,
        control_requires = excluded.control_requires,
        default_version_struct = excluded.default_version_struct;
end;
$$;
