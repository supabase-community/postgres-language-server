INSERT INTO prt2_e SELECT i, i, i % 25 FROM generate_series(0, 599, 3) i;
