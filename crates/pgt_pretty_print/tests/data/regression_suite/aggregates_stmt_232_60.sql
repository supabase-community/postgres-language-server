select array_agg(distinct val)
from (select null as val from generate_series(1, 2));
