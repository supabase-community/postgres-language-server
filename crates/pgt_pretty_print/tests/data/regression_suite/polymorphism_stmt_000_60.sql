create function polyf(x anyelement) returns anyelement as $$
  select x + 1
$$ language sql;
