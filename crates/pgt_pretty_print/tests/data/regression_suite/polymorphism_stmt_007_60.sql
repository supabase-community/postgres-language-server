create function polyf(x anyarray) returns anyelement as $$
  select x[1]
$$ language sql;
