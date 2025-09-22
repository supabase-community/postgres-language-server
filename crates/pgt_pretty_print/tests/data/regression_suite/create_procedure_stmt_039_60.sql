CREATE PROCEDURE ptest6a(inout a anyelement, out b anyelement)
LANGUAGE SQL
AS $$
SELECT $1, $1;
$$;
