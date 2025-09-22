select f1, sql_if(f1 > 0, bleat(f1), bleat(f1 + 1)) from int4_tbl;
