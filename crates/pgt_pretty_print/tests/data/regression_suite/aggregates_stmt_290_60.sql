select string_agg(distinct f1, ',' order by f1::text) from varchar_tbl;
