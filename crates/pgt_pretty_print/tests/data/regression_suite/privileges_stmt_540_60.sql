CREATE FUNCTION priv_testfunc3(int) RETURNS int AS 'select 2 * $1;' LANGUAGE sql;
