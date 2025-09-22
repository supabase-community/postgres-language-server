CREATE FUNCTION tftest(int) returns table(a int, b int) as $$
SELECT $1, $1+i FROM generate_series(1,5) g(i);
$$ LANGUAGE sql IMMUTABLE STRICT;
