create function extractq2_2(t int8_tbl) returns table(ret1 int8) as $$
  select extractq2(t) offset 0
$$ language sql immutable;
