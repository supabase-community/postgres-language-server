INSERT INTO idxpart (a, b, c) SELECT i, i, i FROM generate_series(1, 50) i;
