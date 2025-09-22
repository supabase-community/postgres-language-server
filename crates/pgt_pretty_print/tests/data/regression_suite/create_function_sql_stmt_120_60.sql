CREATE FUNCTION functest_srf0() RETURNS SETOF int
LANGUAGE SQL
AS $$ SELECT i FROM generate_series(1, 100) i $$;
