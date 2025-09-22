insert into semijoin_unique_tbl select i%10, i%10 from generate_series(1,1000)i;
