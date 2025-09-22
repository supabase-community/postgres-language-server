select q from (select max(f1) from int4_tbl group by f1 order by f1) q;
