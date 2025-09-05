with 
names_of_parents as (
  select m.member, r.rolname
  from pg_catalog.pg_auth_members m
  join pg_catalog.pg_roles r
  on r.oid = m.roleid
),
names_of_children as (
  select m.roleid, r.rolname
  from pg_catalog.pg_auth_members m
  join pg_catalog.pg_roles r
  on r.oid = m.member
)
select 
  r.rolname as "name!", 
  r.rolsuper as "is_super_user!", 
  r.rolcreatedb as "can_create_db!", 
  r.rolcanlogin  as "can_login!",
  r.rolbypassrls as "can_bypass_rls!",
  r.rolcreaterole as "can_create_roles!",
  -- this works even if we don't have access to pg_authid; manually verified
  shobj_description(r.oid, 'pg_authid') as "comment",
  coalesce((
    select array_agg(m.rolname)
    from names_of_parents m
    where m.member = r.oid
  ), ARRAY[]::text[]) as "member_of!",
  coalesce((
    select array_agg(m.rolname)
    from names_of_children m
    where m.roleid = r.oid
  ), ARRAY[]::text[]) as "has_member!"
from pg_catalog.pg_roles r;