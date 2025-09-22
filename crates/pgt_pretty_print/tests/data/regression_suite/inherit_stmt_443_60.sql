insert into inhpar select x, x::text from generate_series(1,10) x;
