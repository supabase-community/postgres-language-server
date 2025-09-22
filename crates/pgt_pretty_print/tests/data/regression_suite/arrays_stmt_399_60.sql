select array_agg(array[i, nullif(i, 3), i+1]) from generate_series(1,4) g(i);
