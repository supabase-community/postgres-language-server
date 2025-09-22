INSERT INTO mcv_lists (a, b, c, ia, filler1)
     SELECT mod(i,100), mod(i,50), mod(i,25), array[mod(i,25)], i
       FROM generate_series(1,5000) s(i);
