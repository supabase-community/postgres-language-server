select (1 = any(array_agg(f1))) = any (select false) from int4_tbl;
