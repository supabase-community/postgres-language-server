insert into inhcld select x::text, x from generate_series(6,10) x;
