insert into hp_prefix_test
select
  case a when 0 then null else 1 end,
  case b when 0 then null else 2 end,
  case c when 0 then null else 3 end,
  case d when 0 then null else 4 end
from
  generate_series(0,1) a,
  generate_series(0,1) b,
  generate_Series(0,1) c,
  generate_Series(0,1) d;
