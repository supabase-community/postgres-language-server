select 
  schemaname as "schema_name!", 
  tablename as "table_name!", 
  policyname as "name!", 
  permissive as "is_permissive!", 
  roles as "role_names!", 
  cmd as "command!", 
  qual as "security_qualification", 
  with_check
from 
 pg_catalog.pg_policies;