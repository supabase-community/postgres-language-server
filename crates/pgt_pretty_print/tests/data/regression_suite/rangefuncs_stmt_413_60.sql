create function extractq2_2_opt(t int8_tbl) returns table(ret1 int8) as $$
  select extractq2(t)
$$ language sql immutable;
