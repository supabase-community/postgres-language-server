INSERT INTO mcv_lists (a, b, c, filler1)
     SELECT i, i, i, i FROM generate_series(1,1000) s(i);
