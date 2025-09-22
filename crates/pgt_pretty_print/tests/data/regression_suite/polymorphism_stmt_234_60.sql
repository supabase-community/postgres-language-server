create function dfunc(a int = 1, b int) returns int as $$
  select $1 + $2;
$$ language sql;
