do $$
begin

if (select count(*) from pg_catalog.pg_roles where rolname in ('owner', 'test_login','test_nologin')) != 3 then 

  perform pg_advisory_lock(12345);

  if not exists (
    select from pg_catalog.pg_roles
    where rolname = 'owner'
  ) then
    create role owner superuser createdb login bypassrls;
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

  perform pg_advisory_unlock(12345);

end if;


end 
$$;