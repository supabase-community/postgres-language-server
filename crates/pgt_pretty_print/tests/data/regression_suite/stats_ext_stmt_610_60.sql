INSERT INTO mcv_lists_partial (a, b, c)
     SELECT
         i,
         i,
         i
     FROM generate_series(0,3999) s(i);
