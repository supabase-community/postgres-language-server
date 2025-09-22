create function polyf(x anyelement) returns anyrange as $$
  select array[x + 1, x + 2]
$$ language sql;
