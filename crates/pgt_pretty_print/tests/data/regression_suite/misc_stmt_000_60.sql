CREATE FUNCTION overpaid(emp)
   RETURNS bool
   AS 'regresslib'
   LANGUAGE C STRICT;
