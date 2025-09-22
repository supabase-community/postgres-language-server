select string_agg(distinct f1::text, ',' order by f1) from varchar_tbl;
