select distinct on (1) floor(random()) as r, f1 from int4_tbl order by 1,2;
