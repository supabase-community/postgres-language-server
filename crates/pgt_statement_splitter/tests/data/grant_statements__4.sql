GRANT CREATE ON SCHEMA public TO anon;

GRANT SELECT, INSERT ON public.users TO anon WITH GRANT OPTION GRANTED BY Owner;

GRANT read_access, write_access TO user_role
  WITH INHERIT TRUE
  GRANTED BY security_admin;

GRANT manager_role TO employee_role
  WITH ADMIN OPTION
  GRANTED BY admin_role;
