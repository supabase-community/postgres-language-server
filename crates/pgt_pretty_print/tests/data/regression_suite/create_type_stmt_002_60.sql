CREATE FUNCTION int44in(cstring)
   RETURNS city_budget
   AS 'regresslib'
   LANGUAGE C STRICT IMMUTABLE;
