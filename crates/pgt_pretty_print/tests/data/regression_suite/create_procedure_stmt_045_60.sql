CREATE PROCEDURE ptest6c(inout a anyelement, inout b anyelement)
LANGUAGE SQL
AS $$
SELECT $1, 1;
$$;
