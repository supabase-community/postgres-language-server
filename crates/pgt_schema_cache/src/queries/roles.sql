select 
  rolname as "name!", 
  rolsuper as "is_super_user!", 
  rolcreatedb as "can_create_db!", 
  rolcanlogin  as "can_login!",
  rolbypassrls as "can_bypass_rls!"
from pg_catalog.pg_roles;