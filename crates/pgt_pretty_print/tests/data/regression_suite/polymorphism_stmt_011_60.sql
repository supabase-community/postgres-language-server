create function polyf(x anyarray) returns anyarray as $$
  select x
$$ language sql;
