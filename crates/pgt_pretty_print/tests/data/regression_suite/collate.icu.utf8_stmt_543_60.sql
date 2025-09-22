INSERT INTO pagg_tab6 (b, c) SELECT substr('cdCD', (i % 4) + 1 , 1), substr('cdCD', (i % 2) + 1 , 1) FROM generate_series(0, 5) i;
