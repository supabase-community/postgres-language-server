insert into exists_tbl select x, x/2, x+1 from generate_series(0,10) x;
