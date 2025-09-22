select distinct on (a, b+1) a, b+1
from (values (1, 0), (2, 1)) as t (a, b) where a = b+1
group by grouping sets((a, b+1), (a))
order by a, b+1;
