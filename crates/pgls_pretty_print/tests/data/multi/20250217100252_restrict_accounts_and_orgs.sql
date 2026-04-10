-- Only allow authenticated users to view their own accounts.
alter policy accounts_select_policy
    on app.accounts
    to authenticated
    using (id = auth.uid());

-- Only allow organization maintainers to view their own organizations.
alter policy organizations_select_policy
    on app.organizations
    to authenticated
    using (app.is_organization_maintainer(auth.uid(), id));

-- Allow authenticated users to get an account by handle.
create or replace function public.get_account(
    handle text
)
    returns setof public.accounts
    language sql
    security definer
    strict
as $$
    select id, handle, avatar_path, display_name, bio, created_at
    from public.accounts a
    where a.handle = get_account.handle
    and auth.uid() is not null;
$$;

-- Allow authenticated users to get an organization by handle.
create or replace function public.get_organization(
    handle text
)
    returns setof public.organizations
    language sql
    security definer
    strict
as $$
    select id, handle, avatar_path, display_name, bio, created_at
    from public.organizations o
    where o.handle = get_organization.handle
    and auth.uid() is not null;
$$;

-- Allow service role to read all accounts and organizations.
grant select on app.accounts to service_role;
grant select on app.organizations to service_role;

-- Allow service role to read all packages.
grant select on app.packages to service_role;
grant select on app.package_versions to service_role;
