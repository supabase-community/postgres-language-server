create function dfunc(a int = 1, out sum int, b int = 2) as $$
  select $1 + $2;
$$ language sql;
