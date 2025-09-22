select count((unique1)) from tenk1
where hundred = any ((select array_agg(i) from generate_series(1, 100, 15) i)::int[]);
