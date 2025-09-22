insert into tbl_ra select i, i%100 from generate_series(1,1000)i;
