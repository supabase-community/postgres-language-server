-- app.handle_registry
grant insert
    (handle, is_organization)
    on app.handle_registry
    to authenticated;


-- app.accounts
alter table app.accounts enable row level security;

grant update
    (avatar_id, display_name, bio, contact_email)
    on app.accounts
    to authenticated;

create policy accounts_update_policy
    on app.accounts
    as permissive
    for update
    to authenticated
    using (id = auth.uid());

create policy accounts_select_policy
    on app.accounts
    as permissive
    for select
    to authenticated
    using (true);

-- app.organizations
alter table app.organizations enable row level security;

grant insert
    (handle, avatar_id, display_name, bio, contact_email)
    on app.organizations
    to authenticated;

grant update
    (avatar_id, display_name, bio, contact_email)
    on app.organizations
    to authenticated;

create policy organizations_insert_policy
    on app.organizations
    as permissive
    for insert
    to authenticated
    with check (true);

create policy organizations_update_policy
    on app.organizations
    as permissive
    for update
    to authenticated
    using (app.is_organization_maintainer(auth.uid(), id));

create policy organizations_select_policy
    on app.organizations
    as permissive
    for select
    to authenticated
    using (true);

-- app.members
alter table app.members enable row level security;

grant insert
    (organization_id, account_id, role)
    on app.members
    to authenticated;

grant delete
    on app.members
    to authenticated;

create policy members_insert_policy
    on app.members
    as permissive
    for insert
    to authenticated
    with check (app.is_organization_maintainer(auth.uid(), organization_id));

create policy members_delete_policy
    on app.members
    as permissive
    for delete
    to authenticated
    using (app.is_organization_maintainer(auth.uid(), organization_id));

create policy members_select_policy
    on app.members
    as permissive
    for select
    to authenticated
    using (true);

-- app.packages
alter table app.packages enable row level security;

grant insert (partial_name, handle)
    on app.packages
    to authenticated;

create policy package_insert_policy
    on app.packages
    as permissive
    for insert
    to authenticated
    with check (app.is_handle_maintainer(auth.uid(), handle));

create policy packages_select_policy
    on app.packages
    as permissive
    for select
    to authenticated
    using (true);

-- app.package_versions
alter table app.package_versions enable row level security;

grant insert
    (package_id, version_struct, sql, description_md)
    on app.package_versions
    to authenticated;

create policy package_versions_insert_policy
    on app.package_versions
    as permissive
    for insert
    to authenticated
    with check ( app.is_package_maintainer(auth.uid(), package_id) );

create policy package_versions_update_policy
    on app.package_versions
    as permissive
    for update
    to authenticated
    using ( app.is_package_maintainer(auth.uid(), package_id) );

create policy package_versions_select_policy
    on app.package_versions
    as permissive
    for select
    to public
    using (true);

-- app.package_upgrades
alter table app.package_upgrades enable row level security;

grant insert
    (package_id, from_version_struct, to_version_struct, sql)
    on app.package_upgrades
    to authenticated;

create policy package_upgrades_insert_policy
    on app.package_upgrades
    as permissive
    for insert
    to authenticated
    with check ( app.is_package_maintainer(auth.uid(), package_id) );

create policy package_upgrades_update_policy
    on app.package_upgrades
    as permissive
    for update
    to authenticated
    using ( app.is_package_maintainer(auth.uid(), package_id) );

create policy package_upgrades_select_policy
    on app.package_upgrades
    as permissive
    for select
    to public
    using (true);

-- storage.objects

create policy storage_objects_insert_policy
    on storage.objects
    as permissive
    for insert
    to authenticated
    with check (
        app.is_handle_maintainer(
            auth.uid(),
            (string_to_array(name, '-'::text))[1]::app.valid_name
        )
    );

create policy storage_objects_select_policy
    on storage.objects
    as permissive
    for select
    to public -- all roles
    using (bucket_id = 'avatars');
