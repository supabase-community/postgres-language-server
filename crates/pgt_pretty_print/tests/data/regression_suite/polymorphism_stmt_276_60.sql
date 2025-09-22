create function dfunc(text) returns text as $$
  select $1;
$$ language sql;
