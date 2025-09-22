INSERT INTO mcv_lists_partial (a, b, c)
     SELECT
         mod(i,10),
         mod(i,10),
         mod(i,10)
     FROM generate_series(0,999) s(i);
