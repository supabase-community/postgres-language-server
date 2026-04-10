alter view public.accounts set (security_invoker=true);
alter view public.organizations set (security_invoker=true);
alter view public.members set (security_invoker=true);
alter view public.packages set (security_invoker=true);
alter view public.package_versions set (security_invoker=true);
alter view public.package_upgrades set (security_invoker=true);
