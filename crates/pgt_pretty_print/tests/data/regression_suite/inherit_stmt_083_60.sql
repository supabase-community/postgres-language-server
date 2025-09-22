insert into some_tab_child select i, i+1, 0 from generate_series(1,1000) i;
