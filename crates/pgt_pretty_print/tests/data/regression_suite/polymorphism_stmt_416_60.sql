create function anyctest(anycompatible)
returns anycompatiblemultirange as $$
  select $1
$$ language sql;
