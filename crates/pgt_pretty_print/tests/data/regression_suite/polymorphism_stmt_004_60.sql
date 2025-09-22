create function polyf(x anyelement) returns anyarray as $$
  select array[x + 1, x + 2]
$$ language sql;
