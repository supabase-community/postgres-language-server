CREATE FUNCTION add(a integer, b integer) RETURNS integer
AS 'SELECT $1 + $2' LANGUAGE SQL;