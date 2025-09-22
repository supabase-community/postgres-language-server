create function dfunc(int = 1, int = 2, int = 3) returns int as $$
  select 3;
$$ language sql;
