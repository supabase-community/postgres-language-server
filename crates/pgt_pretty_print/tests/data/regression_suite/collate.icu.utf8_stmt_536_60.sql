INSERT INTO pagg_tab5 (b, c) SELECT substr('abAB', (i % 4) + 1 , 1), substr('abAB', (i % 2) + 1 , 1) FROM generate_series(0, 5) i;
