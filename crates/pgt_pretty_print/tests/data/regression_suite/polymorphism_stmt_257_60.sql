create function dfunc(out int = 20) returns int as $$
  select 1;
$$ language sql;
