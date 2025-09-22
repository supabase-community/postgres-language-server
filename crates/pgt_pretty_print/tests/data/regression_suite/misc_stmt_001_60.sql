CREATE FUNCTION reverse_name(name)
   RETURNS name
   AS 'regresslib'
   LANGUAGE C STRICT;
