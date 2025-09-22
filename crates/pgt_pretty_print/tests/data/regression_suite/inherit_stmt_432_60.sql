insert into inhpar select x, x::text from generate_series(1,5) x;
