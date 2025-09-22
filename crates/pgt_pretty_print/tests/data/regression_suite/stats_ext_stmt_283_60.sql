INSERT INTO functional_dependencies (a, b, c, filler1)
     SELECT i, i, i, i FROM generate_series(1,5000) s(i);
