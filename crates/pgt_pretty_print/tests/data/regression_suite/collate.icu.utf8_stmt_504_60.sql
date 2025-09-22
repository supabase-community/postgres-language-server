INSERT INTO pagg_tab3 SELECT i % 4 + 1, substr('abAB', (i % 4) + 1 , 1) FROM generate_series(0, 19) i;
