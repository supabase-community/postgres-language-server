INSERT INTO ndistinct (a, b, c, filler1)
     SELECT i/100, i/100, i/100, (i/100) || ' dollars and zero cents'
       FROM generate_series(1,1000) s(i);
