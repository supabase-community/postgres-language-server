select string_agg(distinct f1, ',') filter (where length(f1) > 1)
from varchar_tbl;
