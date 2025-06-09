grant create on schema public.name;

grant select, insert on public.users to anon with grant option granted by owner;

GRANT read_access, write_access TO user_role
  WITH INHERIT TRUE
  GRANTED BY security_admin;

GRANT manager_role TO employee_role
  WITH ADMIN OPTION
  GRANTED BY admin_role;
