create function polyf(x anycompatible) returns anycompatiblemultirange as $$
  select array[x + 1, x + 2]
$$ language sql;
