insert into unique_tbl_p select i%12, i from generate_series(0, 1000)i;
