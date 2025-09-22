create or replace function testrngfunc() returns setof rngfunc_type as $$
  select 1, 2 union select 3, 4 order by 1;
$$ language sql immutable;
