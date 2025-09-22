select x, y,
       first_value(y) over w,
       last_value(y) over w
from
  (select x, x as y from generate_series(1,5) as x
   union all select null, 42
   union all select null, 43) ss
window w as
  (order by x desc nulls last range between 2 preceding and 2 following);
