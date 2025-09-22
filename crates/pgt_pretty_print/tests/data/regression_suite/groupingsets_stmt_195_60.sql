select a, b, row_number() over (order by a, b nulls first)
from (values (1, 1), (2, 2)) as t (a, b) where a = b
group by grouping sets((a, b), (a));
