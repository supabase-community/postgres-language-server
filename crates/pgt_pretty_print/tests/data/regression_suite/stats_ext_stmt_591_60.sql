INSERT INTO mcv_lists_arrays (a, b, c)
     SELECT
         ARRAY[fipshash((i/100)::text), fipshash((i/100-1)::text), fipshash((i/100+1)::text)],
         ARRAY[(i/100-1)::numeric/1000, (i/100)::numeric/1000, (i/100+1)::numeric/1000],
         ARRAY[(i/100-1), i/100, (i/100+1)]
     FROM generate_series(1,5000) s(i);
