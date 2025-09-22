create function set_role_and_error(int) returns int as
  $$ select 1 / $1 $$ language sql parallel safe
  set role = regress_parallel_worker;
