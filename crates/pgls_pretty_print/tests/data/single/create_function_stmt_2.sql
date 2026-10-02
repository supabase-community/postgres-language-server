CREATE FUNCTION public.sql_func_with_transform(int) RETURNS int LANGUAGE sql
AS 'select $1 + 1'
TRANSFORM FOR TYPE int, FOR TYPE text;
