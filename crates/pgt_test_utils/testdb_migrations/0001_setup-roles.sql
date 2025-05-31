do $$
begin

select pg_advisory_lock(12345);

if not exists (
  select from pg_catalog.pg_roles
  where rolname = 'admin'
) then
  create role admin superuser createdb login bypassrls;
end if;

if not exists (
  select from pg_catalog.pg_roles
  where rolname = 'test_login'
) then
  create role test_login login;
end if;

if not exists (
  select from pg_catalog.pg_roles
  where rolname = 'test_nologin'
) then
  create role test_nologin;
end if;

select pg_advisory_unlock(12345);

end 
$$;