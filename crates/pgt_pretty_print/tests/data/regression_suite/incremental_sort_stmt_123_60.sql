insert into prt_tbl select i%200, i from generate_series(1,1000)i;
