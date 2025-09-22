INSERT INTO prt1_m SELECT i, i, i % 25 FROM generate_series(0, 599, 2) i;
