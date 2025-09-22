create function dfunc(int = 1, int = 2) returns int as $$
  select 2;
$$ language sql;
