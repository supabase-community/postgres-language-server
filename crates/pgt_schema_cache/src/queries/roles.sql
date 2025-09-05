select 
  rolname as "name!", 
  rolsuper as "is_super_user!", 
  rolcreatedb as "can_create_db!", 
  rolcanlogin  as "can_login!",
  rolbypassrls as "can_bypass_rls!",
  -- this works even if we don't have access to pg_authid; manually verified
  shobj_description(oid, 'pg_authid') as "comment"
from pg_catalog.pg_roles;