select distinct sub.unique1, stringu1
from tenk1, lateral (select tenk1.unique1 from generate_series(1, 1000)) as sub;
