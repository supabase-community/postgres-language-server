create function testrngfunc() returns rngfunc_type as $$
  select 7.136178319899999964, 7.136178319899999964;
$$ language sql immutable;
