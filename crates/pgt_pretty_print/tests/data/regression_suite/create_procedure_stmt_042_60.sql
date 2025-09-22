CREATE PROCEDURE ptest6b(a anyelement, out b anyelement, out c anyarray)
LANGUAGE SQL
AS $$
SELECT $1, array[$1];
$$;
