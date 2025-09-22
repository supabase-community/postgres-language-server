CREATE OR REPLACE FUNCTION rngfunc(in f1 int, out f2 int, out f3 text) RETURNS int
AS 'select $1+1' LANGUAGE sql;
