create function app.is_organization_maintainer(account_id uuid, organization_id uuid)
    returns boolean
    language sql
    stable
as $$
    -- Does the currently authenticated user have permission to admin orgs and org members?
    select
        exists(
            select
                1
            from
                app.members m
            where
                m.account_id = $1
                and m.organization_id = $2
                and m.role = 'maintainer'
        )
$$;


create function app.is_handle_maintainer(account_id uuid, handle app.valid_name)
    returns boolean
    language sql
    stable
as $$
    select
        exists(
            select
                1
            from
                app.accounts acc
            where
                acc.id = $1
                and acc.handle = $2
        )
        or exists(
            select
                1
            from
                app.organizations o
                join app.members m
                    on o.id = m.organization_id
            where
                m.role = 'maintainer'
                and m.account_id = $1
                and o.handle = $2
            )
$$;




create function app.is_package_maintainer(account_id uuid, package_id uuid)
    returns boolean
    language sql
    stable
as $$
    select
        exists(
            select
                1
            from
                app.accounts acc
                join app.packages p
                    on acc.handle = p.handle
            where
                acc.id = $1
                and p.id = $2
        )
        or exists(
            -- current user is maintainer of org that owns the package
            select
                1
            from
                app.packages p
                join app.organizations o
                    on p.handle = o.handle
                join app.members m
                    on o.id = m.organization_id
            where
                m.role = 'maintainer'
                and m.account_id = $1
                and p.id = $2
            )
$$;


create function app.is_package_version_maintainer(account_id uuid, package_version_id uuid)
    returns boolean
    language sql
    stable
as $$
    select
        app.is_package_maintainer($1, pv.package_id)
    from
        app.package_versions pv
    where
        pv.id = $2
$$;
