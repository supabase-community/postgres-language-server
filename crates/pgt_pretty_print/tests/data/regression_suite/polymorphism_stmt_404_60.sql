create function anyctest(anycompatible)
returns anycompatiblerange as $$
  select $1
$$ language sql;
