CREATE OR REPLACE FUNCTION rngfunc(in f1 int, out f2 int) RETURNS float
AS 'select $1+1' LANGUAGE sql;
