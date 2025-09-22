INSERT INTO ndistinct (a, b, c, filler1)
     SELECT mod(i,13), mod(i,17), mod(i,19),
            mod(i,23) || ' dollars and zero cents'
       FROM generate_series(1,1000) s(i);
