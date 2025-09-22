select distinct on (a, b) a, b
from (values (1, 1), (2, 2)) as t (a, b) where a = b
group by grouping sets((a, b), (a))
order by a, b;
