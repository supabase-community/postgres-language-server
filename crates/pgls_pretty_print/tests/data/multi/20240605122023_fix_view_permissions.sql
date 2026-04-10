-- set view security_invoker=true to fix linter errors
alter view public.packages set (security_invoker=true);
alter view public.package_versions set (security_invoker=true);
alter view public.package_upgrades set (security_invoker=true);

-- create policies to allow anon role to read from the views
create policy packages_select_policy_anon
    on app.packages
    as permissive
    for select
    to anon
    using (true);

create policy package_versions_select_policy_anon
    on app.package_versions
    as permissive
    for select
    to anon
    using (true);

create policy package_upgrades_select_policy_anon
    on app.package_upgrades
    as permissive
    for select
    to anon
    using (true);
