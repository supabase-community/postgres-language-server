select
  unique1,
  (select t.unique1 from tenk1 where tenk1.unique1 = t.unique1)
from tenk1 t, generate_series(1, 1000)
order by 1, 2;
