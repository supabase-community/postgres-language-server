create function extractq2(t int8_tbl) returns int8 as $$
  select t.q2
$$ language sql immutable;
