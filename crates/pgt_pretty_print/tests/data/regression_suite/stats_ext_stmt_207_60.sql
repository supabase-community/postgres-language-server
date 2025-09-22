INSERT INTO ndistinct (a, b, c, d)
     SELECT mod(i,3), mod(i,9), mod(i,5), mod(i,20)
       FROM generate_series(1,1000) s(i);
