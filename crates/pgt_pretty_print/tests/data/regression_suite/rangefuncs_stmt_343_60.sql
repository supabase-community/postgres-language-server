create function testrngfunc() returns setof rngfunc_type as $$
  select 7.136178319899999964, 7.136178319899999964;
$$ language sql immutable;
