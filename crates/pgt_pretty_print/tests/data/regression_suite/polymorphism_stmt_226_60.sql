create function dfunc(a int = 1, int = 2) returns int as $$
  select $1 + $2;
$$ language sql;
